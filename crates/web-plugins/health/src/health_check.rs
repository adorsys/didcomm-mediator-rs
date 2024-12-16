use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use sysinfo::{System, RefreshKind};
use std::net::SocketAddr;
use tracing::{info, error};

#[derive(Debug)]
struct AppMetrics {
    system: System,
}

impl AppMetrics {
    fn new() -> Self {
        // Initialize system with required refresh options
        let system = System::new_with_specifics(RefreshKind::everything());
        AppMetrics { system }
    }

    fn update_metrics(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu_all();
    }

    fn get_memory_usage(&self) -> (u64, u64) {
        (self.system.used_memory(), self.system.total_memory())
    }

    fn get_cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }
}

pub(crate) async fn health_check() -> impl IntoResponse {
    Html("Server is healthy".to_string())
}

async fn system_metrics(State(metrics): State<Arc<Mutex<AppMetrics>>>) -> impl IntoResponse {
    let mut metrics = match metrics.lock() {
        Ok(guard) => guard,
        Err(poison_error) => {
            error!("Failed to lock Mutex: {}", poison_error);
            return Html("Error: failed to lock Mutex".to_string());
        }
    };

    metrics.update_metrics();

    let (memory_used, total_memory) = metrics.get_memory_usage();
    let cpu_usage = metrics.get_cpu_usage();

    Html(format!(
        "Memory: {} used / {} total, CPU usage: {:.2}%",
        memory_used, total_memory, cpu_usage
    ))
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
