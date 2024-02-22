use std::env::current_dir;
use std::net::SocketAddr;
use crate::args::KeikoArgs;
use clap::{Parser};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;
use axum::http::Method;
use axum::Router;
use axum::routing::{get, get_service, MethodFilter, on};
use tower_http::add_extension::AddExtensionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{Any, CorsLayer};
use keiko_api::handlers::{katana, keiko};
// use crate::utils::run_torii;
use std::process::{Command, Stdio};
use std::fs::File;
use serde_json::Value;

mod args;
mod utils;

const KEIKO_ASSETS: &str = "static/keiko/assets";
const KEIKO_INDEX: &str = "static/keiko/index.html";
const KATANA_LOG: &str = "katana.log.json";
const TORII_LOG: &str = "torii.log";

#[tokio::main]
async fn main() {
    let config = KeikoArgs::parse();

    if config.server.prod {
        Command::new("npx")
            .args(vec!["import-meta-env", "-x", ".env.example", "-p", KEIKO_INDEX])
            .output()
            .expect("Failed to build dashboard");
    }

    // Start Katana if needed
    let katana = match config.should_run_katana() {
        true => {
            let katana_args = config.get_katana_args();
            let result = Some(task::spawn(async move {
                let output = File::create(KATANA_LOG).expect("Failed to create file");

                Command::new("katana")
                    .args(katana_args)
                    .stdout(Stdio::from(output))
                    .spawn()
                    .expect("Failed to start process");
            }));

            wait_for_non_empty_file(KATANA_LOG).await;
            result
        }
        false => None
    };

    let mut world_address = String::from("");
    let rpc_url = "http://localhost:5050";

    // Check if we're starting with a Dojo Genesis
    if let (Some(genesis), Some(manifest)) = (&config.keiko.genesis, &config.keiko.manifest) {
        // It looks like the Genesis has deployed contracts already
        let file = File::open(manifest.clone()).expect("File should open read only");
        let json: Value = serde_json::from_reader(file).expect("Manifest was not well-formatted");
        world_address = json["world"]["address"].as_str().unwrap().to_string();
        println!("World address: {}", world_address);

        // TODO do we need the world owner address here?
        // TODO also, what happens if the genesis has World deployments but the current scarb.toml has a different account?
    }

    // TODO Modify the Scarb.toml if needed with world address

    // TODO Deploy Dojo/contracts if needed

    // Start Torii if needed
    let torii = match config.should_run_torii() {
        true => {
            let mut args: Vec<String> = vec![];
            args.push("--world".to_string());
            args.push(world_address.to_string());

            // if let Some(genesis) = &self.keiko.genesis {
            if let Some(torii_db) = config.keiko.torii_db.clone() {
                args.push("--database".to_string());
                args.push(format!("sqlite:///{}/{}", current_dir().unwrap().display(), torii_db.clone().to_string()));
            }

            let result = Some(task::spawn(async move {
                let output = File::create(TORII_LOG).expect("Failed to create file");

                Command::new("torii")
                    .stdout(Stdio::from(output))
                    .args(args)
                    .spawn()
                    .expect("Failed to start torii");
            }));

            wait_for_non_empty_file(TORII_LOG).await;
            result
        }

        false => None
    };

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers(Any);

    let mut router = Router::new();

    if config.should_run_katana() {
        router = router
            .route(
                "/api/state",
                get(katana::state::save_state)
                    .on(MethodFilter::PUT, katana::state::load_state)
                    .on(MethodFilter::DELETE, katana::state::reset_state),
            )
            .route("/api/fund", get(katana::funds::handler))
            .route("/api/block", on(MethodFilter::POST, katana::block::handler))
    }


    router = router
        // .route("/api/accounts", get(katana::account::handler))
        .route(
            "/manifests/:app_name",
            get(keiko::manifest::get_manifest)
                .on(MethodFilter::POST, keiko::manifest::store_manifest),
        )
        .route("/config", get(keiko::config::handler))
        .nest_service("/keiko/assets", get_service(ServeDir::new(KEIKO_ASSETS)))
        .nest_service("/keiko", get_service(ServeFile::new(KEIKO_INDEX)))
        .nest_service("/assets", get_service(ServeDir::new(config.server.static_path.join("assets"))))
        .fallback_service(get_service(ServeFile::new(config.server.static_path.join("index.html"))))
        .layer(cors)
        .layer(AddExtensionLayer::new(config.server_state()));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>());

    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = server => {
            // This arm will run if the server shuts down by itself.
            println!("Stopping server...");

        }
        _ = sigterm.recv() => {
            // This arm will run if a SIGINT signal is received.
            println!("sigterm received, stopping server...");
        }
    }


    if let Some(katana) = katana {
        katana.abort();
    }

    // if let Some(torii) = torii {
    //     torii.abort();
    // }
}

async fn wait_for_non_empty_file(path: &str) {
    loop {
        let contents = tokio::fs::read_to_string(path).await.expect("Failed to read file");
        if !contents.is_empty() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Sleep for a second
    }
}
