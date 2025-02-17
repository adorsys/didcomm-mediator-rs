use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::TraceLayer;

pub fn setup_metrics(router: Router) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    router
        .route(
            "/metrics",
            get(move || async move { metric_handle.render() }),
        )
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
}
