use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use dojo_world::manifest::Manifest;
use tokio::time::sleep;
use keiko_api::handlers::katana::account::get_accounts;
use crate::args::KeikoArgs;

fn is_port_open(port: u16) -> bool {
    if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
        // The port is available, so close the listener and return true
        drop(listener);
        true
    } else {
        // The port is still occupied
        false
    }
}

pub async fn run_torii(config: KeikoArgs) {
    let rpc_url = config.rpc_url();
    let world_address = match config.can_run_katana() {
        true => {
            // wait till port is accessible
            while is_port_open(rpc_url.port().unwrap()) {
                sleep(Duration::from_secs(1)).await;
            }

            let accounts = get_accounts(&config.json_rpc_client()).await;
            let account = accounts.first().unwrap();

            let mut scarb_toml_path_as_str = config.server.contract_path.clone();
            let scarb_toml_path_as_str = scarb_toml_path_as_str.join("Scarb.toml");
            let scarb_toml_path_as_str = scarb_toml_path_as_str.to_str().unwrap();

            // deploy world
            let mut sozo_args = vec![
                "migrate",
                "--rpc-url",
                rpc_url.as_str(),
                "--manifest-path",
                scarb_toml_path_as_str,
                "--private-key",
                &account.private_key,
                "--account-address",
                &account.address
            ];

            if let Some(name) = &config.world.name {
                sozo_args.extend(vec!["--name", name]);
            }

            Command::new("sozo")
                .args(sozo_args)
                .spawn()
                .unwrap();

            // TODO: run post deploy script

            let manifest = Manifest::load_from_path(config.server.contract_path.join("target/dev/manifest.json")).unwrap();
            manifest.world.address.unwrap().to_string()
        }
        false => config.world.address.clone().unwrap()
    };

    Command::new("torii")
        .args(["--world", &world_address, "--rpc", rpc_url.as_str(), "--database", "sqlite:///{}/indexer.db"])
        .spawn()
        .expect("Failed to start process");
}