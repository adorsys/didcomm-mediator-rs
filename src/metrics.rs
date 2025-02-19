use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub request_count: Counter,
    pub api_response_time: Histogram,
}

pub fn setup_metrics(router: Router) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Custom metrics
    let mut registry = Registry::default();
    let request_count = Counter::default();
    let api_response_time = Histogram::new(vec![0.1, 0.5, 1.0, 2.0, 5.0]);

    registry.register(
        "http_requests_total",
        "Total number of HTTP requests",
        request_count.clone(),
    );
    registry.register(
        "api_response_time_seconds",
        "API response time in seconds",
        api_response_time.clone(),
    );

    let state = Arc::new(AppState {
        request_count,
        api_response_time,
    });

    router
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(axum::Extension(state))
}
