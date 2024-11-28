use axum::response::IntoResponse;
use hyper::StatusCode;
use prometheus::{Encoder, TextEncoder};

/// Prometheus metrics handler
pub async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, buffer)
}
