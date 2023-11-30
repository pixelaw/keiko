use std::collections::HashMap;
use std::net::SocketAddr;
use crate::args::KeikoArgs;
use clap::{Parser};
use dojo_world::manifest::Manifest;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;
use std::process::Command;
use std::sync::Arc;
use axum::http::Method;
use axum::Router;
use axum::routing::{get, get_service, MethodFilter, on};
use keiko_api::server_state::ServerState;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::{Any, CorsLayer};
use keiko_api::handlers::{katana, keiko};
use crate::utils::run_torii;

mod args;
mod utils;

const KEIKO_ASSETS: &str = "static/keiko/assets";
const KEIKO_INDEX: &str = "static/keiko/index.html";

#[tokio::main]
async fn main() {
    let config = KeikoArgs::parse();
    let store = Arc::new(tokio::sync::Mutex::new(HashMap::<String, Manifest>::new()));

    if config.server.prod {
        Command::new("npx")
            .args(vec!["import-meta-env", "-x", ".env.example", "-p", KEIKO_INDEX])
            .output()
            .expect("Failed to build dashboard");
    }

    let katana = match config.can_run_katana() {
        true => {
            Some(task::spawn(async move {
                Command::new("katana")
                    .arg("--dev")
                    .spawn()
                    .expect("Failed to start process");
            }))
        }
        false => None
    };

    let torii = match config.can_run_torii() {
        true =>Some(task::spawn(run_torii(config.clone()))),
        false => None
    };

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(Any)
        .allow_headers(Any);

    let mut router = Router::new();

    if config.can_run_katana() {
        router = router
            .route(
                "/api/state",
                get(katana::state::save_state)
                    .on(MethodFilter::PUT, katana::state::load_state)
                    .on(MethodFilter::DELETE, katana::state::reset_state),
            )
            .route("/api/accounts", get(katana::account::handler))
            .route("/api/fund", get(katana::funds::handler))
            .route("/api/block", on(MethodFilter::POST, katana::block::handler))
    }


    router = router
        .route(
            "/manifests/:app_name",
               get(keiko::manifest::get_manifest)
                   .on(MethodFilter::POST, keiko::manifest::store_manifest)
        )
        .route("/config", get(keiko::config::handler))
        .nest_service("/keiko/assets", get_service(ServeDir::new(KEIKO_ASSETS)))
        .nest_service("/keiko", get_service(ServeFile::new(KEIKO_INDEX)))
        .nest_service("/assets", get_service(ServeDir::new(config.server.static_path.join("assets"))))
        .fallback_service(get_service(ServeFile::new(config.server.static_path.join("index.html"))))
        .layer(cors)
        .layer(AddExtensionLayer::new(ServerState {
            json_rpc_client: config.json_rpc_client(),
            rpc_url: config.rpc_url(),
            store,
            torii_url: config.torii_url()
        }));

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

    if let Some(torii) = torii {
        torii.abort();
    }
}
