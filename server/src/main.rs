use std::env::current_dir;
use std::net::SocketAddr;
use crate::args::{Config};
use clap::Parser;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;
use axum::http::Method;
use axum::Router;
use axum::routing::{get, get_service, MethodFilter, on};
use tower_http::add_extension::AddExtensionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{Any, CorsLayer};
use keiko_api::handlers::{katana, keiko};
use std::process::{Command, Stdio};
use std::fs::File;
use serde_json::Value;

mod args;
mod utils;

const KEIKO_ASSETS: &str = "static/keiko/assets";
const KEIKO_INDEX: &str = "static/keiko/index.html";
const KATANA_LOG: &str = "log/katana.log.json";
const TORII_LOG: &str = "log/torii.log";
const CONFIG_MANIFEST: &str = "config/manifest.json";

#[tokio::main]
async fn main() {
    let mut config = Config::new();

    if config.server.prod {
        Command::new("npx")
            .args(vec!["import-meta-env", "-x", ".env.example", "-p", KEIKO_INDEX])
            .output()
            .expect("Failed to build dashboard");
    }

    let katana = start_katana(config.get_katana_args()).await;

    // Get the world address from the manifest
    let manifest_json: Value = serde_json::from_reader(
        File::open(CONFIG_MANIFEST).expect("File should open read only")
    ).expect("Cannot parse config/manifest.json");

    let world_address = manifest_json["world"]["address"].as_str().unwrap().to_string();
    config.set_world_address(world_address.to_string());

    let rpc_url = "http://localhost:5050";
    let torii = start_torii(world_address).await;

    // TODO Modify the Scarb.toml if needed with world address

    // TODO Deploy Dojo/contracts if needed


    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers(Any);

    let mut router = Router::new();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port.clone()));

    router = router
        .route("/api/fund", get(katana::funds::handler))
        .route("/api/block", on(MethodFilter::POST, katana::block::handler))
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
        .layer(AddExtensionLayer::new(config));


    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>());

    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = server => println!("Stopping server..."),
        _ = sigterm.recv() => println!("sigterm received, stopping server...")
    }

    // Close Katana and Torii
    katana.abort();
    torii.abort();
}

async fn start_katana(katana_args: Vec<String>) -> task::JoinHandle<()> {
    let result = task::spawn(async move {
        let output = File::create(KATANA_LOG).expect("Failed to create file");

        Command::new("katana")
            .args(katana_args)
            .stdout(Stdio::from(output))
            .spawn()
            .expect("Failed to start process");
    });
    // TODO get the server/port from args
    utils::wait_for_port("127.0.0.1:5050".parse().unwrap()).await;
    result
}

async fn start_torii(world_address: String) -> task::JoinHandle<()> {
    let mut args: Vec<String> = vec![
        "--world".to_string(),
        world_address,
        "--database".to_string(),
        format!("sqlite:///{}/storage/torii.sqlite", current_dir().unwrap().display()),
    ];

    let result = task::spawn(async move {
        let output = File::create(TORII_LOG).expect("Failed to create file");

        Command::new("torii")
            .stdout(Stdio::from(output))
            .args(args)
            .spawn()
            .expect("Failed to start torii");
    });

    utils::wait_for_port("127.0.0.1:8080".parse().unwrap()).await;

    result
}