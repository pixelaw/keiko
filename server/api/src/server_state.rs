use jsonrpsee_http_client::HttpClient;
use url::Url;

#[derive(Clone)]
pub struct ServerState {
    pub json_rpc_client: HttpClient,
    pub rpc_url: Url,
    pub manifest_directory_path: String,
    pub torii_url: Url,
    pub starknet: StarknetOptions
}

#[derive(Clone)]
pub struct StarknetOptions {
    pub seed: String,
    pub total_accounts: u8
}