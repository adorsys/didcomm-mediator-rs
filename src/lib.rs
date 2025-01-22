pub mod plugins;

use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use eyre::{eyre, Result};
use hyper::Method;
use plugins::manager::PluginContainer;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn app() -> Result<(PluginContainer<'static>, Router)> {
    let mut container = PluginContainer::new();
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    container.load().map_err(|e| eyre!(e))?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let router = Router::new()
        .merge(container.routes().unwrap_or_default())
        .route("/metrics", get(|| async move { metric_handle.render()}))
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
        .layer(cors);

    Ok((container, router))
}

pub fn health_check() -> &'static str {
    "OK"
}