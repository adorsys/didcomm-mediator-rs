use axum::{routing::get, Router, response::Html};
use axum::response::IntoResponse;
use super::models::retrieve_or_generate_oob_inv;
use super::models::retrieve_or_generate_qr_image;

pub fn routes() -> Router {
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
}

async fn handler_oob_inv() -> impl IntoResponse {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")
    .map_err(|_| {
        tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
    })
    .expect("Failed to get SERVER_PUBLIC_DOMAIN from env");

    let server_local_port = std::env::var("SERVER_LOCAL_PORT")
    .map_err(|_| {
        tracing::error!("SERVER_LOCAL_PORT env variable required");
    })
    .expect("Failed to get SERVER_LOCAL_PORT from env");

    let storage_dirpath = std::env::var("STORAGE_DIRPATH")
    .map_err(|_| {
        tracing::error!("STORAGE_DIRPATH env variable required");
    })
    .expect("Failed to get STORAGE_DIRPATH from env");

    retrieve_or_generate_oob_inv(&server_public_domain, &server_local_port, &storage_dirpath)
}

async fn handler_oob_qr() -> impl IntoResponse {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")
    .map_err(|_| {
        tracing::error!("SERVER_PUBLIC_DOMAIN env variable required");
    })
    .expect("Failed to get SERVER_PUBLIC_DOMAIN from env");

    let server_local_port = std::env::var("SERVER_LOCAL_PORT")
    .map_err(|_| {
        tracing::error!("SERVER_LOCAL_PORT env variable required");
    })
    .expect("Failed to get SERVER_LOCAL_PORT from env");

    let storage_dirpath = std::env::var("STORAGE_DIRPATH")
    .map_err(|_| {
        tracing::error!("STORAGE_DIRPATH env variable required");
    })
    .expect("Failed to get STORAGE_DIRPATH from env");

    let oob_inv = retrieve_or_generate_oob_inv(&server_public_domain, &server_local_port, &storage_dirpath);

    let image_data = retrieve_or_generate_qr_image(&storage_dirpath, &oob_inv);

     Html(format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>QR Code Image</title>
                </head>
                <body>
                    <img src="data:image/png;base64,{}" alt="QR Code Image">
                </body>
            </html>
            "#,
            base64::encode(&image_data)
    ))
}