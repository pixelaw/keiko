use std::path::PathBuf;
use clap::{Args, Parser};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use url::Url;
use std::str::FromStr;
use keiko_api::server_state;
use std::net::SocketAddr;

const LOCAL_KATANA: &str = "http://0.0.0.0:5050";
const LOCAL_TORII: &str = "http://0.0.0.0:8080";

pub const KEIKO_ASSETS: &str = "static/keiko/assets";
pub const KEIKO_INDEX: &str = "static/keiko/index.html";
pub const KATANA_LOG: &str = "log/katana.log.json";
pub const TORII_LOG: &str = "log/torii.log";
pub const TORII_DB: &str = "torii.sqlite";


#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerOptions,
    pub starknet: StarknetOptions,
    pub katana: KatanaOptions,
    pub torii: ToriiOptions,
    pub world_address: String,
}


impl From<KeikoArgs> for Config {
    fn from(args: KeikoArgs) -> Self {
        Self {
            server: args.server,
            starknet: args.starknet,
            katana: args.katana,
            torii: args.torii,
            world_address: "".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let keiko_args = KeikoArgs::parse();
        let config = Self::from(keiko_args);

        config
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct KeikoArgs {
    #[command(flatten)]
    #[command(next_help_heading = "Server options")]
    pub server: ServerOptions,

    #[command(flatten)]
    #[command(next_help_heading = "Starknet options")]
    pub starknet: StarknetOptions,

    #[command(flatten)]
    #[command(next_help_heading = "Katana options")]
    pub katana: KatanaOptions,

    #[command(flatten)]
    #[command(next_help_heading = "Torii options")]
    pub torii: ToriiOptions,
}

#[derive(Debug, Args, Clone)]
pub struct ServerOptions {
    #[arg(long)]
    #[arg(default_value = "3000")]
    #[arg(help = "Port number to listen on.")]
    #[arg(env = "SERVER_PORT")]
    pub port: u16,

    #[arg(short, long)]
    #[arg(default_value = "contracts")]
    #[arg(value_parser = PathBuf::from_str)]
    #[arg(help = "Path to the contracts directory")]
    #[arg(env = "CONTRACT_PATH")]
    pub contract_path: PathBuf,

    #[arg(long)]
    #[arg(default_value = "static")]
    #[arg(value_parser = PathBuf::from_str)]
    #[arg(help = "Path to the static directory")]
    #[arg(env = "STATIC_PATH")]
    pub static_path: PathBuf,

    #[arg(long)]
    #[arg(default_value = "false")]
    #[arg(help = "Builds the dashboard if set to true")]
    #[arg(env = "PROD")]
    pub prod: bool,
}


#[derive(Debug, Args, Clone)]
pub struct StarknetOptions {
    #[arg(long)]
    #[arg(help = "Disable charging fee for transactions.")]
    #[arg(env = "DISABLE_FEE")]
    pub disable_fee: bool,

    #[arg(long)]
    #[arg(help = "Disable validation when executing transactions.")]
    #[arg(env = "DISABLE_VALIDATE")]
    pub disable_validate: bool,

    #[command(flatten)]
    #[command(next_help_heading = "Environment options")]
    pub environment: EnvironmentOptions,
}

#[derive(Debug, Args, Clone)]
pub struct EnvironmentOptions {
    #[arg(long)]
    #[arg(help = "The chain ID.")]
    #[arg(default_value = "KATANA")]
    #[arg(env = "CHAIN_ID")]
    pub chain_id: String,

    #[arg(long)]
    #[arg(help = "The gas price.")]
    #[arg(env = "GAS_PRICE")]
    pub gas_price: Option<u128>,

    #[arg(long)]
    #[arg(help = "The maximum number of steps available for the account validation logic.")]
    #[arg(env = "VALIDATE_MAX_STEPS")]
    pub validate_max_steps: Option<u32>,

    #[arg(long)]
    #[arg(help = "The maximum number of steps available for the account execution logic.")]
    #[arg(env = "INVOKE_MAX_STEPS")]
    pub invoke_max_steps: Option<u32>,
}

#[derive(Debug, Args, Clone)]
pub struct ToriiOptions {
    #[arg(long)]
    #[arg(env = "TORII_SEQUENCER_RPC")]
    #[arg(help = "The sequencer rpc endpoint to index.")]
    pub torii_sequencer_rpc: Option<Url>,

    #[arg(long)]
    #[arg(env = "TORII_EXTERNAL_URL")]
    #[arg(help = "The external url of the Torii server, used when hosting")]
    pub torii_external_url: Option<Url>,
}

#[derive(Debug, Args, Clone)]
pub struct KatanaOptions {
    #[arg(long)]
    #[arg(help = "The world address Keiko uses")]
    #[arg(env = "WORLD_ADDRESS")]
    pub world_address: String,

    #[arg(long)]
    #[arg(help = "Don't print anything on startup.")]
    pub katana_silent: bool,

    #[arg(long)]
    #[arg(conflicts_with = "katana_block_time")]
    #[arg(help = "Disable auto and interval mining, and mine on demand instead via an endpoint.")]
    pub katana_no_mining: bool,

    #[arg(short, long)]
    #[arg(value_name = "MILLISECONDS")]
    #[arg(help = "Block time in milliseconds for interval mining.")]
    pub katana_block_time: Option<u64>,

    #[arg(long)]
    #[arg(value_name = "PATH")]
    #[arg(env = "KATANA_DB_DIR")]
    #[arg(help = "Directory path of the database to initialize from.")]
    #[arg(long_help = "Directory path of the database to initialize from. The path must either \
                       be an empty directory or a directory which already contains a previously \
                       initialized Katana database.")]
    pub katana_db_dir: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_name = "URL")]
    #[arg(help = "The Starknet RPC provider to fork the network from.")]
    pub katana_rpc_url: Option<Url>,

    #[arg(long)]
    pub katana_dev: bool,

    #[arg(long)]
    #[arg(help = "Output logs in JSON format.")]
    pub katana_json_log: bool,

    /// Enable Prometheus metrics.
    ///
    /// The metrics will be served at the given interface and port.
    // #[arg(long, value_name = "SOCKET", value_parser = parse_socket_address, help_heading = "Metrics")]
    pub katana_metrics: Option<SocketAddr>,

    #[arg(long)]
    #[arg(requires = "katana_rpc_url")]
    #[arg(value_name = "BLOCK_NUMBER")]
    #[arg(help = "Fork the network at a specific block.")]
    pub katana_fork_block_number: Option<u64>,

    #[cfg(feature = "messaging")]
    #[arg(long)]
    #[arg(value_name = "PATH")]
    #[arg(value_parser = katana_core::service::messaging::MessagingConfig::parse)]
    #[arg(help = "Configure the messaging with an other chain.")]
    #[arg(long_help = "Configure the messaging to allow Katana listening/sending messages on a \
                       settlement chain that can be Ethereum or an other Starknet sequencer. \
                       The configuration file details and examples can be found here: https://book.dojoengine.org/toolchain/katana/reference#messaging")]
    pub katana_messaging: Option<katana_core::service::messaging::MessagingConfig>,

}


impl Config {
    pub fn get_storage_base_dir(&self) -> String {
        format!("storage/{}", self.katana.world_address)
    }

