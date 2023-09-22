use std::collections::HashMap;

use url::Url;

use super::errors::DIDResolutionError;

pub type ParsedDIDUrl = (String, HashMap<String, String>, Option<String>);

/// Parses DID URL into (did, query, fragment)
pub fn parse_did_url(did_url: &str) -> Result<ParsedDIDUrl, DIDResolutionError> {
    if !did_url.starts_with("did:") || did_url.contains("%%") {
        return Err(DIDResolutionError::InvalidDidUrl);
    }

    let parts: Vec<_> = did_url.split(':').collect();
    if parts.len() < 3 {
        return Err(DIDResolutionError::InvalidDidUrl);
    }

    let url = did_url.replace(':', "%%");
    let url = Url::parse(&format!("scheme://{}", url)).map_err(|_| DIDResolutionError::InvalidDidUrl)?;
    let domain = url.domain().ok_or(DIDResolutionError::InvalidDidUrl)?;

    let did = domain.replace("%%", ":");
    let query = {
        let mut map = HashMap::new();
        for (key, value) in url.query_pairs() {
            map.insert(key.to_string(), value.to_string());
        }
        map
    };
    let fragment = url.fragment().map(|x| x.to_string());

    Ok((did, query, fragment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_url_parsing() {
        let (did, query, fragment) = parse_did_url("did:key:abcd?x=a&y=b#f").unwrap();
        assert_eq!(did, "did:key:abcd");
        assert_eq!(query.get("x").unwrap(), "a");
        assert_eq!(query.get("y").unwrap(), "b");
        assert_eq!(fragment.unwrap(), "f");

        let (did, query, fragment) = parse_did_url("did:web:example.com:a:b?m=hello+world").unwrap();
        assert_eq!(did, "did:web:example.com:a:b");
        assert_eq!(query.get("m").unwrap(), "hello world");
        assert!(fragment.is_none());
    }
}
