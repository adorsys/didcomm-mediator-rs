use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use sysinfo::System;
use std::net::SocketAddr;
use tracing::info;

#[derive(Debug)]
struct AppMetrics {
    system: System,
}

impl AppMetrics {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all(); 
        AppMetrics { system }
    }

    fn update_metrics(&mut self) {
        self.system.refresh_all();
    }
}


pub(crate) async fn health_check() -> impl IntoResponse {
    "Server is healthy"
}

async fn system_metrics(State(metrics): State<Arc<Mutex<AppMetrics>>>) -> impl IntoResponse {
    let mut metrics = metrics.lock().expect("Failed to lock Mutex");
    metrics.update_metrics();

    let memory_used = metrics.system.used_memory();
    let total_memory = metrics.system.total_memory();
    let cpu_usage = metrics.system.global_cpu_usage();

    format!(
        "Memory: {} used / {} total, CPU usage: {:.2}%",
        memory_used, total_memory, cpu_usage
    )
}

pub fn create_router() -> Router {
    let metrics = Arc::new(Mutex::new(AppMetrics::new()));
    Router::new()
        .route("/health", get(health_check)) 
        .route("/health/metrics", get(system_metrics)) 
        .with_state(metrics)
}

#[tokio::main]
async fn main() {
    // Set up tracing
    tracing_subscriber::fmt::init();

    // Create the application router
    let app = create_router();

    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running at http://{}", addr);

    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
