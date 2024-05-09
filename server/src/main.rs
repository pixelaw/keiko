use std::net::SocketAddr;
use crate::args::{Config};
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
use axum::body::Body;
use args::{KATANA_LOG, KEIKO_ASSETS, KEIKO_INDEX, TORII_LOG};
use std::fs;
use std::path::Path;


mod args;
mod utils;

#[tokio::main]
async fn main() {
    let config = Config::new();

    if config.server.prod {
        Command::new("npx")
            .args(vec!["import-meta-env", "-x", ".env.example", "-p", KEIKO_INDEX])
            .output()
            .expect("Failed to build dashboard");
    }


    let world_storage_dir = format!("storage/{}/", config.katana.world_address);

    if !Path::new(&world_storage_dir).exists() || fs::read_dir(&world_storage_dir).unwrap().next().is_none() {
        fs::create_dir_all("storage").unwrap();
        Command::new("cp")
            .arg("-r")
            .arg("storage_init/")
            .arg(&world_storage_dir)
            .status()
            .unwrap();
    }

    let katana = start_katana(&config).await;

    let torii = start_torii(&config).await;

    // TODO Modify the Scarb.toml if needed with world address

    // TODO Deploy Dojo/contracts if needed


    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port.clone()));

    let router = create_router(&config);

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


fn create_router(config: &Config) -> Router<(), Body> {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers(Any);


    Router::new()
        .route("/api/fund", get(katana::funds::handler))
        .route("/api/block", on(MethodFilter::POST, katana::block::handler))
        // .route("/api/accounts", get(katana::account::handler))
        .route("/config", get(keiko::config::handler))
        .nest_service("/keiko/assets", get_service(ServeDir::new(KEIKO_ASSETS)))
        .nest_service("/keiko", get_service(ServeFile::new(KEIKO_INDEX)))
        .nest_service("/assets", get_service(ServeDir::new(config.server.static_path.join("assets"))))
        .fallback_service(get_service(ServeFile::new(config.server.static_path.join("index.html"))))
        .layer(cors)
        .layer(AddExtensionLayer::new(config.server_state()))
}

async fn start_katana(config: &Config) -> task::JoinHandle<()> {
    let katana_args = config.get_katana_args();

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

async fn start_torii(config: &Config) -> task::JoinHandle<()> {
    let args = config.get_torii_args();

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