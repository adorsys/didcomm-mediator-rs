use did_utils::jwk::Jwk;
use lazy_static::lazy_static;
use mediator_coordination::didcomm::bridge::LocalSecretsResolver;
use serde_json::json;

use didcomm::{did::{
    DIDCommMessagingService, DIDDoc, Service, ServiceKind, VerificationMaterial,
    VerificationMethod, VerificationMethodType,
}, secrets::{Secret, SecretMaterial, SecretType, SecretsResolver}};

use super::constants::BOB_DID;

lazy_static! {
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-x25519-1".into(),
            controller: "did:example:bob#key-x25519-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "GDTrI66K0pFfO54tlCSvfjjNapIs44dzpneBgyx0S3E",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_2: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-x25519-2".into(),
            controller: "did:example:bob#key-x25519-2".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "UT9S3F5ep16KSNBBShU2wh3qSfqYjlasZimn0mB8_VM",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_3: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-x25519-3".into(),
            controller: "did:example:bob#key-x25519-3".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "82k2BTUiywKv49fKLZa-WwDi8RBf0tB0M8bvSAUQ3yY",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_NOT_IN_SECRETS_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-x25519-not-secrets-1".into(),
            controller: "did:example:bob#key-x25519-not-secrets-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "82k2BTUiywKv49fKLZa-WwDi8RBf0tB0M8bvSAUQ3yY",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p256-1".into(),
            controller: "did:example:bob#key-p256-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "FQVaTOksf-XsCUrt4J1L2UGvtWaDwpboVlqbKBY2AIo",
                    "y": "6XFB9PYo7dyC5ViJSO9uXNYkxTJWn0d_mqJ__ZYhcNY",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_2: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p256-2".into(),
            controller: "did:example:bob#key-p256-2".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "n0yBsGrwGZup9ywKhzD4KoORGicilzIUyfcXb1CSwe0",
                    "y": "ov0buZJ8GHzV128jmCw1CaFbajZoFFmiJDbMrceCXIw",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_NOT_IN_SECRETS_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p256-not-secrets-1".into(),
            controller: "did:example:bob#key-p256-not-secrets-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "n0yBsGrwGZup9ywKhzD4KoORGicilzIUyfcXb1CSwe0",
                    "y": "ov0buZJ8GHzV128jmCw1CaFbajZoFFmiJDbMrceCXIw",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p384-1".into(),
            controller: "did:example:bob#key-p384-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-384",
                    "x": "MvnE_OwKoTcJVfHyTX-DLSRhhNwlu5LNoQ5UWD9Jmgtdxp_kpjsMuTTBnxg5RF_Y",
                    "y": "X_3HJBcKFQEG35PZbEOBn8u9_z8V1F9V1Kv-Vh0aSzmH-y9aOuDJUE3D4Hvmi5l7",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_2: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p384-2".into(),
            controller: "did:example:bob#key-p384-2".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-384",
                    "x": "2x3HOTvR8e-Tu6U4UqMd1wUWsNXMD0RgIunZTMcZsS-zWOwDgsrhYVHmv3k_DjV3",
                    "y": "W9LLaBjlWYcXUxOf6ECSfcXKaC3-K9z4hCoP0PS87Q_4ExMgIwxVCXUEB6nf0GDd",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_NOT_IN_SECRETS_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p384-not-secrets-1".into(),
            controller: "did:example:bob#key-p384-not-secrets-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-384",
                    "x": "2x3HOTvR8e-Tu6U4UqMd1wUWsNXMD0RgIunZTMcZsS-zWOwDgsrhYVHmv3k_DjV3",
                    "y": "W9LLaBjlWYcXUxOf6ECSfcXKaC3-K9z4hCoP0PS87Q_4ExMgIwxVCXUEB6nf0GDd",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p521-1".into(),
            controller: "did:example:bob#key-p521-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-521",
                    "x": "Af9O5THFENlqQbh2Ehipt1Yf4gAd9RCa3QzPktfcgUIFADMc4kAaYVViTaDOuvVS2vMS1KZe0D5kXedSXPQ3QbHi",
                    "y": "ATZVigRQ7UdGsQ9j-omyff6JIeeUv3CBWYsZ0l6x3C_SYqhqVV7dEG-TafCCNiIxs8qeUiXQ8cHWVclqkH4Lo1qH",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_2: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p521-2".into(),
            controller: "did:example:bob#key-p521-2".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-521",
                    "x": "ATp_WxCfIK_SriBoStmA0QrJc2pUR1djpen0VdpmogtnKxJbitiPq-HJXYXDKriXfVnkrl2i952MsIOMfD2j0Ots",
                    "y": "AEJipR0Dc-aBZYDqN51SKHYSWs9hM58SmRY1MxgXANgZrPaq1EeGMGOjkbLMEJtBThdjXhkS5VlXMkF0cYhZELiH",
                })
            },
        };
    pub static ref BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_NOT_IN_SECRETS_1: VerificationMethod =
        VerificationMethod {
            id: "did:example:bob#key-p521-not-secrets-1".into(),
            controller: "did:example:bob#key-p521-not-secrets-1".into(),
            type_: VerificationMethodType::JsonWebKey2020,
            verification_material: VerificationMaterial::JWK {
                public_key_jwk: json!(
                {
                    "kty": "EC",
                    "crv": "P-521",
                    "x": "ATp_WxCfIK_SriBoStmA0QrJc2pUR1djpen0VdpmogtnKxJbitiPq-HJXYXDKriXfVnkrl2i952MsIOMfD2j0Ots",
                    "y": "AEJipR0Dc-aBZYDqN51SKHYSWs9hM58SmRY1MxgXANgZrPaq1EeGMGOjkbLMEJtBThdjXhkS5VlXMkF0cYhZELiH",
                })
            },
        };
    pub static ref BOB_DID_COMM_MESSAGING_SERVICE: DIDCommMessagingService =
        DIDCommMessagingService {
            uri: "http://example.com/path".into(),
            accept: Some(vec!["didcomm/v2".into(), "didcomm/aip2;env=rfc587".into()]),
            routing_keys: vec!["did:example:mediator1#key-x25519-1".into()],
        };
    pub static ref BOB_SERVICE: Service = Service {
        id: "did:example:bob#didcomm-1".into(),
        service_endpoint: ServiceKind::DIDCommMessaging {
            value: BOB_DID_COMM_MESSAGING_SERVICE.clone()
        },
    };
    pub static ref BOB_DID_DOC: DIDDoc = DIDDoc {
        id: "did:example:bob".into(),
        authentication: vec![],
        key_agreement: vec![
            "did:example:bob#key-x25519-1".into(),
            "did:example:bob#key-x25519-2".into(),
            "did:example:bob#key-x25519-3".into(),
            "did:example:bob#key-p256-1".into(),
            "did:example:bob#key-p256-2".into(),
            "did:example:bob#key-p384-1".into(),
            "did:example:bob#key-p384-2".into(),
            "did:example:bob#key-p521-1".into(),
            "did:example:bob#key-p521-2".into(),
        ],
        service: vec![BOB_SERVICE.clone()],
        verification_method: vec![
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_3.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_2.clone(),
        ],
    };
    pub static ref BOB_DID_DOC_NO_SECRETS: DIDDoc = DIDDoc {
        id: "did:example:bob".into(),
        authentication: vec![],
        key_agreement: vec![
            "did:example:bob#key-x25519-1".into(),
            "did:example:bob#key-x25519-2".into(),
            "did:example:bob#key-x25519-3".into(),
            "did:example:bob#key-x25519-not-secrets-1".into(),
            "did:example:bob#key-p256-1".into(),
            "did:example:bob#key-p256-2".into(),
            "did:example:bob#key-p256-not-secrets-1".into(),
            "did:example:bob#key-p384-1".into(),
            "did:example:bob#key-p384-2".into(),
            "did:example:bob#key-p384-not-secrets-1".into(),
            "did:example:bob#key-p521-1".into(),
            "did:example:bob#key-p521-2".into(),
            "did:example:bob#key-p521-not-secrets-1".into(),
        ],
        service: vec![BOB_SERVICE.clone()],
        verification_method: vec![
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_3.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_X25519_NOT_IN_SECRETS_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P256_NOT_IN_SECRETS_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P384_NOT_IN_SECRETS_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_1.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_2.clone(),
            BOB_VERIFICATION_METHOD_KEY_AGREEM_P521_NOT_IN_SECRETS_1.clone(),
        ],
    };

    pub static ref MEDIATOR_DID_DOC: DIDDoc = serde_json::from_str(
        r#"{
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

    pub static ref MEDIATOR_KEY: Vec<Secret> = vec![
        Secret {
            id: "did:web:alice-mediator.com:alice_mediator_pub#keys-3".into(),
            type_: SecretType::JsonWebKey2020,
            secret_material: SecretMaterial::JWK {
                private_key_jwk: json!({
                    "kty": "OKP",
                    "crv": "X25519",
                    "x": "SHSUZ6V3x355FqCzIUfgoPzrZB0BQs0JKyag4UfMqHQ",
                    "d": "0A8SSFkGHg3N9gmVDRnl63ih5fcwtEvnQu9912SVplY"
                })
            },
        }
    ];
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_X25519_1: Secret = Secret {
        id: "did:example:bob#key-x25519-1".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "OKP",
                "d": "b9NnuOCB0hm7YGNvaE9DMhwH_wjZA1-gWD6dA0JWdL0",
                "crv": "X25519",
                "x": "GDTrI66K0pFfO54tlCSvfjjNapIs44dzpneBgyx0S3E",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_X25519_2: Secret = Secret {
        id: "did:example:bob#key-x25519-2".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "OKP",
                "d": "p-vteoF1gopny1HXywt76xz_uC83UUmrgszsI-ThBKk",
                "crv": "X25519",
                "x": "UT9S3F5ep16KSNBBShU2wh3qSfqYjlasZimn0mB8_VM",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_X25519_3: Secret = Secret {
        id: "did:example:bob#key-x25519-3".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "OKP",
                "d": "f9WJeuQXEItkGM8shN4dqFr5fLQLBasHnWZ-8dPaSo0",
                "crv": "X25519",
                "x": "82k2BTUiywKv49fKLZa-WwDi8RBf0tB0M8bvSAUQ3yY",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P256_1: Secret = Secret {
        id: "did:example:bob#key-p256-1".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "PgwHnlXxt8pwR6OCTUwwWx-P51BiLkFZyqHzquKddXQ",
                "crv": "P-256",
                "x": "FQVaTOksf-XsCUrt4J1L2UGvtWaDwpboVlqbKBY2AIo",
                "y": "6XFB9PYo7dyC5ViJSO9uXNYkxTJWn0d_mqJ__ZYhcNY",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P256_2: Secret = Secret {
        id: "did:example:bob#key-p256-2".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "agKz7HS8mIwqO40Q2dwm_Zi70IdYFtonN5sZecQoxYU",
                "crv": "P-256",
                "x": "n0yBsGrwGZup9ywKhzD4KoORGicilzIUyfcXb1CSwe0",
                "y": "ov0buZJ8GHzV128jmCw1CaFbajZoFFmiJDbMrceCXIw",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P384_1: Secret = Secret {
        id: "did:example:bob#key-p384-1".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "ajqcWbYA0UDBKfAhkSkeiVjMMt8l-5rcknvEv9t_Os6M8s-HisdywvNCX4CGd_xY",
                "crv": "P-384",
                "x": "MvnE_OwKoTcJVfHyTX-DLSRhhNwlu5LNoQ5UWD9Jmgtdxp_kpjsMuTTBnxg5RF_Y",
                "y": "X_3HJBcKFQEG35PZbEOBn8u9_z8V1F9V1Kv-Vh0aSzmH-y9aOuDJUE3D4Hvmi5l7",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P384_2: Secret = Secret {
        id: "did:example:bob#key-p384-2".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "OiwhRotK188BtbQy0XBO8PljSKYI6CCD-nE_ZUzK7o81tk3imDOuQ-jrSWaIkI-T",
                "crv": "P-384",
                "x": "2x3HOTvR8e-Tu6U4UqMd1wUWsNXMD0RgIunZTMcZsS-zWOwDgsrhYVHmv3k_DjV3",
                "y": "W9LLaBjlWYcXUxOf6ECSfcXKaC3-K9z4hCoP0PS87Q_4ExMgIwxVCXUEB6nf0GDd",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P521_1: Secret = Secret {
        id: "did:example:bob#key-p521-1".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "AV5ocjvy7PkPgNrSuvCxtG70NMj6iTabvvjSLbsdd8OdI9HlXYlFR7RdBbgLUTruvaIRhjEAE9gNTH6rWUIdfuj6",
                "crv": "P-521",
                "x": "Af9O5THFENlqQbh2Ehipt1Yf4gAd9RCa3QzPktfcgUIFADMc4kAaYVViTaDOuvVS2vMS1KZe0D5kXedSXPQ3QbHi",
                "y": "ATZVigRQ7UdGsQ9j-omyff6JIeeUv3CBWYsZ0l6x3C_SYqhqVV7dEG-TafCCNiIxs8qeUiXQ8cHWVclqkH4Lo1qH",
            })
        },
    };
    pub static ref BOB_SECRET_KEY_AGREEMENT_KEY_P521_2: Secret = Secret {
        id: "did:example:bob#key-p521-2".into(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: json!(
            {
                "kty": "EC",
                "d": "ABixMEZHsyT7SRw-lY5HxdNOofTZLlwBHwPEJ3spEMC2sWN1RZQylZuvoyOBGJnPxg4-H_iVhNWf_OtgYODrYhCk",
                "crv": "P-521",
                "x": "ATp_WxCfIK_SriBoStmA0QrJc2pUR1djpen0VdpmogtnKxJbitiPq-HJXYXDKriXfVnkrl2i952MsIOMfD2j0Ots",
                "y": "AEJipR0Dc-aBZYDqN51SKHYSWs9hM58SmRY1MxgXANgZrPaq1EeGMGOjkbLMEJtBThdjXhkS5VlXMkF0cYhZELiH",
            })
        },
    };
    pub static ref BOB_SECRETS: Vec<Secret> = vec![
        BOB_SECRET_KEY_AGREEMENT_KEY_X25519_1.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_X25519_2.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_X25519_3.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P256_1.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P256_2.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P384_1.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P384_2.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P521_1.clone(),
        BOB_SECRET_KEY_AGREEMENT_KEY_P521_2.clone(),
    ];
    
}
pub fn _sender_secrets_resolver() -> impl SecretsResolver {
    let secret_id = BOB_DID.to_owned() + "#z6LSiZbfm5L5zR3mrqpHyL7T2b2x3afUMpmGnMrEQznAz5F3";
    let secret: Jwk = serde_json::from_str(
        r#"{
            "kty": "OKP",
            "crv": "X25519",
            "x": "ZlJzHqy2dLrDQNlV15O3zDOIXpWVQnq6VtiVZ78O0hY",
            "d": "8OK7-1IVMdcM86PZzYKsbIi3kCJ-RxI8XFKe9JEcF2Y"
        }"#,
    )
    .unwrap();

    LocalSecretsResolver::new(&secret_id, &secret)
}
