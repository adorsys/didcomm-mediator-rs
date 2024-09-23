
use did_utils::didcore::Document;
use didcomm::{
    algorithms::AnonCryptAlg, protocols::routing::wrap_in_forward, secrets::resolvers::ExampleSecretsResolver, Attachment, AttachmentData, JsonAttachmentData, Message, PackEncryptedOptions
};
use mediator_coordination::didcomm::bridge::LocalDIDResolver;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use uuid::Uuid;

use crate::{constants::{MEDIATION_ENDPOINT, MEDIATOR_DID}, DIDCOMM_CONTENT_TYPE};

use super::data::{_sender_secrets_resolver, BOB_SECRETS};

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
let did_resolver = LocalDIDResolver::new(&doc);
    let _secrets_resolver = ExampleSecretsResolver::new(BOB_SECRETS.clone());
    let attachment = Attachment {
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
        Uuid::new_v4().to_string(),
        "example/v1".to_owned(),
        json!({}),
    )
    .to(_recipient_did())
    .from(_sender_did())
    .attachments(vec![attachment])
    .finalize();

    let (packed_forward_msg, _metadata) = msg
        .pack_encrypted(
            &_recipient_did(),
            Some("did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3"),
            None,
            &did_resolver,
            &_sender_secrets_resolver(),
            &PackEncryptedOptions::default(),
        )
        .await
        .expect("Unable pack_encrypted");

    let msg = wrap_in_forward(
        &packed_forward_msg,
        None,
        &&_recipient_did(),
        &vec![MEDIATOR_DID.to_string()],
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
        .unwrap();
    println!("{}", response.status())
}

pub fn _sender_did() -> String {
    "did:key:z6MkwKfDFAK49Lb9D6HchFiCXdcurRUSFrbnwDBk5qFZeHA3".to_string()
}

pub fn _recipient_did() -> String {
    "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".to_string()
}
