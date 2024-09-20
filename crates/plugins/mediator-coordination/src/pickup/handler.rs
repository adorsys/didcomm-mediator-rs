use crate::{
    model::stateful::entity::{Connection, RoutedMessage},
    pickup::{
        constants::{MESSAGE_DELIVERY_3_0, PROBLEM_REPORT_2_0, STATUS_RESPONSE_3_0},
        error::PickupError,
        model::{
            BodyDeliveryResponse, BodyLiveDeliveryChange, BodyStatusResponse, DeliveryResponse,
            LiveDeliveryChange, StatusResponse,
        },
    },
    web::{AppState, AppStateRepository},
};
use axum::response::{IntoResponse, Response};
use didcomm::{Attachment, Message, MessageBuilder};
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

// Process pickup status request
pub(crate) async fn handle_status_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        return Err(PickupError::MalformedRequest(
            "Invalid \"return_route\" specifier".to_string(),
        )
        .into_response());
    }

    let mediator_did = &state.diddoc.id;
    let recipient_did = message.body.get("recipient_did").and_then(Value::as_str);
    let sender_did = sender_did(&message)?;

    let repository = repository(Arc::clone(&state))?;
    let connection = client_connection(&repository, sender_did).await?;

    let message_count = count_messages(repository, recipient_did, connection).await?;

    let id = Uuid::new_v4().urn().to_string();
    let response: MessageBuilder = StatusResponse {
        id: id.as_str(),
        type_: STATUS_RESPONSE_3_0,
        body: BodyStatusResponse {
            recipient_did,
            message_count,
            live_delivery: Some(false),
            ..Default::default()
        },
    }
    .into();

    let response = response
        .to(sender_did.to_string())
        .from(mediator_did.to_string())
        .finalize();

    Ok(response)
}

pub(crate) async fn handle_delivery_request(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        return Err(PickupError::MalformedRequest(
            "Invalid \"return_route\" specifier".to_string(),
        )
        .into_response());
    }

    let mediator_did = &state.diddoc.id;
    let recipient_did = message.body.get("recipient_did").and_then(Value::as_str);
    let sender_did = sender_did(&message)?;
    let limit = message
        .body
        .get("limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            PickupError::MalformedRequest("Invalid \"limit\" specifier".to_string()).into_response()
        })?;

    let repository = repository(Arc::clone(&state))?;
    let connection = client_connection(&repository, sender_did).await?;

    let messages = messages(repository, recipient_did, connection, limit as usize).await?;

    let response: MessageBuilder;
    let id = Uuid::new_v4().urn().to_string();
    let mut attachments: Vec<Attachment> = vec![];

    if messages.is_empty() {
        response = StatusResponse {
            id: id.as_str(),
            type_: STATUS_RESPONSE_3_0,
            body: BodyStatusResponse {
                recipient_did,
                message_count: 0,
                live_delivery: Some(false),
                ..Default::default()
            },
        }
        .into();
    } else {
        for message in messages {
            let attached = Attachment::json(message.message)
                .id(message.id.unwrap_or_default().to_string())
                .finalize();

            attachments.push(attached);
        }

        response = DeliveryResponse {
            id: id.as_str(),
            thid: id.as_str(),
            type_: MESSAGE_DELIVERY_3_0,
            body: BodyDeliveryResponse { recipient_did },
            attachments,
        }
        .into();
    }

    let response = response
        .to(sender_did.to_string())
        .from(mediator_did.to_string())
        .finalize();

    Ok(response)
}

