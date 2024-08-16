#![allow(unused)]

pub fn crate_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_default()
}
