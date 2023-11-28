use std::collections::HashMap;
use std::env;
use std::env::current_dir;
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use dojo_world::manifest::Manifest;
use run_script::run_script;
use run_script::types::{ScriptOptions, ScriptResult};
use tokio::time::sleep;
use keiko_api::handlers::katana::account::{Account, get_accounts};
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

const UPDATE_CONTRACTS: &str =
    r#"
        #!/bin/bash
        for row in $(cat target/dev/manifest.json | jq -r '.contracts[] | @base64'); do
            _jq() {
                echo ${row} | base64 --decode | jq -r ${1}
            }

            name=$(_jq '.name')
            address=$(_jq '.address')

            # Convert to uppercase and replace '-' with '_'
            var_name=$(echo $name | tr '[:lower:]' '[:upper:]' | tr '-' '_')

            declare "${var_name}"=$address
        done
    "#;

const UPDATE_SCARB: &str =
    r#"
        # Get world address from manifest
        WORLD_ADDRESS=$(cat target/dev/manifest.json | jq '.world.address')

        # Check if WORLD_ADDRESS is not "null"
        if [ "$WORLD_ADDRESS" != "null" ]; then
          # Update Scarb.toml
          sed -i "s/world_address = ".*"/world_address = "$WORLD_ADDRESS"/" Scarb.toml
        fi


        echo "Scarb.toml has been updated with address(es)"
    "#;

fn update_account(account: &Account, options: &ScriptOptions) -> ScriptResult<(i32, String, String)> {
    let script = format!(
        r#"
            sed -i "s/account_address = ".*"/account_address = "{}" Scarb.toml
            sed -i "s/private_key = ".*"/private_key = "{}" Scarb.toml
        "#,
        account.address,
        account.private_key
    );
    run_script::run_script!(script, options)
}

pub async fn run_torii(config: KeikoArgs) {
    let rpc_url = config.rpc_url();
    let world_address = match config.can_run_katana() {
        false => config.world.address.clone().unwrap(),
        true => {
            println!("deploying contracts");
            // wait till port is accessible
            while is_port_open(rpc_url.port().unwrap()) {
                sleep(Duration::from_secs(1)).await;
            }

            let accounts = get_accounts(&config.json_rpc_client()).await;
            let account = accounts.first().unwrap();

            let scarb_toml_path_as_str = config.server.contract_path.clone();
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

            // deploy contracts
            Command::new("sozo")
                .args(sozo_args)
                .output()
                .unwrap();

            // update environment variables
            println!("updating environment variables");

            let mut env: HashMap<String, String> = HashMap::new();
            let manifest = Manifest::load_from_path(config.server.contract_path.clone().join("target/dev/manifest.json")).unwrap();
            manifest.contracts.iter().for_each(|contract| {
                env.insert(contract.name.clone().to_string().to_uppercase(), contract.address.unwrap().to_string());
            });

            let mut options = ScriptOptions::new();
            options.working_directory = Some(config.server.contract_path.clone());
            options.exit_on_error = true;
            run_script::run_script!(UPDATE_SCARB, &options).unwrap();

            update_account(account, &options).unwrap();

            println!("running post deployment");

            Command::new("scarb")
                .args([
                    "--manifest-path",
                    scarb_toml_path_as_str,
                    "run",
                    "post_deploy",
                    "--"
                ])
                .args(env.iter().map(|(k, v)| format!("{}={}", k, v)))
                .envs(env)
                .spawn()
                .unwrap();


            manifest.world.address.unwrap().to_string()
        }
    };

    Command::new("torii")
        .args([
            "--world",
            &world_address,
            "--rpc",
            rpc_url.as_str(),
            "--database",
            &format!("sqlite:///{}/indexer.db", current_dir().unwrap().to_str().unwrap())
        ])
        .spawn()
        .expect("Failed to start process");
}