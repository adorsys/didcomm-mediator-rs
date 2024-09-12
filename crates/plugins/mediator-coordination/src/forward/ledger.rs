use didcomm::{
    did::DIDDoc,
    secrets::{Secret, SecretMaterial, SecretType},
};
use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
    // Obtained from oob band invitation
  pub(crate) static ref MEDIATOR_DID_DOC: DIDDoc = serde_json::from_str(
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
         "assertionMethod": [
           "did:web:alice-mediator.com:alice_mediator_pub#keys-2"
         ],
         "keyAgreement": [
           "did:web:alice-mediator.com:alice_mediator_pub#keys-3"
         ],
         "service": []
       }"#
 ).unwrap();
 pub(crate) static ref ALICE_DID_DOC: DIDDoc = serde_json::from_str(
     r#"{
     "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
     "verificationMethod": [
         {
             "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
             "type": "JsonWebKey2020",
             "controller": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
             "publicKeyJwk": {
                 "kty": "OKP",
                 "crv": "Ed25519",
                 "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
             }
         },
         {
             "id": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr",
             "type": "JsonWebKey2020",
             "controller": "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7",
             "publicKeyJwk": {
                 "kty": "OKP",
                 "crv": "X25519",
                 "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU"
             }
         }
     ],
     "authentication": [
         "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
     ],
     "assertionMethod": [
         "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
     ],
     "capabilityDelegation": [
         "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
     ],
     "capabilityInvocation": [
         "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7"
     ],
     "keyAgreement": [
         "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr"
     ],
     "service": []
 }"#
)
.unwrap();

  pub(crate) static ref ALICE_SECRETS: Vec<Secret> = vec![
    Secret {
        id: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6LSbuUXWSgPfpiDBjUK6E7yiCKMN2eKJsXn5b55ZgqGz6Mr".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!({
                "kty": "OKP",
                "crv": "X25519",
                "x": "A2gufB762KKDkbTX0usDbekRJ-_PPBeVhc2gNgjpswU",
                "d": "oItI6Jx-anGyhiDJIXtVAhzugOha05s-7_a5_CTs_V4"
            })
        },
    },
    Secret {
        id: "did:key:z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7#z6MkfyTREjTxQ8hUwSwBPeDHf3uPL3qCjSSuNPwsyMpWUGH7".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "d": "UXBdR4u4bnHHEaDK-dqE04DIMvegx9_ZOjm--eGqHiI",
                "x": "Fpf4juyZWYUNmC8Bv87MmFLDWApxqOYYZUhWyiD7lSo"
            })
        },
    },
  ];
}
