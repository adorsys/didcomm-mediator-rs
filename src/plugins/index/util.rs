pub(crate) fn crate_name() -> String {
    let current_dir = std::env::current_dir().unwrap();
    let basename = current_dir.file_name().unwrap().to_str().unwrap();
    basename.to_string()
}
