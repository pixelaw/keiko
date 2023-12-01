use dojo_world::manifest::Manifest;
use jsonrpsee_http_client::HttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

#[derive(Clone)]
pub struct ServerState {
    pub json_rpc_client: HttpClient,
    pub rpc_url: Url,
    pub store: Arc<tokio::sync::Mutex<HashMap<String, Manifest>>>,
    pub torii_url: Url,
    pub starknet: StarknetOptions
}

#[derive(Clone)]
pub struct StarknetOptions {
    pub seed: String,
    pub total_accounts: u8
}