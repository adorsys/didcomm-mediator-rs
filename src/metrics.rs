use prometheus::{Encoder, TextEncoder, register_counter_vec};
use axum::response::IntoResponse;

lazy_static::lazy_static! {
    static ref HTTP_COUNTER: prometheus::CounterVec = register_counter_vec!(
        "http_requests_total",
        "Number of HTTP requests",
        &["method", "endpoint"]
    ).unwrap();
}

pub async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, buffer)
}
