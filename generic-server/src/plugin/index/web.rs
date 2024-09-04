use axum::{response::Json, routing::get, Router};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::time::SystemTime;

use crate::util::crate_name;

pub fn routes() -> Router {
    Router::new() //
        .route("/about", get(index))
}

pub async fn index() -> Json<Value> {
    let now: DateTime<Utc> = SystemTime::now().into();

    Json(json!({
        "app": crate_name(),
        "clk": now.to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::crate_name;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn index() {
        let app = routes();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/about")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.get("app").unwrap(), &crate_name());
    }
}
