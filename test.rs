use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use uuid::Uuid;
use std::sync::Arc;

// Mediator implementation for acknowledgment and message processing
struct Mediator {
    last_ack: Mutex<Option<String>>,
    delivery_status: Mutex<Vec<(String, bool)>>,
}

impl Mediator {
    fn new() -> Self {
        Self {
            last_ack: Mutex::new(None),
            delivery_status: Mutex::new(Vec::new()),
        }
    }

    async fn send_message(&self, message: &str) -> Result<String, String> {
        let message_id = Uuid::new_v4().to_string();
        {
            let mut status = self.delivery_status.lock().await;
            status.push((message_id.clone(), false));
        }
        // Simulate delivery
        sleep(Duration::from_secs(1)).await;
        {
            let mut ack = self.last_ack.lock().await;
            *ack = Some(message_id.clone());
        }
        {
            let mut status = self.delivery_status.lock().await;
            if let Some(entry) = status.iter_mut().find(|(id, _)| *id == message_id) {
                entry.1 = true;
            }
        }
        Ok(message_id)
    }

    async fn get_last_ack(&self) -> Option<String> {
        let ack = self.last_ack.lock().await;
        ack.clone()
    }

    async fn get_delivery_status(&self, message_id: &str) -> Option<bool> {
        let status = self.delivery_status.lock().await;
        status.iter().find(|(id, _)| id == message_id).map(|(_, delivered)| *delivered)
    }
}

// Retry logic implementation
pub async fn send_message_with_retries<F, T>(
    message: &str,
    mut send_function: F,
) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let max_retries = 3;
    let mut delay = Duration::from_secs(1);

    for attempt in 1..=max_retries {
        match send_function() {
            Ok(response) => return Ok(response),
            Err(err) => {
                if attempt == max_retries {
                    return Err(format!("Failed after {} attempts: {}", max_retries, err));
                }
                eprintln!("Attempt {} failed: {}. Retrying after {:?}...", attempt, err, delay);
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
    Err("Unexpected error in retry logic".into())
}

// Tests
#[tokio::test]
async fn test_simulated_network_failure() {
    let mediator = Mediator::new();
    let result = send_message_with_retries("example-message", || {
        mediator.send_message("example-message").await
    })
    .await;

    assert!(result.is_ok(), "Expected successful message delivery after retries");
}

#[tokio::test]
async fn test_retry_logic_with_timeout() {
    let mut attempts = 0;
    let mock_network = || {
        attempts += 1;
        Err("Simulated failure".to_string())
    };

    let result = send_message_with_retries("test-message", mock_network).await;

    assert!(result.is_err(), "Expected failure after max retries");
    assert_eq!(attempts, 3, "Expected exactly 3 retry attempts");
}

#[tokio::test]
async fn test_message_acknowledgment() {
    let mediator = Mediator::new();
    let message_id = mediator.send_message("example-message").await.unwrap();
    let ack = mediator.get_last_ack().await;

    assert!(ack.is_some(), "Expected acknowledgment to be present");
    assert_eq!(ack.unwrap(), message_id, "Acknowledgment ID should match message ID");
}

