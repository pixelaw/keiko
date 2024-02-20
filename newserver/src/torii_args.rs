use std::net::SocketAddr;

use clap::Parser;
use common::parse::{parse_socket_address, parse_url};
use katana_primitives::FieldElement;
use url::Url;

/// Dojo World Indexer
#[derive(Parser, Debug)]
#[command(name = "torii", author, version, about, long_about = None)]
pub(crate) struct ToriiArgs {
    /// The world to index
    #[arg(short, long = "world", env = "DOJO_WORLD_ADDRESS")]
    pub(crate) world_address: FieldElement,

    /// The sequencer rpc endpoint to index.
    #[arg(long, value_name = "URL", default_value = ":5050", value_parser = parse_url)]
    pub(crate) rpc: Url,

    /// Database filepath (ex: indexer.db). If specified file doesn't exist, it will be
    /// created. Defaults to in-memory database
    #[arg(short, long, default_value = ":memory:")]
    pub(crate) database: String,

    /// Specify a block to start indexing from, ignored if stored head exists
    #[arg(short, long, default_value = "0")]
    pub(crate) start_block: u64,

    /// Address to serve api endpoints at.
    #[arg(long, value_name = "SOCKET", default_value = "0.0.0.0:8080", value_parser = parse_socket_address)]
    pub(crate) addr: SocketAddr,

    /// Port to serve Libp2p TCP & UDP Quic transports
    #[arg(long, value_name = "PORT", default_value = "9090")]
    pub(crate) relay_port: u16,

    /// Port to serve Libp2p WebRTC transport
    #[arg(long, value_name = "PORT", default_value = "9091")]
    pub(crate) relay_webrtc_port: u16,

    /// Path to a local identity key file. If not specified, a new identity will be generated
    #[arg(long, value_name = "PATH")]
    pub(crate) relay_local_key_path: Option<String>,

    /// Path to a local certificate file. If not specified, a new certificate will be generated
    /// for WebRTC connections
    #[arg(long, value_name = "PATH")]
    pub(crate) relay_cert_path: Option<String>,

    /// Specify allowed origins for api endpoints (comma-separated list of allowed origins, or "*"
    /// for all)
    #[arg(long, default_value = "*")]
    #[arg(value_delimiter = ',')]
    pub(crate) allowed_origins: Vec<String>,

    /// The external url of the server, used for configuring the GraphQL Playground in a hosted
    /// environment
    #[arg(long)]
    pub(crate) external_url: Option<Url>,

    /// Enable Prometheus metrics.
    ///
    /// The metrics will be served at the given interface and port.
    #[arg(long, value_name = "SOCKET", value_parser = parse_socket_address, help_heading = "Metrics")]
    pub(crate) metrics: Option<SocketAddr>,
}
