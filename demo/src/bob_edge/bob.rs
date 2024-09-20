use did_utils::{crypto::PublicKeyFormat, didcore::Document};
use didcomm::{
    algorithms::AnonCryptAlg, did::{resolvers::ExampleDIDResolver, DIDDoc, DIDResolver}, error::{Error, ErrorKind}, protocols::routing::wrap_in_forward, secrets::resolvers::ExampleSecretsResolver, Attachment, AttachmentData, JsonAttachmentData, Message, PackEncryptedOptions, UnpackOptions
};
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use uuid::Uuid;
use crate::{
    alice_edge::{constants::MEDIATION_ENDPOINT, secret_data::{MEDIATOR_DID, ROUTING_DID}},
    bob_edge::{
        constants::BOB_DID,
        data::{_sender_secrets_resolver, BOB_DID_DOC, BOB_SECRETS, MEDIATOR_DID_DOC},
    },
    ledger::ALICE_DID, DIDCOMM_CONTENT_TYPE,
};

pub(crate) async fn forward_msg() {
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
    let did_resolver = mediator_coordination::didcomm::bridge::LocalDIDResolver::new(&doc);
    let secrets_resolver = ExampleSecretsResolver::new(BOB_SECRETS.clone());

    let plaintest_msg = Attachment {
        id: None,
        description: Some("A friendly reminder to take a break and enjoy some fresh air!".to_string()),
        media_type: None,
        data: AttachmentData::Json { value: JsonAttachmentData{json: json!("Hey there! Just wanted to remind you to step outside for a bit. A little fresh air can do wonders for your mood."), jws: None} },
        filename: Some("reminder.txt".to_string()),
        format: Some("mime_type".to_string()),
        lastmod_time: None,
        byte_count: None
    };
    let msg = Message::build(
        "example-1".to_owned(),
        "example/v1".to_owned(),
        json!("example-body"),
    )
    .to(ALICE_DID.to_owned())
    .from(BOB_DID.to_owned())
    .attachments(vec![plaintest_msg])
    .finalize();

    let (packed_forward_msg, _metadata) = msg
        .pack_encrypted(
            &ALICE_DID,
            Some(BOB_DID),
            None,
            &did_resolver,
            &_sender_secrets_resolver(),
            &PackEncryptedOptions::default(),
        )
        .await
        .expect("Unable pack_encrypted");
    println!("Encryption metadata is\n{:?}\n", _metadata);

    // --- Sending message to Alice ---
    println!("Alice is sending message \n{}\n", packed_forward_msg);

    let msg = wrap_in_forward(
        &packed_forward_msg,
        None,
        &ALICE_DID,
        &vec!["did:peer:2.Ez6MkiEHxxUjjjXb62JrGNPbBqBewrU2PY9ppGgH4bUBfMpzH.Vz6LSr61MU6UwZArRSFe6vH4wnqM63a127g1L5XX9dPuSBYxm.SeyJhIjpbImRpZGNvbW0vdjIiXSwiaWQiOiIjZGlkY29tbSIsInMiOiJodHRwOi8vYWxpY2UtbWVkaWF0b3IuY29tIiwidCI6ImRtIn0".to_string()],
        &AnonCryptAlg::default(),
        &did_resolver,
    )
    .await
    .expect("Unable wrap_in_forward");

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

    let (msg, _metadata) = Message::unpack(
        &response,
        &did_resolver,
        &secrets_resolver,
        &UnpackOptions::default(),
    )
    .await
    .expect("Unable unpack");
    let unpacked_msg = Message::unpack(&response, &did_resolver, &secrets_resolver, &UnpackOptions::default()).await.unwrap();

}
