use std::path::PathBuf;
use clap::{Args, Parser};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use url::Url;
use std::str::FromStr;
use keiko_api::server_state;

const LOCAL_KATANA: &str = "http://0.0.0.0:5050";
const LOCAL_TORII: &str = "http://0.0.0.0:8080";

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
    #[command(next_help_heading = "Slot options")]
    pub slot: SlotOptions,

    #[command(flatten)]
    #[command(next_help_heading = "World options")]
    pub world: WorldOptions,

    #[command(flatten)]
    #[command(next_help_heading = "Katana options")]
    pub katana: KatanaOptions
}

#[derive(Debug, Args, Clone)]
pub struct KatanaOptions {
    #[arg(long)]
    #[arg(help = "Don't print anything on startup.")]
    #[arg(env = "KATANA_SILENT_LOGS")]
    pub silent: bool,

    #[arg(long)]
    #[arg(conflicts_with = "block_time")]
    #[arg(help = "Disable auto and interval mining, and mine on demand instead via an endpoint.")]
    #[arg(env = "KATANA_NO_MINING")]
    pub no_mining: bool,

    #[arg(short, long)]
    #[arg(value_name = "MILLISECONDS")]
    #[arg(help = "Block time in milliseconds for interval mining.")]
    #[arg(env = "KATANA_BLOCK_TIME")]
    pub block_time: Option<u64>,

    #[arg(long)]
    #[arg(value_name = "PATH")]
    #[arg(help = "Dump the state of chain on exit to the given file.")]
    #[arg(long_help = "Dump the state of chain on exit to the given file. If the value is a \
                       directory, the state will be written to `<PATH>/state.bin`.")]
    #[arg(env = "KATANA_DUMP_STATE")]
    pub dump_state: Option<PathBuf>,

    #[arg(long)]
    #[arg(value_name = "URL")]
    #[arg(help = "The Starknet RPC provider to fork the network from.")]
    #[arg(env = "KATANA_FORK_RPC_URL")]
    pub rpc_url: Option<Url>,

    #[arg(long)]
    #[arg(help = "Output logs in JSON format.")]
    #[arg(env = "KATANA_JSON_LOG")]
    pub json_log: bool,

    #[arg(long)]
    #[arg(requires = "rpc_url")]
    #[arg(value_name = "BLOCK_NUMBER")]
    #[arg(help = "Fork the network at a specific block.")]
    #[arg(env = "KATANA_FORK_BLOCK_NUMBER")]
    pub fork_block_number: Option<u64>
}


#[derive(Debug, Args, Clone)]
pub struct SlotOptions {
    #[arg(long)]
    #[arg(help = "the url to the deployed slot katana")]
    #[arg(env = "SLOT_KATANA")]
    pub katana: Option<Url>,

    #[arg(long)]
    #[arg(help = "the url to the deployed slot torii")]
    #[arg(env = "SLOT_TORII")]
    pub torii: Option<Url>,
}

#[derive(Debug, Args, Clone)]
pub struct WorldOptions {
    #[arg(long)]
    #[arg(help = "the world address")]
    #[arg(env = "WORLD_ADDRESS")]
    pub address: Option<String>,

    #[arg(long)]
    #[arg(help = "the world salt")]
    #[arg(env = "WORLD_NAME")]
    pub name: Option<String>,
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
    #[arg(default_value = "manifests")]
    #[arg(help = "Path to the manifests directory")]
    #[arg(env = "MANIFEST_DIRECTORY_PATH")]
    pub manifest_directory_path: String,

    #[arg(long)]
    #[arg(default_value = "false")]
    #[arg(help = "Builds the dashboard if set to true")]
    #[arg(env = "PROD")]
    pub prod: bool,
}

#[derive(Debug, Args, Clone)]
pub struct StarknetOptions {
    #[arg(long)]
    #[arg(default_value = "0")]
    #[arg(help = "Specify the seed for randomness of accounts to be predeployed.")]
    #[arg(env = "SEED")]
    pub seed: String,

    #[arg(long = "accounts")]
    #[arg(value_name = "NUM")]
    #[arg(default_value = "10")]
    #[arg(help = "Number of pre-funded accounts to generate.")]
    #[arg(env = "TOTAL_ACCOUNTS")]
    pub total_accounts: u8,

    #[arg(long)]
    #[arg(help = "Disable charging fee for transactions.")]
    #[arg(env = "DISABLE_FEE")]
    pub disable_fee: bool,

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

impl KeikoArgs {

    /**
     * checks if keiko should run katana
     */
    pub fn can_run_katana(&self) -> bool {
        self.slot.katana.is_none()
    }

    /**
     * gets all katana args to run katana with
     */
    pub fn get_katana_args(&self) -> Vec<String> {
        // by default katana runs on dev mode
        let mut args = vec!["--dev".to_string()];

        if self.katana.silent {
            args.push("--silent".to_string())
        }

        if self.katana.no_mining {
            args.push("--no-mining".to_string())
        }

        if let Some(block_time) = &self.katana.block_time {
            args.push("--block-time".to_string());
            args.push(block_time.to_string());
        }

        if let Some(dump_state) = &self.katana.dump_state {
            args.push("--dump-state".to_string());
            args.push(dump_state.to_str().unwrap().to_string())
        }

        if let Some(rpc_url) = &self.katana.rpc_url {
            args.push("--rpc-url".to_string());
            args.push(rpc_url.to_string())
        }

        if self.katana.json_log {
            args.push("--json-log".to_string());
        }

        if let Some(fork_block_number) = &self.katana.fork_block_number {
            args.push("--fork-block-number".to_string());
            args.push(fork_block_number.to_string())
        }

        args.push("--seed".to_string());
        args.push(self.starknet.seed.clone());

        args.push("--accounts".to_string());
        args.push(self.starknet.total_accounts.to_string());

        if self.starknet.disable_fee {
            args.push("--disable-fee".to_string())
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

        args

    }

    /**
     * checks if keiko should run torii
     */
    pub fn can_run_torii(&self) -> bool {
        match self.slot.torii {
            None => self.can_run_katana() || self.world.address.is_some(),
            Some(_) => false
        }
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
        self.slot.katana.clone().unwrap_or(Url::parse(LOCAL_KATANA).unwrap())
    }

    /*
    *    creates the torii_url
    */
    pub fn torii_url(&self) -> Url {
        self.slot.torii.clone().unwrap_or(Url::parse(LOCAL_TORII).unwrap())
    }

    /*
    *    gets the server state
    */
    pub fn server_state(&self) -> server_state::ServerState {
        server_state::ServerState {
            json_rpc_client: self.json_rpc_client(),
            rpc_url: self.rpc_url(),
            manifest_directory_path: self.server.manifest_directory_path.clone(),
            torii_url: self.torii_url(),
            starknet: server_state::StarknetOptions {
                seed: self.starknet.seed.clone(),
                total_accounts: self.starknet.total_accounts.clone(),
            },
        }
    }

}
