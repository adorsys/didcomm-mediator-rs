use axum::{response::IntoResponse, Json};
use eyre::Result;
use hyper::StatusCode;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {
    let mongo_url =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://mongodb:27017".to_string());

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

    // Check if MongoDB is functional
    let db = client.database("admin");
    let ping_result = db.run_command(doc! { "ping": 1 }).await?;
    if ping_result.get_f64("ok")? != 1.0 {
        return Err(eyre::eyre!("MongoDB ping failed"));
    }

    let databases = client.list_database_names().await?;
    tracing::debug!("Connected to MongoDB. Databases: {:?}", databases);

    Ok(())
}
