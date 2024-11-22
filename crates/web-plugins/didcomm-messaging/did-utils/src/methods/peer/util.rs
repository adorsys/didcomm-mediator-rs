use super::errors::DIDPeerMethodError;
use crate::didcore::Document as DIDDocument;
use crate::didcore::{Service, VerificationMethodType};
use serde_json::{json, Map, Value};

pub(super) fn abbreviate_service_for_did_peer_2(service: &Service) -> Result<String, DIDPeerMethodError> {
    let abbrv_key = |key: &str| -> String {
        match key {
            "type" => "t",
            "serviceEndpoint" => "s",
            "routingKeys" => "r",
            "accept" => "a",
            _ => key,
        }
        .to_string()
    };

    let abbrv_val = |val: &str| -> String {
        match val {
            "DIDCommMessaging" => "dm",
            _ => val,
        }
        .to_string()
    };

    let mut value = json!(service);
    abbrv_service(&mut value, &abbrv_key, &abbrv_val);

    Ok(json_canon::to_string(&value)?)
}

pub(super) fn reverse_abbreviate_service_for_did_peer_2(service: &str) -> Result<Service, DIDPeerMethodError> {
    let mut value = serde_json::from_str(service)?;

    let rev_abbrv_key = |key: &str| -> String {
        match key {
            "t" => "type",
            "s" => "serviceEndpoint",
            "r" => "routingKeys",
            "a" => "accept",
            _ => key,
        }
        .to_string()
    };

    let rev_abbrv_val = |val: &str| -> String {
        match val {
            "dm" => "DIDCommMessaging",
            _ => val,
        }
        .to_string()
    };

    abbrv_service(&mut value, &rev_abbrv_key, &rev_abbrv_val);
    Ok(serde_json::from_value(value)?)
}

fn abbrv_service(value: &mut Value, abbrv_key: &dyn Fn(&str) -> String, abbrv_val: &dyn Fn(&str) -> String) {
    match value {
        Value::Object(obj) => {
            let mut new_obj = Map::new();

            for (key, val) in obj.iter() {
                let k = abbrv_key(key.as_str());

                let mut v = val.clone();
                abbrv_service(&mut v, &abbrv_key, &abbrv_val);

                new_obj.insert(k, v);
            }

            *obj = new_obj;
        }
        Value::Array(arr) => {
            for val in arr.iter_mut() {
                abbrv_service(val, &abbrv_key, &abbrv_val);
            }
        }
        Value::String(val) => {
            *val = abbrv_val(val.as_str());
        }
        _ => (),
    }
}

pub(super) fn validate_input_document(diddoc: &DIDDocument) -> Result<(), DIDPeerMethodError> {
    // The document should not be empty.
    if diddoc.verification_method.is_none()
        && diddoc.authentication.is_none()
        && diddoc.assertion_method.is_none()
        && diddoc.key_agreement.is_none()
        && diddoc.capability_delegation.is_none()
        && diddoc.capability_invocation.is_none()
        && diddoc.service.is_none()
    {
        return Err(DIDPeerMethodError::InvalidStoredVariant);
    }

    // There must not be an id field at the root
    // All id fields within the document must be relative
    // All references to resources must be relative
    if !diddoc.id.is_empty() || !are_all_ids_and_references_relative(diddoc) {
        return Err(DIDPeerMethodError::InvalidStoredVariant);
    }
    Ok(())

    // If there's already a controller field in the doc it means that
    // the document is controlled by another DID so we do nothing.
}

fn are_all_ids_and_references_relative(diddoc: &DIDDocument) -> bool {
    #[inline]
    fn is_relative(item: &str) -> bool {
        item.starts_with('#')
    }

    #[inline]
    fn check_methods<T>(methods: &Option<Vec<T>>, f: impl Fn(&T) -> bool) -> bool {
        methods.as_ref().map_or(true, |items| items.iter().all(f))
    }

    check_methods(&diddoc.verification_method, |method| is_relative(&method.id))
        && check_methods(&diddoc.authentication, |auth| match auth {
            VerificationMethodType::Reference(reference) => is_relative(reference),
            VerificationMethodType::Embedded(method) => is_relative(&method.id),
        })
        && check_methods(&diddoc.assertion_method, |assert| match assert {
            VerificationMethodType::Reference(reference) => is_relative(reference),
            VerificationMethodType::Embedded(method) => is_relative(&method.id),
        })
        && check_methods(&diddoc.key_agreement, |key| match key {
            VerificationMethodType::Reference(reference) => is_relative(reference),
            VerificationMethodType::Embedded(method) => is_relative(&method.id),
        })
        && check_methods(&diddoc.capability_delegation, |delegation| match delegation {
            VerificationMethodType::Reference(reference) => is_relative(reference),
            VerificationMethodType::Embedded(method) => is_relative(&method.id),
        })
        && check_methods(&diddoc.capability_invocation, |invocation| match invocation {
            VerificationMethodType::Reference(reference) => is_relative(reference),
            VerificationMethodType::Embedded(method) => is_relative(&method.id),
        })
        && check_methods(&diddoc.service, |service| is_relative(&service.id))
}

#[cfg(test)]
mod tests {
    // TODO! Update these tests upon revising the Service struct for compliance

    use super::*;
    use crate::didcore::{Service, VerificationMethod};
    use serde_json::json;

