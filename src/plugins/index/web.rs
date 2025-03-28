use axum::{response::Json, routing::get, Router};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::time::SystemTime;

use super::util::crate_name;

pub(crate) fn routes() -> Router {
    Router::new() //
        .route("/about", get(index))
}

pub(crate) async fn index() -> Json<Value> {
    let now: DateTime<Utc> = SystemTime::now().into();

    Json(json!({
        "app": crate_name(),
        "clk": now.to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
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

        let body = BodyExt::collect(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body.to_bytes()).unwrap();

        assert_eq!(body.get("app").unwrap(), &crate_name());
    }
}
