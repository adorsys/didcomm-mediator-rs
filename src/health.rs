use actix_web::{HttpResponse, Responder};
use sysinfo::{System, SystemExt};
use crate::tower_http::services::{database, keystore};
/// Health check endpoint
pub async fn health_check() -> impl Responder {
    let mut sys = System::new_all();
    sys.refresh_all();

    let db_status = database::check_database_health();
    let keystore_status = keystore::check_keystore_health();

    let overall_status = if db_status == "healthy" && keystore_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    let health_status = serde_json::json!({
        "status": overall_status,
        "components": {
            "database": db_status,
            "keystore": keystore_status,
        },
        "system_metrics": {
            "cpu_usage": sys.global_processor_info().cpu_usage(),
            "memory_used": sys.used_memory(),
            "memory_total": sys.total_memory(),
        }
    });

    HttpResponse::Ok().json(health_status)
}
