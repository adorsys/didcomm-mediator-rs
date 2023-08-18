use axum::routing::get;
use axum::{response::Json, Router};
use hyper::StatusCode;
use serde_json::{json, Value};

use chrono::{DateTime, Utc};
use std::time::SystemTime;

use crate::DIDDOC_DIR;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/.well-known/did.json", get(diddoc))
}

pub async fn index() -> Json<Value> {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let now: DateTime<Utc> = SystemTime::now().into();

    Json(json!({
        "app": crate_name,
        "clk": now.to_rfc3339(),
    }))
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

#[cfg(test)]
mod tests {
    use crate::app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn index() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
        assert_eq!(body.get("app").unwrap(), &crate_name);
    }
}
