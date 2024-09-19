fn main() {
    println!("Hello, world!");
}
use std::collections::HashMap;

use didcomm::{
    did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Message,
    PackEncryptedOptions, UnpackOptions,
};
use ledger::{ALICE_DID, ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID, MEDIATOR_DID_DOC};
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
const DIDCOMM_CONTENT_TYPE: &str = "application/didcomm-encrypted+json";
static ROUTING_DID: Lazy<String> = Lazy::new(|| {});

mod ledger;
#[tokio::main]
async fn main() {
    println!("\n=================== MEDIATING REQUEST ===================\n");
    mediate_request().await;
}
async fn mediate_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_owned(),
        json!({}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_string())
    .from(ALICE_DID.to_string())
    .finalize();

    // Encrypt message for mediator

    let (msg, _) = msg
        .pack_encrypted(
            &MEDIATOR_DID,
            Some(&ALICE_DID),
            None,
            &did_resolver,
            &secrets_resolver,
            &PackEncryptedOptions::default(),
        )
        .await
        .expect("Unable to pack_encrypted");
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/mediate")
        .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
        .body(msg)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("packed message from");

    // unpack response
    let (msg, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();
  ////  ROUTING_DID = msg.body.get("r")

}
async fn keylist_update_payload() {
    // --- Building message from ALICE to MEDIATOR ---
    let msg = Message::build(
        "id_alice_keylist_update_request".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/keylist-update".to_owned(),
        json!({"updates": [
            {
                "recipient_did": "did:key:alice_identity_pub1@alice_mediator",
                "action": "add"
            },
            {
                "recipient_did": "did:key:alice_identity_pub2@alice_mediator",
                "action": "remove"
            }
        ]}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_owned())
    .from(ALICE_DID.to_owned())
    .finalize();

    // --- Packing encrypted and authenticated message ---
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    let (msg, _) = msg
        .pack_encrypted(
            &MEDIATOR_DID,
            Some(&ALICE_DID),
            None,
            &did_resolver,
            &secrets_resolver,
            &PackEncryptedOptions::default(),
        )
        .await
        .expect("Unable to pack_encrypted");

    // --- Sending message by Alice ---
    println!("Edge agent is sending message \n{}\n", msg);
}
