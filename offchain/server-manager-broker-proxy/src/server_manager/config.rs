// Copyright 2022 Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use clap::Parser;

use crate::grpc::cartesi_machine::{ConcurrencyConfig, MachineRuntimeConfig};
use crate::grpc::cartesi_server_manager::{CyclesConfig, DeadlineConfig};

#[derive(Debug)]
pub struct ServerManagerConfig {
    pub server_manager_endpoint: String,
    pub session_id: String,
    pub pending_inputs_sleep_duration: u64,
    pub pending_inputs_max_retries: u64,
    pub runtime_config: MachineRuntimeConfig,
    pub deadline_config: DeadlineConfig,
    pub cycles_config: CyclesConfig,
}

impl ServerManagerConfig {
    pub fn parse_from_cli(cli_config: ServerManagerCLIConfig) -> Self {
        let runtime_config = MachineRuntimeConfig {
            concurrency: Some(ConcurrencyConfig {
                update_merkle_tree: cli_config
                    .sm_concurrency_update_merkle_tree,
            }),
        };

        let deadline_config = DeadlineConfig {
            checkin: cli_config.sm_deadline_checkin,
            advance_state: cli_config.sm_deadline_advance_state,
            advance_state_increment: cli_config
                .sm_deadline_advance_state_increment,
            inspect_state: cli_config.sm_deadline_inspect_state,
            inspect_state_increment: cli_config
                .sm_deadline_inspect_state_increment,
            machine: cli_config.sm_deadline_machine,
            store: cli_config.sm_deadline_store,
            fast: cli_config.sm_deadline_fast,
        };

        let cycles_config = CyclesConfig {
            max_advance_state: cli_config.sm_cycles_max_advance_state,
            advance_state_increment: cli_config
                .sm_cycles_advance_state_increment,
            max_inspect_state: cli_config.sm_cycles_max_inspect_state,
            inspect_state_increment: cli_config
                .sm_cycles_inspect_state_increment,
        };

        Self {
            server_manager_endpoint: cli_config.server_manager_endpoint,
            session_id: cli_config.session_id,
            pending_inputs_sleep_duration: cli_config
                .sm_pending_inputs_sleep_duration,
            pending_inputs_max_retries: cli_config
                .sm_pending_inputs_max_retries,
            runtime_config,
            deadline_config,
            cycles_config,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "server-manager")]
pub struct ServerManagerCLIConfig {
    /// Server-manager gRPC endpoint
    #[arg(long, env, default_value = "http://127.0.0.1:5001")]
    pub server_manager_endpoint: String,

    /// Server-manager session id
    #[arg(long, env, default_value = "default_rollups_id")]
    pub session_id: String,

    /// Sleep duration while polling for server-manager pending inputs (in millis)
    #[arg(long, env, default_value_t = 1000)]
    pub sm_pending_inputs_sleep_duration: u64,

    /// Max number of retries while polling server-manager for pending inputs
    #[arg(long, env, default_value_t = 600)]
    pub sm_pending_inputs_max_retries: u64,

    /// Defines the number of threads to use while calculating the merkle tree
    #[arg(long, env, default_value_t = 0)]
    pub sm_concurrency_update_merkle_tree: u64,

    /// Deadline for receiving checkin from spawned machine server
    #[arg(long, env, default_value_t = 5 * 1000)]
    pub sm_deadline_checkin: u64,

    /// Deadline for advancing the state
    #[arg(long, env, default_value_t = 1000 * 60 * 3)]
    pub sm_deadline_advance_state: u64,

    /// Deadline for each increment when advancing state
    #[arg(long, env, default_value_t = 1000 * 10)]
    pub sm_deadline_advance_state_increment: u64,

    /// Deadline for inspecting state
    #[arg(long, env, default_value_t = 1000 * 60 * 3)]
    pub sm_deadline_inspect_state: u64,

    /// Deadline for each increment when inspecting state
    #[arg(long, env, default_value_t = 1000 * 10)]
    pub sm_deadline_inspect_state_increment: u64,

    /// Deadline for instantiating a machine
    #[arg(long, env, default_value_t = 1000 * 60 * 5)]
    pub sm_deadline_machine: u64,

    /// Deadline for storing a machine
    #[arg(long, env, default_value_t = 1000 * 60 * 3)]
    pub sm_deadline_store: u64,

    /// Deadline for quick machine server tasks
    #[arg(long, env, default_value_t = 1000 * 5)]
    pub sm_deadline_fast: u64,

    /// Maximum number of cycles that processing the input in an AdvanceState can take
    #[arg(long, env, default_value_t = u64::MAX >> 2)]
    pub sm_cycles_max_advance_state: u64,

    /// Number of cycles in each increment to processing an input
    #[arg(long, env, default_value_t = 1 << 22)]
    pub sm_cycles_advance_state_increment: u64,

    /// Maximum number of cycles that processing the query in an InspectState can take
    #[arg(long, env, default_value_t = u64::MAX >> 2)]
    pub sm_cycles_max_inspect_state: u64,

    /// Number of cycles in each increment to processing a query
    #[arg(long, env, default_value_t = 1 << 22)]
    pub sm_cycles_inspect_state_increment: u64,
}
