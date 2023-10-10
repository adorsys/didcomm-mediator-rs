pub mod didweb;
pub mod keystore;

pub use keystore::KeyStore;

#[cfg(test)]
pub fn dotenv_flow_read(key: &str) -> Option<String> {
    dotenv_flow::dotenv_iter().unwrap().find_map(|item| {
        let (k, v) = item.unwrap();
        (k == key).then_some(v)
    })
}
