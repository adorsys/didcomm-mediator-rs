use axum::{response::Json, routing::get, Router};
use hyper::StatusCode;
use serde_json::Value;

use crate::DIDDOC_DIR;

pub fn routes() -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
    // .route("/.well-known/did/pop.json", get(didpop))
}

pub async fn diddoc() -> Result<Json<Value>, StatusCode> {
    match tokio::fs::read_to_string(DIDDOC_DIR.to_owned() + "/did.json").await {
        Ok(content) => Ok(Json(serde_json::from_str(&content).unwrap())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
