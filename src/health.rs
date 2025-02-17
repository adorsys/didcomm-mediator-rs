// src/health.rs
use axum::{response::IntoResponse, Json};
use eyre::Result;
use hyper::StatusCode;
use mongodb::{options::ClientOptions, Client};
use serde_json::json;
use tracing;

pub async fn health_check() -> impl IntoResponse {
    let mongo_url =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    match check_mongo_connection(&mongo_url).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "mongo": "connected" })),
        ),
        Err(err) => {
            tracing::error!("Health check failed: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "error", "error": err.to_string() })),
            )
        }
    }
}

async fn check_mongo_connection(mongo_url: &str) -> Result<()> {
    let client_options = ClientOptions::parse(mongo_url).await?;
    let client = Client::with_options(client_options)?;

    let databases = client.list_database_names(None, None).await?;
    tracing::debug!("Connected to MongoDB. Databases: {:?}", databases);

    Ok(())
}