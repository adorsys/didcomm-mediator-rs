use std::collections::HashMap;

use url::Url;

use super::errors::DIDResolutionError;

pub type ParsedDIDUrl = (String, HashMap<String, String>, Option<String>);

/// Parses DID URL into (did, query, fragment)
pub fn parse_did_url(did_url: &str) -> Result<ParsedDIDUrl, DIDResolutionError> {
    if !did_url.starts_with("did:") {
        return Err(DIDResolutionError::InvalidDidUrlPrefix);
    }

    if did_url.contains("%%") {
        return Err(DIDResolutionError::InvalidDidUrlFormat);
    }

    if did_url.split(':').filter(|x| !x.is_empty()).count() < 3 {
        return Err(DIDResolutionError::DidUrlPartLengthTooShort);
    }

    let url = format!("scheme://{}", did_url.replace(':', "%%"));
    let url = Url::parse(&url).map_err(|_| DIDResolutionError::InvalidDidUrlFormat)?;
    let domain = url.domain().ok_or(DIDResolutionError::InvalidDidUrlFormat)?;

    let did = domain.replace("%%", ":");
    if did.split(':').filter(|x| !x.is_empty()).count() < 3 {
        return Err(DIDResolutionError::DidUrlPartLengthTooShort);
    }

    let query = url.query_pairs().map(|(key, val)| (key.to_string(), val.to_string())).collect();
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

    #[test]
    fn test_did_url_parsing_fails_as_expected() {
        let entries = [
            ("dxd:key:abcd", DIDResolutionError::InvalidDidUrlPrefix),
            ("did:key:a%%d", DIDResolutionError::InvalidDidUrlFormat),
            ("did:key", DIDResolutionError::DidUrlPartLengthTooShort),
            ("did:key:", DIDResolutionError::DidUrlPartLengthTooShort),
            ("did:key:?k=v", DIDResolutionError::DidUrlPartLengthTooShort),
            ("did:key:ab\\cd", DIDResolutionError::InvalidDidUrlFormat),
            ("did:key:abcd|80", DIDResolutionError::InvalidDidUrlFormat),
            ("did:key:abcd[80]", DIDResolutionError::InvalidDidUrlFormat),
        ];

        for (did_url, err) in entries {
            assert_eq!(parse_did_url(did_url).unwrap_err(), err);
        }
    }
}
