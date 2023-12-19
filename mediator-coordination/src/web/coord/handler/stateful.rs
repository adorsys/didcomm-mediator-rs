use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::{json, Value};

use crate::{
    constant::KEYLIST_UPDATE_2_0,
    model::stateful::coord::KeylistUpdate,
    web::{coord::error::MediationError, AppState, AppStateRepository},
};

pub async fn test_connection_repository(State(state): State<AppState>) -> Json<Value> {
    let AppStateRepository {
        connection_repository,
        ..
    } = state.repository.expect("missing persistence layer");

    let connections = connection_repository.find_all().await.unwrap();
    Json(json!(connections))
}

#[axum::debug_handler]
pub async fn process_plain_keylist_update_message(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    Json(keylist_update): Json<KeylistUpdate>,
) -> Response {
    // Temp! Read declared sender from message
    let sender = query.get("sender").cloned();

    // Validate sender
    if sender.is_none() {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::Generic(String::from("no declared sender")).json(),
        );

        return response.into_response();
    }

    // Validate message type
    if keylist_update.message_type != KEYLIST_UPDATE_2_0 {
        let response = (
            StatusCode::BAD_REQUEST,
            MediationError::InvalidMessageType.json(),
        );

        return response.into_response();
    }

    // Retrieve repository to connection entities
    let AppStateRepository {
        connection_repository,
        ..
    } = state.repository.expect("missing persistence layer");

    // Find connection for this keylist update
    let connection = match connection_repository
        .find_one_by(doc! { "client_did": sender.unwrap() })
        .await
        .unwrap()
    {
        Some(connection) => connection,
        None => {
            let response = (
                StatusCode::UNAUTHORIZED,
                MediationError::UncoordinatedSender.json(),
            );

            return response.into_response();
        }
    };

    Json(json!(connection)).into_response()
}
