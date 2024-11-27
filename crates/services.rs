use prometheus::{IntCounterVec, Opts, Registry, Encoder, TextEncoder};
use std::sync::{Arc, Mutex};
use axum::response::IntoResponse;
use axum::http::StatusCode;

lazy_static::lazy_static! {
    static ref SERVICE_HEALTH_COUNTER: IntCounterVec = IntCounterVec::new(
        Opts::new("service_health", "Counts the health check status of various services"),
        &["service", "status"]
    ).unwrap();
}

pub struct ServiceHealth {
    pub name: String,
    pub status: String,
}

impl ServiceHealth {
    pub fn new(name: String, status: String) -> Self {
        ServiceHealth { name, status }
    }
}

pub fn service_health_check() -> Vec<ServiceHealth> {
    // Simulating health check for different services like database, messaging, etc.
    let services = vec![
        ServiceHealth::new("Database".to_string(), "healthy".to_string()),
        ServiceHealth::new("Keystore".to_string(), "healthy".to_string()),
    ];

    // Increment the health check counters for each service
    for service in &services {
        SERVICE_HEALTH_COUNTER
            .with_label_values(&[&service.name, &service.status])
            .inc();
    }

    services
}

pub async fn health_check() -> impl IntoResponse {
    // Simulate the checking of service health status
    let services = service_health_check();
    let response = services.iter()
        .map(|s| format!("{}: {}", s.name, s.status))
        .collect::<Vec<String>>()
        .join("\n");

    (StatusCode::OK, response)
}

pub async fn metrics_handler() -> impl IntoResponse {
    let registry = Arc::new(Mutex::new(prometheus::Registry::new()));
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    registry.lock().unwrap().gather();
    encoder.encode(&registry.lock().unwrap().gather(), &mut buffer).unwrap();

    (StatusCode::OK, buffer)
}
