/*use prometheus::{Encoder, IntCounter, TextEncoder, Opts, Registry};
use actix_web::{web, App, HttpServer, Responder};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn metrics_handler(registry: web::Data<Arc<RwLock<Registry>>>) -> impl Responder {
    let registry = registry.read().await;
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&registry.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up metrics
    let registry = Arc::new(RwLock::new(Registry::new()));

    // Counter example
    let counter_opts = Opts::new("request_count", "Number of requests processed");
    let counter = IntCounter::with_opts(counter_opts).unwrap();
    registry.write().await.register(Box::new(counter.clone())).unwrap();
    counter.inc();

    // Set up server with the metrics endpoint
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(registry.clone()))
            .route("/metrics", web::get().to(metrics_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

    https://www.youtube.com/watch?v=5ep_NvXJ1Fw
    https://www.youtube.com/watch?v=zFDIj7OufE8
*/