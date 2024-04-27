use serde_json::json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use dojo_world::manifest::{AbstractManifestError, DeploymentManifest};
use crate::server_state::ServerState;
use std::fs;
use std::io::ErrorKind;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;


pub async fn get_manifest(
    Path(app_name): Path<String>,
    Extension(server_state): Extension<ServerState>,
)
    -> impl IntoResponse
{
    let provider = JsonRpcClient::new(HttpTransport::new(
        server_state.rpc_url
    ));

    // let manifest = DeploymentManifest::load_from_remote(provider, FieldElement::from(server_state.world_address)).await?;
    let world_address = FieldElement::from_hex_be(server_state.world_address.as_str()).unwrap();


    match DeploymentManifest::load_from_remote(provider, world_address).await {
        Ok(manifest) => {
            let world_json = serde_json::to_value(&manifest.base).unwrap();
            (StatusCode::OK, Json(world_json))
        }

        _ => (StatusCode::NOT_FOUND, Json(serde_json::json!("Not found")))
    }

    //
    // if remote_manifest.is_none() {
    //     ui.print_sub("No remote World found");
    // }
    //
    // match fs::read_to_string(&path) {
    //     Ok(content) => (StatusCode::OK, manifest),
    //     Err(ref e) if e.kind() == ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
    //     Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server error".into()),
    // }
}
