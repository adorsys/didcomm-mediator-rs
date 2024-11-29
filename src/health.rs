use axum::{response::IntoResponse, Json};
use serde_json::json;
use sysinfo::{ProcessorExt, System, SystemExt};

/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Example: Add specific checks for your application components here
    let database_status = "healthy"; 
    let keystore_status = "healthy";

    let overall_status = if database_status == "healthy" && keystore_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    Json(json!({
        "status": overall_status,
        "components": {
            "database": database_status,
            "keystore": keystore_status
        },
        "system_metrics": {
            "cpu_usage": sys.global_processor_info().cpu_usage(),
            "memory_used": sys.used_memory(),
            "memory_total": sys.total_memory(),
        }
    }))
}
