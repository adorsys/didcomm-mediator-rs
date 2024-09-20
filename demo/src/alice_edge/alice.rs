use didcomm::{
    did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, Message,
    PackEncryptedOptions, UnpackOptions,
};
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};

use crate::{
    alice_edge::{
        constants::{DID_DOC_ENDPOINT, MEDIATE_REQUEST_2_0, MEDIATION_ENDPOINT},
        secret_data::MEDIATOR_DID,
    },
    ledger::{ALICE_DID, ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID_DOC},
    DIDCOMM_CONTENT_TYPE,
};

use super::secret_data::ROUTING_DID;

pub(crate) async fn get_mediator_didoc() {
    let client = reqwest::Client::new();

    let response = client
        .get(DID_DOC_ENDPOINT)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let did_doc: did_utils::didcore::Document = serde_json::from_str(&response).unwrap();
    let mut mediator_did = MEDIATOR_DID.lock().unwrap();
    *mediator_did = did_doc.clone().id;
    println!("\n Mediator DID Document {}\n", response)
}
pub(crate) async fn mediate_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        MEDIATE_REQUEST_2_0.to_owned(),
        json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.lock().unwrap().clone())
    .from(ALICE_DID.to_string())
    .finalize();

    // pack message for mediator
    let (msg, _) = msg
        .pack_encrypted(
            &MEDIATOR_DID.lock().unwrap(),
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
        .post(MEDIATION_ENDPOINT)
        .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
        .body(msg)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // Unpack response
    let (msg, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();
    let result = msg
        .body
        .get("body")
        .unwrap()
        .as_object()
        .unwrap()
        .get("routing_did")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let mut routing_did = ROUTING_DID.lock().unwrap();
    *routing_did = result;
    println!("\nMediation Request Response{:#?}\n", msg,)
}
pub(crate) async fn keylist_update_payload() {
    // --- Building message from ALICE to MEDIATOR ---
    let msg = Message::build(
        "id_alice_keylist_update_request".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/mediate-request".to_owned(),
        json!({"updates": [
        {
            "recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
            "action": "add"
        },
        {
            "recipient_did": "did:key:alice_identity_pub2@alice_mediator", // not existing did just for demonstration
            "action": "remove"
        }
        ]}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.lock().unwrap().clone())
    .from(ALICE_DID.to_owned())
    .finalize();

    // --- Packing encrypted and authenticated message ---
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());
    println!("this is mediator did {}", MEDIATOR_DID.lock().unwrap());

    let (msg, _) = msg
        .pack_encrypted(
            &MEDIATOR_DID.lock().unwrap(),
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
        .post(MEDIATION_ENDPOINT)
        .header(CONTENT_TYPE, DIDCOMM_CONTENT_TYPE)
        .body(msg)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let unpacked_msg = Message::unpack(&response, &did_resolver, &secrets_resolver, &UnpackOptions::default()).await.unwrap();
    println!("\nMediation Update Response{:#?}\n", unpacked_msg,)
}
