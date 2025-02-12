use reqwest::Client;
use reqwest::Error;

pub async fn test_retry_logic(client: &Client, url: &str) -> Result<String, Error> {
    // Sample retry logic for validation
    Ok(client.get(url).send().await?.text().await?)
}