    #[test]
    fn test_abbreviate_service_for_did_peer_2() {
        let service: Service = serde_json::from_str(
            r##"{
                "id": "#didcomm",
                "type": "DIDCommMessaging",
                "serviceEndpoint": "http://example.com/didcomm",
                "accept": ["didcomm/v2"],
                "routingKeys": ["did:example:123456789abcdefghi#key-1"]
            }"##,
        )
        .unwrap();

        assert_eq!(
            abbreviate_service_for_did_peer_2(&service).unwrap(),
            r##"{"a":["didcomm/v2"],"id":"#didcomm","r":["did:example:123456789abcdefghi#key-1"],"s":"http://example.com/didcomm","t":"dm"}"##
        );
    }

    #[test]
    fn test_abbreviate_service_for_did_peer_2_with_pushed_boundaries() {
        let service: Service = serde_json::from_str(
            r##"{
                "id": "#didcomm",
                "type": "DIDCommMessaging",
                "DIDCommMessaging": "DIDCommMessaging",
                "serviceEndpoint": "routingKeys",
                "accept": ["didcomm/v2", "type"],
                "routingKeys": ["did:example:123456789abcdefghi#key-1"]
            }"##,
        )
        .unwrap();

        assert_eq!(
            abbreviate_service_for_did_peer_2(&service).unwrap(),
            json_canon::to_string(&json!({
                "id": "#didcomm",
                "t": "dm",
                "DIDCommMessaging": "dm",
                "s": "routingKeys",
                "a": ["didcomm/v2", "type"],
                "r": ["did:example:123456789abcdefghi#key-1"]
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_reverse_abbreviate_service_for_did_peer_2() {
        let sv = r##"{"a":["didcomm/v2"],"id":"#didcomm","r":["did:example:123456789abcdefghi#key-1"],"s":"http://example.com/didcomm","t":"dm"}"##;

        let service = reverse_abbreviate_service_for_did_peer_2(sv).unwrap();

        assert_eq!(
            json!(service),
            json!({
                "id": "#didcomm",
                "type": "DIDCommMessaging",
                "serviceEndpoint": "http://example.com/didcomm",
                "accept": ["didcomm/v2"],
                "routingKeys": ["did:example:123456789abcdefghi#key-1"]
            })
        );
    }

    #[test]
    fn test_reverse_abbreviate_service_for_did_peer_2_errs_on_malformed_service() {
        // id must be a string
        let sv = r##"{"a":["didcomm/v2"],"id":[],"r":["did:example:123456789abcdefghi#key-1"],"s":"http://example.com/didcomm","t":"dm"}"##;

        let res = reverse_abbreviate_service_for_did_peer_2(sv);
        let DIDPeerMethodError::SerdeError(err) = res.unwrap_err() else {
            panic!()
        };

        assert!(err.to_string().contains("invalid type: sequence, expected a string"));
    }

    #[test]
    fn test_validate_input_document_empty() {
        let diddoc = DIDDocument {
            verification_method: None,
            authentication: None,
            key_agreement: None,
            assertion_method: None,
            service: None,
            ..DIDDocument::default()
        };
        assert!(validate_input_document(&diddoc).is_err());
    }

    #[test]
    fn test_validate_input_document_not_empty() {
        let diddoc = DIDDocument {
            service: Some(vec![Service {
                id: "#service-0".to_string(),
                ..Default::default()
            }]),
            ..DIDDocument::default()
        };
        assert!(validate_input_document(&diddoc).is_ok());
    }

    #[test]
    fn test_validate_input_document_with_id() {
        let diddoc = DIDDocument {
            id: "did:peer:123".to_string(),
            ..DIDDocument::default()
        };
        assert!(validate_input_document(&diddoc).is_err());
    }

    #[test]
    fn test_all_relative_ids_and_references() {
        let diddoc = DIDDocument {
            id: String::new(),
            verification_method: Some(vec![
                VerificationMethod {
                    id: "#key-0".to_string(),
                    ..Default::default()
                },
                VerificationMethod {
                    id: "#key-1".to_string(),
                    ..Default::default()
                },
            ]),
            authentication: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            assertion_method: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            key_agreement: Some(vec![VerificationMethodType::Reference("#key-1".to_string())]),
            capability_delegation: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            capability_invocation: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            service: Some(vec![
                Service {
                    id: "#service-0".to_string(),
                    ..Default::default()
                },
                Service {
                    id: "#service-1".to_string(),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        assert!(are_all_ids_and_references_relative(&diddoc));
    }

    #[test]
    fn test_non_relative_ids() {
        let diddoc = DIDDocument {
            verification_method: Some(vec![VerificationMethod {
                id: "did:peer:123#key-0".to_string(),
                ..Default::default()
            }]),
            authentication: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            ..Default::default()
        };

        assert!(!are_all_ids_and_references_relative(&diddoc));
    }

    #[test]
    fn test_non_relative_references() {
        let diddoc = DIDDocument {
            authentication: Some(vec![VerificationMethodType::Reference("did:example:123#key-0".to_string())]),
            ..Default::default()
        };

        assert!(!are_all_ids_and_references_relative(&diddoc));
    }

    #[test]
    fn test_mixed_relative_and_non_relative() {
        let diddoc = DIDDocument {
            verification_method: Some(vec![
                VerificationMethod {
                    id: "#key-0".to_string(),
                    ..Default::default()
                },
                VerificationMethod {
                    id: "did:example:123#key-1".to_string(),
                    ..Default::default()
                },
            ]),
            authentication: Some(vec![VerificationMethodType::Reference("#key-0".to_string())]),
            ..Default::default()
        };

        assert!(!are_all_ids_and_references_relative(&diddoc));
    }
}
