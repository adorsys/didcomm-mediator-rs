use axum::{
    extract::State,
    response::IntoResponse,
};
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt};  // Make sure to use `SystemExt` here for easy access to system information

#[derive(Debug)]
pub struct AppMetrics {
    system: System,
}

impl AppMetrics {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        AppMetrics { system }
    }

    pub fn update_metrics(&mut self) {
        self.system.refresh_all();
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
            log::error!("Failed to lock Mutex: {}", poison_error);
            return "Error: failed to lock Mutex".into_response();
        }
    };
    metrics.update_metrics();

    let (memory_used, total_memory) = metrics.get_memory_usage();
    let cpu_usage = metrics.get_cpu_usage();

    format!(
        "Memory: {} used / {} total, CPU usage: {:.2}%",
        memory_used, total_memory, cpu_usage
    )
}
