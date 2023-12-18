use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::web::{AppState, AppStateRepository};

pub async fn test_connection_repository(State(state): State<AppState>) -> Json<Value> {
    let AppStateRepository {
        connection_repository,
        ..
    } = state.repository.expect("missing persistence layer");
    let connections = connection_repository.find_all().await.unwrap();
    Json(json!(connections))
}
