pub(crate) mod handler;
use crate::{
    plugin::OOBMessagesState,
    web::handler::{handler_landing_page_oob, handler_oob_inv, handler_oob_qr},
};
use axum::{routing::get, Router};
use std::sync::Arc;

pub(crate) fn routes(state: Arc<OOBMessagesState>) -> Router {
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
        .route("/", get(handler_landing_page_oob))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryStore;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use did_utils::didcore::Document;
    use std::sync::Mutex;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_routes() {
        std::env::set_var("SERVER_PUBLIC_DOMAIN", "example.com");

        let state = Arc::new(OOBMessagesState {
            store: Arc::new(Mutex::new(InMemoryStore::default())),
            diddoc: Document::default(),
            server_public_domain: "example.com".to_string(),
        });
        let app = routes(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/oob_url")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let app = routes(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/oob_qr")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let app = routes(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
