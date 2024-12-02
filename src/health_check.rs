use actix_web::{web, App, HttpServer, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json({ "status";"UP" })
}
async fn readiness() -> impl Responder {
    HttpRespond::Ok().json({ "status";"READY" })
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
            .route("/readiness", web::get().to(readiness))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}