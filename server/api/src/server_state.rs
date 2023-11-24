use dojo_world::manifest::Manifest;
use jsonrpsee_http_client::HttpClient;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub json_rpc_client: HttpClient,
    pub store: Arc<tokio::sync::Mutex<HashMap<String, Manifest>>>
}