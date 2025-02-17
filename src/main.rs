use axum::{response::IntoResponse, routing::get, Json};
use axum_prometheus::PrometheusMetricLayer;
use didcomm_mediator::app;
use eyre::{Result, WrapErr};
use hyper::StatusCode;
use mongodb::{options::ClientOptions, Client};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    // Load dotenv-flow variables
    dotenv_flow::dotenv_flow()?;

    // Enable logging
    config_tracing();

    // Configure server
    let port = std::env::var("SERVER_LOCAL_PORT").unwrap();
    let port = port.parse().context("failed to parse port")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .context("failed to parse address")?;

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

    // Set up Prometheus metrics
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Add health check endpoint, metrics, and trace layer
    let router = router
        .route("/health", get(health_check))
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http());

    // Start the server
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

async fn health_check() -> impl IntoResponse {
    let mongo_url =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

    match check_mongo_connection(&mongo_url).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "mongo": "connected" })),
        ),
        Err(err) => {
            tracing::error!("Health check failed: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "error", "error": err.to_string() })),
            )
        }
    }
}

async fn check_mongo_connection(mongo_url: &str) -> Result<(), eyre::Report> {
    let client_options = ClientOptions::parse(mongo_url).await?;
    let client = Client::with_options(client_options)?;

    // Try to list databases as a health check
    let databases = client.list_database_names(None, None).await?;
    tracing::debug!("Connected to MongoDB. Databases: {:?}", databases);

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
                match client.get(&url).send().await {
                    Ok(_resp) => (),
                    Err(e) => panic!("{}", e),
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let a = handle.await;
            if let Err(e) = a {
                panic!("{}", e)
            }
        }

        let duration = start.elapsed();
        println!("Completed {} requests in {:?}", num_requests, duration);
    }
}
