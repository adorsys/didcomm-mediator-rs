use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web_prom::PrometheusMetricsBuilder;
use sysinfo::{System, SystemExt};

async fn health() -> impl Responder {
    HttpResponse::Ok().json({ "status"; "UP" })
}

async fn readiness() -> impl Responder {
    HttpResponse::Ok().json({ "status"; "READY" })
}

fn system_metrics() {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("CPU usage: {:?}", sys.global_processor_info().cpu_usage());
    println!("Memory usage: {}/{}", sys.used_memory(), sys.total_memory());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize Prometheus
    let prometheus = PrometheusMetricsBuilder::new("api")
        .build()
        .expect("Failed to build PrometheusMetrics");

    // Log system metrics
    system_metrics();

    // Start the Actix server
    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .route("/metrics", web::get().to(|| async { "Metrics endpoint" }))
            .route("/health", web::get().to(health))
            .route("/readiness", web::get().to(readiness))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::http::StatusCode;
    use actix_web_prom::PrometheusMetricsBuilder;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "UP");
    }

    #[actix_web::test]
    async fn test_readiness_endpoint() {
        let app = test::init_service(
            App::new().route("/readiness", web::get().to(readiness))
        ).await;

        let req = test::TestRequest::get().uri("/readiness").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "READY");
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let prometheus = PrometheusMetricsBuilder::new("api");

        let app = test::init_service(
            App::new()
                .wrap(prometheus.clone())
                .route("/metrics", web::get().to(|| async { "Metrics endpoint" }))
        ).await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Metrics endpoint"));
    }
}
