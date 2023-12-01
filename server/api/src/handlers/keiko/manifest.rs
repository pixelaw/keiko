use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use dojo_world::manifest::Manifest;
use crate::server_state::ServerState;

pub async fn store_manifest(Path(app_name): Path<String>, Extension(server_state): Extension<ServerState>, Json(manifest): Json<Manifest>, ) -> impl IntoResponse {
    let mut store = server_state.store.lock().await;
    let value = store.get(&app_name);
    match value {
        None => {
            store.insert(app_name, manifest);
            (StatusCode::CREATED, "Stored manifest")
        }
        Some(_) => {
            (StatusCode::IM_USED, "Already uploaded")
        }
    }

}

pub async fn get_manifest(Path(app_name): Path<String>, Extension(server_state): Extension<ServerState>) -> impl IntoResponse {
    let store = server_state.store.lock().await;
    match store.get(&app_name) {
        Some(manifest) => (StatusCode::OK, serde_json::to_string(manifest).unwrap()),
        None => (StatusCode::NOT_FOUND, "Not Found".into()),
    }
}




