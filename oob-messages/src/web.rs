use super::models::retrieve_or_generate_oob_inv;
use super::models::retrieve_or_generate_qr_image;
use axum::response::IntoResponse;
use axum::{response::Html, routing::get, Router};
use did_endpoint::util::filesystem::StdFileSystem;
use std::error::Error;

pub fn routes() -> Router {
    Router::new() //
        .route("/oob_url", get(handler_oob_inv))
        .route("/oob_qr", get(handler_oob_qr))
        .route("/", get(handler_landing_page_oob))
        .route("/test", get(test_handler_oob_qr))
}

async fn handler_oob_inv() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => return Html(format!("Error getting environment variables: {}", err)),
        };

    let mut fs = StdFileSystem;

    match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => return Html(format!("{}", oob_inv)),
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)),
    }
}

async fn handler_oob_qr() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => return Html(format!("Error getting environment variables: {}", err)),
        };

    let mut fs = StdFileSystem;

    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)),
    };

    let image_data = match retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => return Html(format!("Error generating QR code: {}", err)),
    };

    Html(image_data)
}

async fn test_handler_oob_qr() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => return Html(format!("Error getting environment variables: {}", err)),
        };

    let mut fs = StdFileSystem;

    let _oob_inv = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)),
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>Base64 Image Example</title>
        </head>
        <body>
          <img id="base64Image" alt="Base64 Image">
          <script>
            fetch('/oob_qr')
              .then(response => response.text())
              .then(base64Data => {{
                document.getElementById('base64Image').src = 'data:image/png;base64,' + base64Data;
              }})
              .catch(error => console.error('Error fetching image:', error));
          </script>
        </body>
        </html>
            "#
    ))
}

async fn handler_landing_page_oob() -> impl IntoResponse {
    let (server_public_domain, server_local_port, storage_dirpath) =
        match get_environment_variables() {
            Ok(result) => result,
            Err(err) => return Html(format!("Error getting environment variables: {}", err)),
        };

    let mut fs = StdFileSystem;

    let oob_inv = match retrieve_or_generate_oob_inv(
        &mut fs,
        &server_public_domain,
        &server_local_port,
        &storage_dirpath,
    ) {
        Ok(oob_inv) => oob_inv,
        Err(err) => return Html(format!("Error retrieving oob inv: {}", err)),
    };

    let image_data = match retrieve_or_generate_qr_image(&mut fs, &storage_dirpath, &oob_inv) {
        Ok(data) => data,
        Err(err) => {
            return Html(format!(
                "Error getting retrieving or generating qr image: {}",
                err
            ))
        }
    };

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
