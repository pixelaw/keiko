use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::response::IntoResponse;
use dojo_world::manifest::Manifest;
use crate::server_state::ServerState;
use std::fs;
use std::io::ErrorKind;

pub async fn store_manifest(Path(app_name): Path<String>, Extension(server_state): Extension<ServerState>, Json(manifest): Json<Manifest>) -> impl IntoResponse {
    let path = &server_state.manifest_base_dir;
    if !std::path::Path::new(path).exists() {
        if let Err(_) = fs::create_dir_all(path) {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Server Error");
        }
    }
    let path = format!("{}/{}.json", &server_state.manifest_base_dir, app_name);
    match fs::metadata(&path) {
        Ok(_) => (StatusCode::IM_USED, "Already uploaded"),
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            let json = serde_json::to_string(&manifest).unwrap();
            match fs::write(path, json) {
                Ok(_) => (StatusCode::CREATED, "Stored manifest"),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server error")
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server error"),
    }
}

pub async fn get_manifest(Path(app_name): Path<String>, Extension(server_state): Extension<ServerState>) -> impl IntoResponse {
    let path = format!("{}/{}.json", &server_state.manifest_base_dir, app_name);
    println!("{}", path);
    match fs::read_to_string(&path) {
        Ok(content) => (StatusCode::OK, content),
        Err(ref e) if e.kind() == ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server error".into()),
    }
}
