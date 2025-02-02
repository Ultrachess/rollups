/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

use async_mutex::Mutex;
use indexer::config::IndexerConfig;
use indexer::{data_service, db_service, error::Error::BadConfiguration, http};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Configure a custom event formatter
    let tracing_format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(true)
        .with_target(true)
        .with_ansi(false)
        .compact();

    // Use tracing library for logs. By default use system standard output logger
    if std::env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_max_level(LevelFilter::INFO)
            .init();
    }

    info!("Starting rollups indexer service");
    let indexer_config =
        IndexerConfig::initialize().map_err(|e| BadConfiguration {
            err: format!("Fail to initialize indexer config: {}", e),
        })?;

    info!("Indexer configuration dapp_contract_address={} state_server_endpoint={} initial_epoch={}\n\
        interval={} server manager endpoint={} session_id={} postgres db host/port {}:{}",
        &indexer_config.dapp_contract_address,
        &indexer_config.state_server_endpoint,
        &indexer_config.initial_epoch,
        &indexer_config.interval,
        &indexer_config.mm_endpoint,
        &indexer_config.session_id,
        &indexer_config.database.postgres_hostname,
        &indexer_config.database.postgres_port,
        );

    // Perform migration if it was not performed before
    {
        let postgres_config = indexer_config.database.clone();
        tokio::task::spawn_blocking(move || {
            info!("Performing migrations if they are not performed before from directory {}", postgres_config.postgres_migration_folder);

            // Perform diesel setup and migration
            match rollups_data::database::perform_diesel_setup(
                &postgres_config.postgres_hostname,
                postgres_config.postgres_port,
                &postgres_config.postgres_user,
                &postgres_config.postgres_password,
                &postgres_config.postgres_db,
                &postgres_config.postgres_migration_folder,
            ) {
                Ok(process_output) => {
                    if process_output.status.success() {
                        info!("Migration successfully performed for database {}", postgres_config.postgres_db)
                    } else {
                        error!("Error while performing migrations of database {} diesel setup exit status: {}, error output: {}",
                            postgres_config.postgres_db, process_output.status, std::str::from_utf8(&process_output.stderr).unwrap_or_default());
                        panic!("Migration failed!");
                    }
                }
                Err(e) => {
                    error!(
                        "Error while performing migrations of database {} error details: {}",
                        postgres_config.postgres_db, e.to_string()
                    );
                    panic!("Migration failed!");

                }
            };
        })
        .await
            .expect("Migration/setup executed successfully");
        info!("Database migrations finished");
    }

    // Channel for passing messaged between data service acquiring data from outside
    // and database service inserting data to the library
    let (message_tx, message_rx) =
        mpsc::channel::<rollups_data::database::Message>(128);

    // Shared health status state variable
    let health_status = Arc::new(Mutex::new(http::HealthStatus {
        server_manager: Ok(()),
        state_server: Ok(()),
        postgres: Ok(()),
        indexer_status: Ok(()),
    }));

    // Run database and data services
    tokio::select! {
        db_service_result = db_service::run(indexer_config.clone(), message_rx, health_status.clone()) => {
            match db_service_result {
                Ok(_) => info!("db service terminated successfully"),
                Err(e) => error!("db service terminated with error: {}", e)
            }
        },
        data_service_result = data_service::run(indexer_config.clone(), message_tx, health_status.clone()) => {
            match data_service_result {
                Ok(_) => info!("data service terminated successfully"),
                Err(e) => error!("data service terminated with error: {}", e)
            }
        },
        http_health_service = http::start_http_service(&indexer_config.health_endpoint.0,
            indexer_config.health_endpoint.1, health_status) => {
            match http_health_service {
                Ok(_) => info!("http health service terminated successfully"),
                Err(e) => error!("http health service terminated with error: {}", e)
            }
        }
    }
    info!("Ended rollups indexer service");
    Ok(())
}
