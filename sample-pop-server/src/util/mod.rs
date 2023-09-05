#![allow(unused)]

pub mod didweb;
pub mod resolver;

mod keystore;
pub use keystore::KeyStore;

pub fn crate_name() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let basename = current_dir.file_name().unwrap().to_str().unwrap();
    basename.to_string()
}
