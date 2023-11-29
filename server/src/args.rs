use std::path::PathBuf;
use clap::{Args, Parser};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use url::Url;
use std::str::FromStr;

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
    #[command(next_help_heading = "Slot options")]
    pub slot: SlotOptions,

    #[command(flatten)]
    #[command(next_help_heading = "World options")]
    pub world: WorldOptions,
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
    #[arg(short, long)]
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

    #[arg(short, long)]
    #[arg(default_value = "static")]
    #[arg(value_parser = PathBuf::from_str)]
    #[arg(help = "Path to the static directory")]
    #[arg(env = "STATIC_PATH")]
    pub static_path: PathBuf
}

impl KeikoArgs {

    /**
     * checks if keiko should run katana
     */
    pub fn can_run_katana(&self) -> bool {
        self.slot.katana.is_none()
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

}
