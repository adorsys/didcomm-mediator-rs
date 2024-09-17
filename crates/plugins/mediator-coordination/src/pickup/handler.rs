use std::sync::Arc;

use crate::{
    pickup::constants::STATUS_RESPONSE_3_0,
    pickup::error::PickupError,
    pickup::model::{BodyStatusResponse, StatusResponse},
    model::stateful::entity::Connection,
    web::{AppState, AppStateRepository},
};
use axum::response::{IntoResponse, Response};
use didcomm::Message;
use mongodb::bson::doc;
use serde_json::{json, Value};
use uuid::Uuid;

// Process pickup status request
pub(crate) async fn handle_status_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    let mediator_did = &state.diddoc.id;
    let recipient_did = message.body.get("recipient_did").and_then(Value::as_str);
    let sender_did = sender_did(&message).await?;

    let repository = repository(Arc::clone(&state)).await?;
    let connection = client_connection(&repository, sender_did).await?;

    let mut message_count = 0;
    if recipient_did.is_some() {
        if connection
            .keylist
            .iter()
            .any(|x| x == recipient_did.unwrap())
        {
            message_count = count_messages(&repository, recipient_did.unwrap()).await?;
        }
    } else {
        for did in &connection.keylist {
            let count = count_messages(&repository, did).await?;
            message_count += count;
        }
    }

    let id = Uuid::new_v4().urn().to_string();
    let status = StatusResponse {
        id: id.as_str(),
        type_: STATUS_RESPONSE_3_0,
        body: BodyStatusResponse {
            recipient_did,
            message_count,
            live_delivery: Some(false),
            ..Default::default()
        },
    };

    Ok(Message::build(
        Uuid::new_v4().urn().to_string(),
        STATUS_RESPONSE_3_0.to_string(),
        json!(status),
    )
    .to(sender_did.to_string())
    .from(mediator_did.clone())
    .finalize())
}

#[inline]
async fn count_messages(
    repository: &AppStateRepository,
    recipient_did: &str,
) -> Result<usize, Response> {
    let result = repository
        .message_repository
        .find_one_by(doc! { "recipient_did": recipient_did })
        .await
        .map_err(|err| PickupError::RepositoryError(err).into_response())?;

    Ok(result.map(|x| x.messages.len()).unwrap_or(0))
}

#[inline]
async fn sender_did(message: &Message) -> Result<&str, Response> {
    message
        .from
        .as_ref()
        .map(|did| did.as_str())
        .ok_or(PickupError::MissingSenderDID.into_response())
}

#[inline]
async fn repository(state: Arc<AppState>) -> Result<AppStateRepository, Response> {
    state
        .repository
        .clone()
        .ok_or(PickupError::MissingRepository.into_response())
}

#[inline]
async fn client_connection(
    repository: &AppStateRepository,
    client_did: &str,
) -> Result<Connection, Response> {
    repository
        .connection_repository
        .find_one_by(doc! { "client_did": client_did })
        .await
        .map_err(|err| PickupError::RepositoryError(err).into_response())?
        .ok_or(PickupError::MissingClientConnection.into_response())
}
