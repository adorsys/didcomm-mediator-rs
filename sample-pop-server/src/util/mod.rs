#![allow(unused)]

pub mod keystore;
pub use keystore::KeyStore;

use std::error::Error;
use url::{ParseError, Url};

/// Turns an HTTP(S) URL into a did:web id.
pub fn url_to_did_web_id(url: &str) -> Result<String, Box<dyn Error>> {
    let url = url.trim();

    let parsed = if url.contains("://") {
        if ["http://", "https://"].iter().all(|x| !url.starts_with(x)) {
            return Err("Scheme not allowed")?;
        }
        Url::parse(url)?
    } else {
        Url::parse(&format!("http://{url}"))?
    };

    let domain = parsed.domain().ok_or(ParseError::EmptyHost)?;

    let mut port = String::new();
    if let Some(parsed_port) = parsed.port() {
        port = format!("%3A{parsed_port}");
    }

    let mut path = parsed.path().replace('/', ":");
    if path.len() == 1 {
        // Discards single '/' character
        path = String::new();
    }

    Ok(format!("did:web:{domain}{port}{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn can_url_to_did_web_id() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            url_to_did_web_id("localhost:8080")?,
            "did:web:localhost%3A8080",
        );

        assert_eq!(
            url_to_did_web_id("https://localhost:8080")?,
            "did:web:localhost%3A8080",
        );

        assert_eq!(
            url_to_did_web_id("https://localhost:8080/user/alice")?,
            "did:web:localhost%3A8080:user:alice",
        );

        assert_eq!(
            url_to_did_web_id("https://github.com/rust-lang/rust/issues?labels=E-easy&state=open")?,
            "did:web:github.com:rust-lang:rust:issues",
        );

        assert!(url_to_did_web_id("ftp://localhost").is_err());
        assert!(url_to_did_web_id("httpss://localhost").is_err());
        assert!(url_to_did_web_id("urn:isbn:0451450523").is_err());
        assert!(url_to_did_web_id("https://127.0.0.1:8080").is_err());

        Ok(())
    }
}
