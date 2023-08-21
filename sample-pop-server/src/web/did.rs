use axum::routing::get;
use axum::{response::Json, Router};
use hyper::StatusCode;
use serde_json::Value;

use crate::DIDDOC_DIR;

pub fn routes() -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
}

pub async fn diddoc() -> (StatusCode, Json<Value>) {
    match tokio::fs::read_to_string(DIDDOC_DIR.to_owned() + "/did.json").await {
        Ok(content) => (
            StatusCode::OK,
            Json(serde_json::from_str(&content).unwrap()),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json(Value::Null)),
    }
}
