use serde_json::{Value, Map};

pub fn abbreviate_service_value(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            let mut new_obj = Map::new();

            for (key, val) in obj.iter() {
                let k = match key.as_str() {
                    "type" => "t",
                    "serviceEndpoint" => "s",
                    "routingKeys" => "r",
                    "accept" => "a",
                    _ => key,
                };

                let mut v = val.clone();
                abbreviate_service_value(&mut v);

                new_obj.insert(k.to_owned(), v);
            }

            *obj = new_obj;
        }
        Value::Array(arr) => {
            for val in arr.iter_mut() {
                abbreviate_service_value(val);
            }
        }
        Value::String(val) => {
            let v = match val.as_str() {
                "DIDCommMessaging" => "dm",
                _ => val,
            };

            *val = v.to_string();
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_abbreviate_service_value() {
        let mut service = json!({
            "type": "DIDCommMessaging",
            "serviceEndpoint": "https://example.com",
            "routingKeys": ["key1", "key2"],
            "accept": true
        });

        abbreviate_service_value(&mut service);

        assert_eq!(service, json!({
            "t": "dm",
            "s": "https://example.com",
            "r": ["key1", "key2"],
            "a": true
        }));
    }
}
