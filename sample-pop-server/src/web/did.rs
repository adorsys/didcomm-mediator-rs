use std::collections::HashMap;

use axum::extract::Query;
use axum::routing::get;
use axum::{response::Json, Router};
use hyper::StatusCode;
use serde_json::{json, Value};

use crate::DIDDOC_DIR;

pub fn routes() -> Router {
    Router::new() //
        .route("/.well-known/did.json", get(diddoc))
        .route("/did/pop", get(didpop))
}

pub async fn diddoc() -> Result<Json<Value>, StatusCode> {
    match tokio::fs::read_to_string(DIDDOC_DIR.to_owned() + "/did.json").await {
        Ok(content) => Ok(Json(serde_json::from_str(&content).unwrap())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn didpop(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let challenge = params.get("challenge").ok_or(StatusCode::BAD_REQUEST)?;

    Ok(Json(json!({
        "challenge": challenge
    })))
}
