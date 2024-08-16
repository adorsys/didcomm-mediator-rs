<<<<<<< HEAD:src/plugins/index/util.rs
pub(crate) fn crate_name() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let basename = current_dir.file_name().unwrap().to_str().unwrap();
    basename.to_string()
=======
#![allow(unused)]

pub fn crate_name() -> String {
    std::env::var("CARGO_PKG_NAME").unwrap_or_default()
>>>>>>> origin/main:generic-server/src/util/mod.rs
}
