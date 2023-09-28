mod did;

use axum::Router;

pub fn routes() -> Router {
    Router::new() //
        .merge(did::routes())
}
