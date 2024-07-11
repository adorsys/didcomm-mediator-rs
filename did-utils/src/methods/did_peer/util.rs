use serde_json::{json, Map, Value};

use crate::didcore::Service;

use super::error::DIDPeerMethodError;

pub fn abbreviate_service_for_did_peer_2(service: &Service) -> Result<String, DIDPeerMethodError> {
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

pub fn reverse_abbreviate_service_for_did_peer_2(service: &str) -> Result<Service, DIDPeerMethodError> {
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

#[cfg(test)]
mod tests {
    // TODO! Update these tests upon revising the Service struct for compliance

    use super::*;

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
}
