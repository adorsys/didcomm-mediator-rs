#[cfg(test)]
pub(crate) fn dotenv_flow_read(key: &str) -> Option<String> {
    dotenv_flow::from_filename_iter(".env.example")
        .unwrap()
        .find_map(|item| {
            let (k, v) = item.unwrap();
            (k == key).then_some(v)
        })
}
