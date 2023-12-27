use serde_json::{Map, Value};

pub fn abbreviate_service_for_did_peer_2(value: &mut Value) {
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

    abbrv_service(value, &abbrv_key, &abbrv_val)
}

pub fn reverse_abbreviate_service_for_did_peer_2(value: &mut Value) {
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

    abbrv_service(value, &rev_abbrv_key, &rev_abbrv_val)
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
    use serde_json::json;

    use super::*;

    #[test]
    fn test_abbreviate_service_for_did_peer_2() {
        let mut service = json!({
            "type": "DIDCommMessaging",
            "serviceEndpoint": {
                "uri": "http://example.com/didcomm",
                "accept": [
                    "didcomm/v2"
                ],
                "routingKeys": [
                    "did:example:123456789abcdefghi#key-1"
                ]
            }
        });

        abbreviate_service_for_did_peer_2(&mut service);

        assert_eq!(
            service,
            json!({
                "t": "dm",
                "s": {
                    "uri": "http://example.com/didcomm",
                    "a": [
                        "didcomm/v2"
                    ],
                    "r": [
                        "did:example:123456789abcdefghi#key-1"
                    ]
                }
            })
        );
    }

    #[test]
    fn test_abbreviate_service_for_did_peer_2_with_pushed_boundaries() {
        let mut service = json!({
            "type": "DIDCommMessaging",
            "DIDCommMessaging": "DIDCommMessaging",
            "serviceEndpoint": {
                "uri": "routingKeys",
                "accept": [
                    "didcomm/v2",
                    "type"
                ],
                "routingKeys": [
                    "did:example:123456789abcdefghi#key-1"
                ]
            }
        });

        abbreviate_service_for_did_peer_2(&mut service);

        assert_eq!(
            service,
            json!({
                "t": "dm",
                "DIDCommMessaging": "dm",
                "s": {
                    "uri": "routingKeys",
                    "a": [
                        "didcomm/v2",
                        "type"
                    ],
                    "r": [
                        "did:example:123456789abcdefghi#key-1"
                    ]
                }
            })
        );
    }

    #[test]
    fn test_reverse_abbreviate_service_for_did_peer_2() {
        let mut service = json!({
            "t": "dm",
            "s": {
                "uri": "http://example.com/didcomm",
                "a": [
                    "didcomm/v2"
                ],
                "r": [
                    "did:example:123456789abcdefghi#key-1"
                ]
            }
        });

        reverse_abbreviate_service_for_did_peer_2(&mut service);

        assert_eq!(
            service,
            json!({
                "type": "DIDCommMessaging",
                "serviceEndpoint": {
                    "uri": "http://example.com/didcomm",
                    "accept": [
                        "didcomm/v2"
                    ],
                    "routingKeys": [
                        "did:example:123456789abcdefghi#key-1"
                    ]
                }
            })
        );
    }
}
