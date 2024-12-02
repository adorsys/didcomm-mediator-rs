use actix_weeb::{web, App, HttpServer};
use actix_web_prom::PrometheusMetrics;
use sysinfo::{System, SystemExt};

fn system_metrics() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("CPU usage: {:?}", sys.global_processor_info().cpu_usage());
    println!("Memory usage: {}/{}", sys.used_memory(), sys.total_memory());
}

fn main() {
    system_metrics();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let prometheus = PrometheusMetrics::new("api", None);

    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .route("/metrics", web::get().to(|| async { "Metrics endpoint" }))
            .route("/health", web::get().to(|| async { "UP" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}