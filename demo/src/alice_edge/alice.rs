use did_utils::didcore::Document;
use didcomm::{
    did::resolvers::ExampleDIDResolver, secrets::resolvers::ExampleSecretsResolver, AttachmentData,
    Message, PackEncryptedOptions, UnpackOptions,
};
use mediator_coordination::didcomm::bridge::LocalDIDResolver;
use reqwest::{header::CONTENT_TYPE, Client};
use serde_json::json;
use uuid::Uuid;

use crate::{
    alice_edge::data::MEDIATOR_DID,
    constants::{
        DID_DOC_ENDPOINT, LIVE_MODE_CHANGE_3_0, MEDIATE_REQUEST_2_0, MEDIATE_UPDATE_2_0,
        MEDIATION_ENDPOINT, PICKUP_DELIVERY_3_0, PICKUP_RECIEVE_3_0, PICKUP_REQUEST_3_0,
    },
    ledger::{ALICE_DID, ALICE_DID_DOC, ALICE_SECRETS, MEDIATOR_DID_DOC},
    DIDCOMM_CONTENT_TYPE,
};

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

    println!("\n Mediator DID Document {}\n", response)
}
pub(crate) async fn mediate_request() {
    let did_resolver: ExampleDIDResolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver: ExampleSecretsResolver =
        ExampleSecretsResolver::new(ALICE_SECRETS.clone());
    // Build message
    let msg = Message::build(
        Uuid::new_v4().to_string(),
        MEDIATE_REQUEST_2_0.to_owned(),
        json!({"recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_string())
    .from(ALICE_DID.to_string())
    .finalize();

    // pack message for mediator
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

    println!("\nMediation Request Response{:#?}\n", msg,)
}
pub(crate) async fn keylist_update_payload() {
    // --- Building message from ALICE to MEDIATOR ---
    let msg = Message::build(
        "id_alice_keylist_update_request".to_owned(),
        MEDIATE_UPDATE_2_0.to_owned(),
        json!({"updates": [
        {
            "recipient_did": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
            "action": "add"
        },
        {
            "recipient_did": "did:key:alice_identity_pub2@alice_mediator",
            "action": "remove"
        }
        ]}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_string())
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
    let unpacked_msg = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();
    println!("\nMediation Update Response{:#?}\n", unpacked_msg,)
}

pub(crate) async fn test_pickup_request() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        PICKUP_REQUEST_3_0.to_owned(),
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
        .post(MEDIATION_ENDPOINT)
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

pub(crate) async fn test_pickup_delivery_request() {
    let doc: Document = serde_json::from_str(
        r#"{
            "@context": [
                "https://www.w3.org/ns/did/v1",
                "https://w3id.org/security/suites/jws-2020/v1"
            ],
            "id": "did:web:alice-mediator.com:alice_mediator_pub",
            "verificationMethod": [
                {
                    "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-1",
                    "type": "JsonWebKey2020",
                    "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
                    }
                },
                {
                    "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-2",
                    "type": "JsonWebKey2020",
                    "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "Ed25519",
                        "x": "Z0GqpN71rMcnAkky6_J6Bfknr8B-TBsekG3qdI0EQX4"
                    }
                },
                {
                    "id": "did:web:alice-mediator.com:alice_mediator_pub#keys-3",
                    "type": "JsonWebKey2020",
                    "controller": "did:web:alice-mediator.com:alice_mediator_pub",
                    "publicKeyJwk": {
                        "kty": "OKP",
                        "crv": "X25519",
                        "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ"
                    }
                }
            ],
            "authentication": [
                "did:web:alice-mediator.com:alice_mediator_pub#keys-1"
            ],
            "keyAgreement": [
                "did:web:alice-mediator.com:alice_mediator_pub#keys-3"
            ],
            "service": []
        }"#,
    )
    .unwrap();
    let did_resolver = LocalDIDResolver::new(&doc);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        PICKUP_DELIVERY_3_0.to_owned(),
        json!({"limit":2,"recipient_did":"did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"}),
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

    // unpack response
    let (msg, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();
    let attachments = msg.attachments.unwrap();
    for attachemnt in attachments {
        // let val = match attachemnt.data {
        //     AttachmentData::Json { value: val } => val.json.clone(),
        //     _ => json!(0)
        // };
        let message = serde_json::to_string(&match attachemnt.data {
            AttachmentData::Json { value: val } => val.json.clone(),
            _ => json!(0),
        })
        .unwrap();

        let message = Message::unpack(
            &message,
            &did_resolver,
            &secrets_resolver,
            &UnpackOptions::default(),
        )
        .await
        .unwrap();
        println!("\nPickup Delivery Message\n{:#?}", message);
    }
}

pub(crate) async fn test_pickup_message_received() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        PICKUP_RECIEVE_3_0.to_owned(),
        json!({"message_id_list": vec!["66ec4d76e8aaed777d76acf9","66ec4d75e8aaed777d76acf8"]}),
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

    println!("\nMessage Received\n{:#?}", msg);
}

pub(crate) async fn keylist_query_payload() {
    let client = Client::new();

    // --- Building message from ALICE to MEDIATOR ---
    let message = Message::build(
        "id_alice_keylist_query".to_owned(),
        "https://didcomm.org/coordinate-mediation/2.0/keylist-query".to_owned(),
        json!({}),
    )
    .header("return_route".into(), json!("all"))
    .to(MEDIATOR_DID.to_string())
    .from(ALICE_DID.to_string())
    .finalize();

    // --- Packing encrypted and authenticated message ---
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    let (message, _) = message
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

    let response = client
        .post("http://localhost:3000/mediate")
        .header(CONTENT_TYPE, "application/didcomm-encrypted+json")
        .body(message)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("packed message from");

    // --- Unpack the message ---
    let (message, _) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .unwrap();
    print!("{:#?}", message)
}
pub(crate) async fn handle_live_delivery_change() {
    let did_resolver =
        ExampleDIDResolver::new(vec![ALICE_DID_DOC.clone(), MEDIATOR_DID_DOC.clone()]);
    let secrets_resolver = ExampleSecretsResolver::new(ALICE_SECRETS.clone());

    // Build message
    let msg = Message::build(
        "urn:uuid:8f8208ae-6e16-4275-bde8-7b7cb81ffa59".to_owned(),
        LIVE_MODE_CHANGE_3_0.to_owned(),
        json!({"message_id_list": vec!["66ec4d76e8aaed777d76acf9","66ec4d75e8aaed777d76acf8"]}),
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

    println!("\nMessage Received\n{:#?}", msg);
}
