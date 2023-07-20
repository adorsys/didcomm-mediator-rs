use crate::coordinate_mediation::models::{MediateGrant, MediateRequest};
use crate::error::Result;
use crate::models::RecipientController;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

// Routes for RecipientController
pub fn routes(recipient_controller: RecipientController) -> Router {
    Router::new()
        .route(
            "/coordinate-mediation/2.0/mediate-request",
            post(handler_mediate_request),
        )
        .with_state(recipient_controller)
}

// Handler for create tickets
async fn handler_mediate_request<'a>(
    State(recipient_controller): State<RecipientController>,
    // ctx: Ctx,
    Json(mediaRequest): Json<MediateRequest>,
) -> Result<Json<MediateGrant>> {
    println!("->> {:<12} - handler_mediate_request", "HANDLER");
    // add the entry using the recipient controller
    let routing_did = recipient_controller
        .process_mediation_request(&mediaRequest.id)
        .await
        .unwrap();
    let mediateGrant: MediateGrant = MediateGrant::new(mediaRequest.id, routing_did.to_owned());
    Ok(Json(mediateGrant))
}
// region: --- Controller

// endregion: --- Controller