    pub fn get_torii_args(&self) -> Vec<String> {
        let mut args = vec![
            "--world".to_string(),
            self.katana.world_address.clone(),
            "--database".to_string(),
            format!("{}/{}", self.get_storage_base_dir().clone(), TORII_DB),
        ];

        if let Some(torii_external_url) = &self.torii.torii_external_url {
            args.push("--external-url".to_string());
            args.push(torii_external_url.to_string());
        }

        if let Some(sequencer_rpc) = &self.torii.torii_sequencer_rpc {
            args.push("--rpc".to_string());
            args.push(sequencer_rpc.to_string());
        }

        args
    }

    pub fn get_katana_args(&self) -> Vec<String> {
        let mut args = vec![];

        args.push("--db-dir".to_string());
        args.push(format!("{}/katana-db", self.get_storage_base_dir()));

        if self.katana.katana_dev {
            args.push("--dev".to_string())
        }

        if self.katana.katana_silent {
            args.push("--silent".to_string())
        }

        if self.katana.katana_no_mining {
            args.push("--no-mining".to_string())
        }

        if let Some(block_time) = &self.katana.katana_block_time {
            args.push("--block-time".to_string());
            args.push(block_time.to_string());
        }

        if let Some(rpc_url) = &self.katana.katana_rpc_url {
            args.push("--rpc-url".to_string());
            args.push(rpc_url.to_string())
        }

        args.push("--json-log".to_string());

        if let Some(fork_block_number) = &self.katana.katana_fork_block_number {
            args.push("--fork-block-number".to_string());
            args.push(fork_block_number.to_string())
        }

        if self.starknet.disable_fee {
            args.push("--disable-fee".to_string())
        }

        if self.starknet.disable_validate {
            args.push("--disable-validate".to_string())
        }

        args.push("--chain-id".to_string());
        args.push(self.starknet.environment.chain_id.clone());

        if let Some(gas_price) = &self.starknet.environment.gas_price {
            args.push("--gas-price".to_string());
            args.push(gas_price.to_string());
        }

        if let Some(validate_max_steps) = &self.starknet.environment.validate_max_steps {
            args.push("--validate-max-steps".to_string());
            args.push(validate_max_steps.to_string());
        }

        if let Some(invoke_max_steps) = &self.starknet.environment.invoke_max_steps {
            args.push("--invoke-max-steps".to_string());
            args.push(invoke_max_steps.to_string());
        }

        args.push("--genesis".to_string());
        args.push(format!("{}/config/genesis.json", self.get_storage_base_dir()));

        args
    }


    /**
     * creates a json_rpc_client from katana
     */
    pub fn json_rpc_client(&self) -> HttpClient {
        HttpClientBuilder::default()
            .build(self.rpc_url())
            .unwrap()
    }

    /**
     * creates the rpc_url
     */
    pub fn rpc_url(&self) -> Url {
        Url::parse(LOCAL_KATANA).unwrap()
    }

    /*
    *    creates the torii_url
    */
    pub fn torii_url(&self) -> Url {
        Url::parse(LOCAL_TORII).unwrap()
    }

    /*
    *    gets the server state
    */
    pub fn server_state(&self) -> server_state::ServerState {
        let manifest_base_dir = format!("{}/manifests", self.get_storage_base_dir());

        server_state::ServerState {
            json_rpc_client: self.json_rpc_client(),
            rpc_url: self.rpc_url(),
            manifest_base_dir,
            torii_url: self.torii_url(),
        }
    }
}
