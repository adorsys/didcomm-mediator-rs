use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use std::sync::{Arc, Mutex};
use sysinfo::{System, RefreshKind};
use log::error;

#[derive(Debug)]
pub struct AppMetrics {
    system: System,
}

impl AppMetrics {
    pub fn new() -> Self {
        // Initialize system with required refresh options
        let system = System::new_with_specifics(RefreshKind::everything());
        AppMetrics { system }
    }

    pub fn update_metrics(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu_all();
    }

    pub fn get_memory_usage(&self) -> (u64, u64) {
        (self.system.used_memory(), self.system.total_memory())
    }

    pub fn get_cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }
}

pub async fn system_metrics(State(metrics): State<Arc<Mutex<AppMetrics>>>) -> impl IntoResponse {
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
