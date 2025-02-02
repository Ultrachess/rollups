use crate::machine::config::{BrokerConfig, BrokerEnvCLIConfig};
use state_client_lib::config::{Error as SCError, SCConfig, SCEnvCLIConfig};
use tx_manager::config::{Error as TxError, TxEnvCLIConfig, TxManagerConfig};

use crate::http_health::config::{HealthCheckConfig, HealthCheckEnvCLIConfig};

use state_fold_types::ethers::types::{Address, U256};

use snafu::{ResultExt, Snafu};
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(name = "rd_config", about = "Configuration for rollups dispatcher")]
pub struct DispatcherEnvCLIConfig {
    #[structopt(flatten)]
    pub sc_config: SCEnvCLIConfig,

    #[structopt(flatten)]
    pub tx_config: TxEnvCLIConfig,

    #[structopt(flatten)]
    pub broker_config: BrokerEnvCLIConfig,

    #[structopt(flatten)]
    pub hc_config: HealthCheckEnvCLIConfig,

    /// Address of rollups dapp
    #[structopt(long, env)]
    pub rd_dapp_contract_address: Option<String>,

    /// Path to file with address of rollups dapp
    #[structopt(long, env)]
    pub rd_dapp_contract_address_file: Option<String>,

    /// First epoch that dispatcher will look at. Default zero.
    #[structopt(long, env)]
    pub rd_initial_epoch: Option<U256>,

    /// Minimum required fee for making claims. A value of zero means an
    /// altruistic validator; the node will always make claims regardless
    /// of fee.
    #[structopt(long, env)]
    pub rd_minimum_required_fee: Option<U256>,

    /// Number of future claim fees that the fee manager should have
    /// uncommitted.
    #[structopt(long, env)]
    pub rd_num_buffer_epochs: Option<usize>,

    /// Number of claims before validator redeems fee.
    #[structopt(long, env)]
    pub rd_num_claims_trigger_redeem: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct DispatcherConfig {
    pub sc_config: SCConfig,
    pub tx_config: TxManagerConfig,
    pub broker_config: BrokerConfig,
    pub hc_config: HealthCheckConfig,

    pub dapp_contract_address: Address,
    pub initial_epoch: U256,

    pub minimum_required_fee: U256,
    pub num_buffer_epochs: usize,
    pub num_claims_trigger_redeem: usize,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("StateClient configuration error: {}", source))]
    StateClientError { source: SCError },

    #[snafu(display("TxManager configuration error: {}", source))]
    TxManagerError { source: TxError },

    #[snafu(display("Configuration missing dapp address"))]
    MissingDappAddress {},

    #[snafu(display("Dapp address string parse error"))]
    DappAddressParseError { source: rustc_hex::FromHexError },

    #[snafu(display("Dapp address read file error"))]
    DappJsonReadFileError { source: std::io::Error },

    #[snafu(display("Dapp json parse error"))]
    DappJsonParseError { source: serde_json::Error },

    #[snafu(display("Dapp json wrong type error"))]
    DappJsonWrongTypeError {},
}

pub type Result<T> = std::result::Result<T, Error>;

const DEFAULT_MINIMUM_REQUIRED_FEE: U256 = U256::zero(); // altruistic
const DEFAULT_NUM_BUFFER_EPOCHS: usize = 4;
const DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM: usize = 4;
const DEFAULT_INITIAL_EPOCH: U256 = U256::zero();

impl DispatcherConfig {
    pub fn initialize_from_args() -> Result<Self> {
        let env_cli_config = DispatcherEnvCLIConfig::from_args();
        Self::initialize(env_cli_config)
    }

    pub fn initialize(env_cli_config: DispatcherEnvCLIConfig) -> Result<Self> {
        let sc_config = SCConfig::initialize(env_cli_config.sc_config)
            .context(StateClientSnafu)?;

        let tx_config = TxManagerConfig::initialize(env_cli_config.tx_config)
            .context(TxManagerSnafu)?;

        let hc_config = HealthCheckConfig::initialize(env_cli_config.hc_config);

        let dapp_contract_address =
            if let Some(a) = env_cli_config.rd_dapp_contract_address {
                Address::from_str(&a).context(DappAddressParseSnafu)?
            } else {
                let path = env_cli_config
                    .rd_dapp_contract_address_file
                    .ok_or(snafu::NoneError)
                    .context(MissingDappAddressSnafu)?;

                let file =
                    File::open(path.clone()).context(DappJsonReadFileSnafu)?;
                let reader = BufReader::new(file);
                let json: serde_json::Value = serde_json::from_reader(reader)
                    .context(DappJsonParseSnafu)?;

                let contents = match &json["address"] {
                    serde_json::Value::String(ref s) => s.clone(),

                    serde_json::Value::Null => {
                        return MissingDappAddressSnafu.fail()
                    }

                    _ => return DappJsonWrongTypeSnafu.fail(),
                };

                Address::from_str(&contents.trim().to_string())
                    .context(DappAddressParseSnafu)?
            };

        let broker_config = BrokerConfig::initialize(
            env_cli_config.broker_config,
            tx_config.chain_id,
            dapp_contract_address.clone().to_fixed_bytes(),
        );

        let initial_epoch = env_cli_config
            .rd_initial_epoch
            .unwrap_or(DEFAULT_INITIAL_EPOCH);

        let minimum_required_fee = env_cli_config
            .rd_minimum_required_fee
            .unwrap_or(DEFAULT_MINIMUM_REQUIRED_FEE);

        let num_buffer_epochs = env_cli_config
            .rd_num_buffer_epochs
            .unwrap_or(DEFAULT_NUM_BUFFER_EPOCHS);

        let num_claims_trigger_redeem: usize = env_cli_config
            .rd_num_claims_trigger_redeem
            .unwrap_or(DEFAULT_NUM_CLAIMS_TRIGGER_REDEEM);

        assert!(
            sc_config.default_confirmations < tx_config.default_confirmations,
            "`state-client confirmations` has to be less than `tx-manager confirmations,`"
        );

        Ok(DispatcherConfig {
            sc_config,
            tx_config,
            broker_config,
            hc_config,

            dapp_contract_address,
            initial_epoch,

            minimum_required_fee,
            num_buffer_epochs,
            num_claims_trigger_redeem,
        })
    }
}
