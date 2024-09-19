use std::collections::HashMap;

use didcomm::{
    did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Message,
    PackEncryptedOptions, UnpackOptions,
};
use ledger::{ALICE_DID, ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID, MEDIATOR_DID_DOC};
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
const DIDCOMM_CONTENT_TYPE: &str = "application/didcomm-encrypted+json";

mod ledger;
async fn mediate_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_owned(),
        json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
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
    println!("{:#?}", msg);
}
async fn keylist_update_payload() {
    // --- Building message from ALICE to MEDIATOR ---
    let msg = Message::build(
        "id_alice_keylist_update_request".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_owned(),
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
async fn test_pickup_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        "https://didcomm.org/messagepickup/3.0/status-request".to_owned(),
        json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
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

    // unpack response
    let (msg, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();

    println!("\nPickup Request Message\n{:#?}", msg);
}
async fn test_pickup_delivery_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        "https://didcomm.org/messagepickup/3.0/delivery-request".to_owned(),
        json!({"limit":10,"recipient_did":"did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_string())
    .from(ALICE_DID.to_string())
    .finalize();

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
    println!("{}", msg);
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

    // unpack response
    let (msg, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();

    println!("\nPickup Delivery Message\n{:#?}", msg);
}

#[tokio::main]
async fn main() {
    println!("\n=================== MEDIATING REQUEST ===================\n");
    mediate_request().await;
    println!("\n=================== PICKUP REQUEST ===================\n");
    test_pickup_request().await;
    println!("\n=================== PICKUP DELIVERY ===================\n");
    test_pickup_delivery_request().await;
}
