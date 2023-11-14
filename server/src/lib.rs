use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use dojo_world::manifest::Manifest;
use log::{error, info};
use tokio::sync::RwLock;
use thiserror::Error;
use serde::Deserialize;

// Define a struct to represent the JSON data structure
#[derive(Deserialize, Debug)]
struct Contract {
    name: String,
    address: String,
}

#[derive(Deserialize, Debug)]
struct Data {
    contracts: Vec<Contract>,
}

#[derive(Error, Debug)]
pub enum KeikoError {
    #[error("sozo migrate failed")]
    SozoMigrateFailed,

    #[error("world address not found")]
    WorldAddressNotFound
}

// Implement From for your error
impl From<std::io::Error> for KeikoError {
    fn from(_: std::io::Error) -> KeikoError {
        KeikoError::WorldAddressNotFound
    }
}

// Implement From for your error
impl From<serde_json::Error> for KeikoError {
    fn from(_: serde_json::Error) -> KeikoError {
        KeikoError::WorldAddressNotFound
    }
}

pub struct CommandManager {
    command: RwLock<Command>,
    args: RwLock<Option<String>>,
    handler: RwLock<Option<Child>>
}

impl CommandManager {
    pub fn new(command: &str, args: Option<String>) -> Self {
        Self {
            command: RwLock::new(Command::new(command)),
            args: RwLock::new(args),
            handler: RwLock::new(None)
        }
    }

    pub async fn start_command(&self) {
        let mut command = self.command.write().await;
        if let Some(args) = self.args.read().await.clone() {
            let args = args.split(" ");
            command.args(args);
        };

        let mut handler = self.handler.write().await;
        let spawned_command = command.spawn();
        match spawned_command {
            Ok(spawned_command) => *handler = Some(spawned_command),
            Err(e) => {
                error!("Could not spawn command {}", e.to_string())
            }
        }


    }

    pub async fn reset_command(&self, new_command: Command, new_args: Option<String>) {
        let mut command = self.command.write().await;
        *command = new_command;

        let mut args = self.args.write().await;
        *args = new_args;

        let mut handler = self.handler.write().await;
        *handler = None;
    }

    pub async fn stop_command(&self) {
        let mut handler = self.handler.write().await;
        let kill_handler = handler.take();
        if let Some(mut kill_handler) = kill_handler {
            kill_handler.kill().expect("killed handler");
        }
    }
}

pub fn get_env(key: &str, default: &str) -> String {
    // Check .env file
    match dotenvy::var(key) {
        Ok(v) => return v,
        Err(_) => {
            // Check linux env
            match env::var(key) {
                Ok(v) => return v,
                Err(_) => {
                    // def
                    info!("{key} not provided, defaulting to {default}");
                    default.to_string()
                }
            }
        }
    }
}

pub fn is_port_open(port: u16) -> bool {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
        // The port is available, so close the listener and return true
        drop(listener);
        true
    } else {
        // The port is still occupied
        false
    }
}

pub fn run_sozo(
    katana_port: &String,
    manifest_json: &String,
    manifest_path: &String,
    private_key: &String,
    account_address: &String
) -> Result<String, KeikoError> {

    match Command::new("sozo")
        .args([
            "migrate",
            "--rpc-url",
            &format!("http://localhost:{}", katana_port),
            "--manifest-path",
            manifest_path,
            "--private-key",
            private_key,
            "--account-address",
            account_address
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output() {
        Ok(_) => {

            let manifest = Manifest::load_from_path(manifest_json);
            match manifest {
                Ok(manifest) => {
                    match manifest.world.address {
                        Some(address) => Ok(address.to_string()),
                        None => Err(KeikoError::WorldAddressNotFound)
                    }
                }
                Err(error) => {
                    println!("{}", error.to_string());
                    Err(KeikoError::SozoMigrateFailed)
                }
            }
        }
        Err(_) => Err(KeikoError::SozoMigrateFailed)
    }
}

pub fn extract_contract_args(manifest_json_path: &str) -> serde_json::Result<Vec<String>> {
    // Open the file
    let file = File::open(manifest_json_path).expect("could not open file");

    // Deserialize the JSON content
    let data: Data = serde_json::from_reader(file)?;

    // Transform the contracts into the desired format
    let transformed: Vec<String> = data.contracts
        .into_iter()
        .map(|contract| format!("{}={}", contract.name, contract.address))
        .collect();

    Ok(transformed)
}