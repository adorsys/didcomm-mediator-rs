use didcomm_mediator::app;
use eyre::{Result, WrapErr};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use reqwest::{Client, StatusCode};
use thiserror::Error;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Request failed with status {0}")]
    RequestFailed(StatusCode),
    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),
}

pub async fn fetch_with_retries(client: &Client, url: &str) -> Result<String, FetchError> {
    let retry_strategy = ExponentialBackoff::from_millis(100).take(3);

    Retry::spawn(retry_strategy, || async {
        let response = client.get(url).send().await?;
        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(FetchError::RequestFailed(response.status()))
        }
    })
    .await
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow().ok();

    // Enable logging
    config_tracing();

    // Configure server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap_or("3000".to_owned());
    let port = port.parse().context("failed to parse port")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .context("failed to bind address")?;

    tracing::debug!("listening on {addr}");

    generic_server_with_graceful_shutdown(listener)
        .await
        .map_err(|err| {
            tracing::error!("{err:?}");
            err
        })?;

    Ok(())
}

async fn generic_server_with_graceful_shutdown(listener: TcpListener) -> Result<()> {
    // Load plugins
    let (mut plugin_container, router) = app()?;

    // Start server
    axum::serve(listener, router)
        .await
        .context("failed to start server")?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutting down gracefully");
            let _ = plugin_container.unload();
        }
    };

    Ok(())
}

fn config_tracing() {
    // Enable errors backtrace
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    use tracing::Level;
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

    let tracing_layer = tracing_subscriber::fmt::layer();
    let filter = filter::Targets::new()
        .with_target("hyper::proto", Level::INFO)
        .with_target("tower_http::trace", Level::DEBUG)
        .with_default(Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::Client;
    use tokio::{task, time::Instant};

    #[tokio::test]
    async fn test() {
        let client = Client::new();
        let url = "https://didcomm-mediator.eudi-adorsys.com/";
        let num_requests = 1000;

        let mut handles = Vec::new();
        let start = Instant::now();

        for _ in 0..num_requests {
            let client = client.clone();
            let url = url.to_string();

            let handle = task::spawn(async move {
                match fetch_with_retries(&client, &url).await {
                    Ok(_resp) => (),
                    Err(e) => panic!("{}", e),
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                panic!("{}", e)
            }
        }

        let duration = start.elapsed();
        println!("Completed {} requests in {:?}", num_requests, duration);
    }
}
