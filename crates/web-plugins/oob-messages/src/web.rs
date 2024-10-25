use super::models::{retrieve_or_generate_oob_inv, retrieve_or_generate_qr_image};
use axum::{
    http::header,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use filesystem::StdFileSystem;
use std::error::Error;

pub(crate) fn routes() -> Router {
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
        .route("/", get(handler_landing_page_oob))
}

async fn handler_oob_inv() -> Response {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => {
                return Html(format!("Error getting environment variables: {}", err))
                    .into_response()
            }
        };

    let mut fs = StdFileSystem;

    let html_content = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)).into_response(),
    };

    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <style>
                pre {{
                    white-space: pre-wrap;
                    word-wrap: break-word;
                }}
            </style>
        <body>
        <pre>
            {}
        </pre>
        </body>
        </html>
            "#,
        html_content
    );

    Html(html_content).into_response()
}

async fn handler_oob_qr() -> Response {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => {
                return Html(format!("Error getting environment variables: {}", err))
                    .into_response()
            }
        };

    let mut fs = StdFileSystem;

    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)).into_response(),
    };

    let image_data = match retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => return Html(format!("Error generating QR code: {}", err)).into_response(),
    };

    let html_content = format!(
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
    );

    (
        [(header::CONTENT_SECURITY_POLICY, "img-src 'self' data:")],
        Html(html_content),
    )
        .into_response()
}

async fn handler_landing_page_oob() -> Response {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => {
                return Html(format!("Error getting environment variables: {}", err))
                    .into_response()
            }
        };

    let mut fs = StdFileSystem;

    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)).into_response(),
    };

    let image_data = match retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => {
            return Html(format!(
                "Error getting retrieving or generating qr image: {}",
                err
            ))
            .into_response()
        }
    };

    let html_content = format!(
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
                        <iframe src="/oob_url,{}" title="OOB URL" height="200" width="500" frameBorder="0"></iframe>
                    </div>
            </body>
        </div>
    </html>
        "#,
        &image_data,
        &oob_inv
    );

    (
        [(header::CONTENT_SECURITY_POLICY, "img-src 'self' data:")],
        Html(html_content),
    )
        .into_response()
}

fn get_environment_variables() -> Result<(String, String, String), Box<dyn Error>> {
    let server_public_domain = std::env::var("SERVER_PUBLIC_DOMAIN")
        .map_err(|_| "SERVER_PUBLIC_DOMAIN env variable required")?;

    let server_local_port = std::env::var("SERVER_LOCAL_PORT")
        .map_err(|_| "SERVER_LOCAL_PORT env variable required")?;

    let storage_dirpath =
        std::env::var("STORAGE_DIRPATH").map_err(|_| "STORAGE_DIRPATH env variable required")?;

    Ok((server_public_domain, server_local_port, storage_dirpath))
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

        std::env::set_var("SERVER_PUBLIC_DOMAIN", "http://example.com");
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
