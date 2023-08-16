fn main() {}

#[cfg(test)]
mod tests {
    #[test]
    fn can_jcs_serialize() {
        let data = serde_json::json!({
            "from_account": "543 232 625-3",
            "to_account": "321 567 636-4",
            "amount": 500.50,
            "currency": "USD"
        });

        let jcs = r#"{"amount":500.5,"currency":"USD","from_account":"543 232 625-3","to_account":"321 567 636-4"}"#;

        assert_eq!(jcs, json_canon::to_string(&data).unwrap());
    }
}
