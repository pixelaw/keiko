use jsonrpsee_http_client::HttpClient;
use url::Url;

#[derive(Clone)]
pub struct ServerState {
    pub json_rpc_client: HttpClient,
    pub rpc_url: Url,
    pub manifest_base_dir: String,
    pub world_address: String,
    pub torii_url: Url,
}
