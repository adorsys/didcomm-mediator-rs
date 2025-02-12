#[cfg(test)]
mod tests {
    use crate::mediator::fetch_with_retries;
    use crate::tests::mock_service::setup_mock_service;
    use mockito::server_url;
    use reqwest::Client;

    #[tokio::test]
    async fn test_mediator_retries_on_upstream_failure() {
        // Setup mock upstream service
        let _mock = setup_mock_service();

        // Mediator client
        let client = Client::new();
        let url = format!("{}/upstream", server_url());

        // Test fetch with retries
        let result = fetch_with_retries(&client, &url).await;
        assert!(result.is_err(), "Expected retries to fail after all attempts");
    }

    #[tokio::test]
    async fn test_mediator_fallback_on_upstream_failure() {
        // Setup mock upstream service
        let _mock = setup_mock_service();

        // Fallback data
        let fallback_data = "Fallback response".to_string();

        // Mediator client
        let client = Client::new();
        let url = format!("{}/upstream", server_url());

        // Fetch with retries and fallback
        let result = fetch_with_retries(&client, &url)
            .await
            .unwrap_or(fallback_data.clone());

        assert_eq!(result, fallback_data, "Expected fallback response on failure");
    }
}
