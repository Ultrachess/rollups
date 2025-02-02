pub mod config;

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use std::net::SocketAddr;

pub async fn start_health_check(host: &str, port: u16) -> Result<()> {
    tracing::info!(
        "Starting dispatcher health check endpoint at http://{}:{}/healthz",
        host,
        port
    );

    let addr = SocketAddr::new(
        host.parse().context("could not parse host address")?,
        port,
    );

    let app = Router::new().route("/healthz", get(|| async { "" }));

    let ret = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(ret)
}
