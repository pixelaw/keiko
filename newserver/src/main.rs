use std::sync::Arc;

use clap::{Parser};
use katana_core::sequencer::KatanaSequencer;
use katana_rpc::{NodeHandle, spawn};
use tokio::signal::ctrl_c;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use std::net::SocketAddr;
use std::str::FromStr;

use common::parse::{parse_socket_address, parse_url};
use dojo_world::contracts::world::WorldContractReader;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;


use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;

use torii_core::engine::{Engine, EngineConfig, Processors};
use torii_core::processors::metadata_update::MetadataUpdateProcessor;
use torii_core::processors::register_model::RegisterModelProcessor;
use torii_core::processors::store_del_record::StoreDelRecordProcessor;
use torii_core::processors::store_set_record::StoreSetRecordProcessor;
use torii_core::processors::store_transaction::StoreTransactionProcessor;
use torii_core::simple_broker::SimpleBroker;
use torii_core::sql::Sql;
use torii_core::types::Model;
use torii_server::proxy::Proxy;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};
use url::Url;


use crate::katana_args::KatanaArgs;
use crate::torii_args::ToriiArgs;

mod katana_args;
mod torii_args;
mod utils;

async fn start_katana() -> Result<NodeHandle, Box<dyn std::error::Error>> {
    let args = KatanaArgs::parse();

    args.init_logging()?;

    let server_config = args.server_config();
    let sequencer_config = args.sequencer_config();
    let starknet_config = args.starknet_config();

    let sequencer = Arc::new(KatanaSequencer::new(sequencer_config, starknet_config).await?);
    let node_handle = spawn(Arc::clone(&sequencer), server_config).await?;

    if !args.silent {
        println!("Katana ready");
    }

    Ok(node_handle)
}

async fn start_torii() -> anyhow::Result<()> {
    let args = ToriiArgs::parse();

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,hyper_reverse_proxy=off"));

    let subscriber = fmt::Subscriber::builder().with_env_filter(filter_layer).finish();

    // Set the global subscriber
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set the global tracing subscriber");

    // Setup cancellation for graceful shutdown
    let (shutdown_tx, _) = broadcast::channel(1);

    let shutdown_tx_clone = shutdown_tx.clone();
    ctrlc::set_handler(move || {
        let _ = shutdown_tx_clone.send(());
    })
        .expect("Error setting Ctrl-C handler");

    let database_url = format!("sqlite:{}", &args.database);
    let options =
        SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true).with_regexp();
    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .connect_with(options)
        .await?;

    // TODO need the migrations from github here
    sqlx::migrate!("./src/torii/migrations_20240214").run(&pool).await?;

    let provider: Arc<_> = JsonRpcClient::new(HttpTransport::new(args.rpc)).into();

    // Get world address
    let world = WorldContractReader::new(args.world_address, &provider);

    let mut db = Sql::new(pool.clone(), args.world_address).await?;
    let processors = Processors {
        event: vec![
            Box::new(RegisterModelProcessor),
            Box::new(StoreSetRecordProcessor),
            Box::new(MetadataUpdateProcessor),
            Box::new(StoreDelRecordProcessor),
        ],
        transaction: vec![Box::new(StoreTransactionProcessor)],
        ..Processors::default()
    };

    let (block_tx, block_rx) = tokio::sync::mpsc::channel(100);

    let mut engine = Engine::new(
        world,
        &mut db,
        &provider,
        processors,
        EngineConfig { start_block: args.start_block, ..Default::default() },
        shutdown_tx.clone(),
        Some(block_tx),
    );

    let shutdown_rx = shutdown_tx.subscribe();
    let (grpc_addr, grpc_server) = torii_grpc::server::new(
        shutdown_rx,
        &pool,
        block_rx,
        args.world_address,
        Arc::clone(&provider),
    )
        .await?;

    let proxy_server = Arc::new(Proxy::new(args.addr, args.allowed_origins, Some(grpc_addr), None));

    let graphql_server = spawn_rebuilding_graphql_server(
        shutdown_tx.clone(),
        pool.into(),
        args.external_url,
        proxy_server.clone(),
    );

    let mut libp2p_relay_server = torii_relay::server::Relay::new(
        args.relay_port,
        args.relay_webrtc_port,
        args.relay_local_key_path,
        args.relay_cert_path,
    )
        .expect("Failed to start libp2p relay server");

    info!(target: "torii::cli", "Starting torii endpoint: {}", format!("http://{}", args.addr));
    info!(target: "torii::cli", "Serving Graphql playground: {}\n", format!("http://{}/graphql", args.addr));


    tokio::select! {
        _ = engine.start() => {},
        _ = proxy_server.start(shutdown_tx.subscribe()) => {},
        _ = graphql_server => {},
        _ = grpc_server => {},
        _ = libp2p_relay_server.run() => {},
    }
    ;

    Ok(())
}

async fn spawn_rebuilding_graphql_server(
    shutdown_tx: Sender<()>,
    pool: Arc<SqlitePool>,
    external_url: Option<Url>,
    proxy_server: Arc<Proxy>,
) {
    let mut broker = SimpleBroker::<Model>::subscribe();

    loop {
        let shutdown_rx = shutdown_tx.subscribe();
        let (new_addr, new_server) =
            torii_graphql::server::new(shutdown_rx, &pool, external_url.clone()).await;

        tokio::spawn(new_server);

        proxy_server.set_graphql_addr(new_addr).await;

        // Break the loop if there are no more events
        if broker.next().await.is_none() {
            break;
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let NodeHandle { addr: _, handle: katana_handle, .. } = start_katana().await?;

    // Wait until Ctrl + C is pressed, then shutdown
    ctrl_c().await?;
    katana_handle.stop()?;

    Ok(())
}