pub(crate) async fn handle_message_acknowledgement(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        return Err(PickupError::MalformedRequest(
            "Invalid \"return_route\" specifier".to_string(),
        )
        .into_response());
    }

    let mediator_did = &state.diddoc.id;
    let repository = repository(Arc::clone(&state))?;
    let sender_did = sender_did(&message)?;
    let connection = client_connection(&repository, sender_did).await?;
    let message_id_list = message
        .body
        .get("message_id_list")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            PickupError::MalformedRequest("Invalid \"message_id_list\" specifier".to_string())
                .into_response()
        })?
        .iter()
        .map(|value| value.as_str().unwrap_or_default())
        .collect::<Vec<_>>();

    for id in message_id_list {
        let msg_id = ObjectId::from_str(id);
        if msg_id.is_err() {
            return Err(
                PickupError::MalformedRequest(format!("Invalid message id: {id}")).into_response(),
            );
        }
        repository
            .message_repository
            .delete_one(msg_id.unwrap())
            .await
            .map_err(|err| PickupError::RepositoryError(err).into_response())?;
    }

    let message_count = count_messages(repository, None, connection).await?;

    let id = Uuid::new_v4().urn().to_string();
    let response: MessageBuilder = StatusResponse {
        id: id.as_str(),
        type_: STATUS_RESPONSE_3_0,
        body: BodyStatusResponse {
            message_count,
            live_delivery: Some(false),
            ..Default::default()
        },
    }
    .into();

    let response = response
        .to(sender_did.to_string())
        .from(mediator_did.to_string())
        .finalize();

    Ok(response)
}

pub(crate) async fn handle_live_delivery_change(
    state: Arc<AppState>,
    message: Message,
) -> Result<Message, Response> {
    if message
        .extra_headers
        .get("return_route")
        .and_then(Value::as_str)
        != Some("all")
    {
        return Err(PickupError::MalformedRequest(
            "Invalid \"return_route\" specifier".to_string(),
        )
        .into_response());
    }

    let mediator_did = &state.diddoc.id;
    let sender_did = sender_did(&message)?;

    let id = Uuid::new_v4().urn().to_string();
    let response: MessageBuilder = LiveDeliveryChange {
        id: id.as_str(),
        pthid: if let Some(thid) = message.thid.as_ref() {
            thid.as_str()
        } else {
            id.as_str()
        },
        type_: PROBLEM_REPORT_2_0,
        body: BodyLiveDeliveryChange {
            code: "e.m.live-mode-not-supported",
            comment: "Connection does not support Live Delivery",
        },
    }
    .into();

    let response: MessageBuilder = match message.body.get("live_delivery").and_then(Value::as_bool)
    {
        Some(true) => response,

        Some(false) => response,

        None => {
            return Err(PickupError::MalformedRequest(
                "Missing \"live_delivery\" specifier".to_string(),
            )
            .into_response());
        }
    };

    let response = response
        .to(sender_did.to_string())
        .from(mediator_did.to_string())
        .finalize();

    Ok(response)
}

async fn count_messages(
    repository: AppStateRepository,
    recipient_did: Option<&str>,
    connection: Connection,
) -> Result<usize, Response> {
    let recipients = recipients(recipient_did, &connection);

    let count = repository
        .message_repository
        .find_all_by(doc! { "recipient_did": { "$in": recipients } }, None)
        .await
        .map_err(|err| PickupError::RepositoryError(err).into_response())?
        .len();

    Ok(count)
}

async fn messages(
    repository: AppStateRepository,
    recipient_did: Option<&str>,
    connection: Connection,
    limit: usize,
) -> Result<Vec<RoutedMessage>, Response> {
    let recipients = recipients(recipient_did, &connection);

    let routed_messages = repository
        .message_repository
        .find_all_by(
            doc! { "recipient_did": { "$in": recipients } },
            Some(limit as i64),
        )
        .await
        .map_err(|err| PickupError::RepositoryError(err).into_response())?;

    Ok(routed_messages)
}

#[inline]
fn recipients<'a>(recipient_did: Option<&'a str>, connection: &'a Connection) -> Vec<&'a str> {
    recipient_did
        .map(|did| {
            if connection.keylist.contains(&did.to_string()) {
                vec![did]
            } else {
                Vec::new()
            }
        })
        .unwrap_or_else(|| connection.keylist.iter().map(|s| s.as_str()).collect())
}

#[inline]
fn sender_did(message: &Message) -> Result<&str, Response> {
    message
        .from
        .as_ref()
        .map(|did| did.as_str())
        .ok_or(PickupError::MissingSenderDID.into_response())
}

#[inline]
fn repository(state: Arc<AppState>) -> Result<AppStateRepository, Response> {
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
