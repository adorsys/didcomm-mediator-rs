pub(crate) mod handler;
use crate::{
    plugin::OOBMessagesState,
    web::handler::{handler_landing_page_oob, handler_oob_inv, handler_oob_qr},
};
use axum::{routing::get, Router};
use handler::decode_oob_inv;
use std::sync::Arc;

pub(crate) fn routes(state: Arc<OOBMessagesState>) -> Router {
    let invitation_path = format!("/_{}", &state.oobmessage);
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
        .route("/", get(handler_landing_page_oob))
        // handle oob invitation to invitation message
        .route(&invitation_path, get(decode_oob_inv))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use crate::models::retrieve_or_generate_oob_inv;

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use filesystem::MockFileSystem;
    use std::sync::Mutex;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_routes() {
        std::env::set_var("STORAGE_DIRPATH", "tmp");
        std::env::set_var("SERVER_PUBLIC_DOMAIN", "example.com");
        std::env::set_var("SERVER_LOCAL_PORT", "8080");
        let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN").unwrap();
        let storage_dirpath = std::env::var("STORAGE_DIRPATH").unwrap();
        let mut fs = MockFileSystem;
        let oobmessage =
            retrieve_or_generate_oob_inv(&mut fs, &server_public_domain, &storage_dirpath).unwrap();
        let state = Arc::new(OOBMessagesState {
            filesystem: Arc::new(Mutex::new(fs)),
            oobmessage,
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
