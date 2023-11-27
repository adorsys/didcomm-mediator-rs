use super::models::retrieve_or_generate_oob_inv;
use super::models::retrieve_or_generate_qr_image;
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
use did_endpoint::util::filesystem::StdFileSystem;

pub fn routes() -> Router {
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
        .route("/", get(handler_landing_page_oob))
}

async fn handler_oob_inv() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) = get_environment_variables();
    let mut fs = StdFileSystem;
    retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    )
}

async fn handler_oob_qr() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) = get_environment_variables();
    let mut fs = StdFileSystem;

    let oob_inv = retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    );
    let image_data = retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv);

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
        &image_data
    ))
}

async fn handler_landing_page_oob() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) = get_environment_variables();
    let mut fs = StdFileSystem;
    let oob_inv = retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    );
    let image_data = retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv);

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>didcomm-mediator-rs</title>
        </head>
        <div>
            <body>
                    <div style="text-align:center">
                        <h1>&nbsp;didcomm-mediator-rs</h1>
                    </div>
                    <div style="text-align:center">
                        <p>IDComm v2 mediator</p></p><br />
                    </div>
                    <div style="text-align:center">
                        <h3>Scan the QR code invitation shown below to start a mediation request:</h3>
                        <img src="data:image/png;base64,{}" alt="QR Code Image">
                        <h3>Or just copy and paste the following Out of Band invitation URL:</h3>
                        <iframe src="/oob_url" title="OOB URL" height="200" width="500" frameBorder="0"></iframe>
                    </div>
            </body>
        </div>
    </html>
        "#,
        &image_data,
    ))
}

fn get_environment_variables() -> (String, String, String) {
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

    (server_public_domain, server_local_port, storage_dirpath)
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tempdir::TempDir;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_routes() {
        let temp_dir = TempDir::new("temp_test_dir").expect("Failed to create temp directory");
        let temp_dir_path = temp_dir.path();

        std::env::set_var("SERVER_PUBLIC_DOMAIN", "example.com");
        std::env::set_var("SERVER_LOCAL_PORT", "8080");
        std::env::set_var("STORAGE_DIRPATH", temp_dir_path);

        let app = routes();

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

        let app = routes();

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

        let app = routes();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
