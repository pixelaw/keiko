use axum::http::StatusCode;
use axum::{Extension, response::Json };
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::server_state::ServerState;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    rpc_url: String,
    torii_url: String
}

pub async fn handler(Extension(server_state): Extension<ServerState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            Config {
                rpc_url: server_state.rpc_url.into(),
                torii_url: server_state.torii_url.into()
            }
        )
    )
}




