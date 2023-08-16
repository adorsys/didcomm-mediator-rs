mod index;

use axum::Router;

pub fn routes() -> Router {
    Router::new().merge(index::routes())
}
