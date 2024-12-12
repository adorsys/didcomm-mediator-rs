use tokio::time::{sleep, Duration};
use std::sync::Mutex;
use uuid::Uuid;

// Mediator implementation for acknowledgment
struct Mediator {
    last_ack: Mutex<Option<String>>,
}

impl Mediator {
    fn new() -> Self {
        Self {
            last_ack: Mutex::new(None),
        }
    }

    async fn send_message(&self, message: &str) -> Result<String, String> {
        let message_id = Uuid::new_v4().to_string();
        let mut ack = self.last_ack.lock().unwrap();
        *ack = Some(message_id.clone());
        Ok(message_id)
    }

    async fn get_last_ack(&self) -> Option<String> {
        let ack = self.last_ack.lock().unwrap();
        ack.clone()
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
    let retry_delay = Duration::from_secs(2);

    for attempt in 1..=max_retries {
        match send_function() {
            Ok(response) => return Ok(response),
            Err(err) => {
                if attempt == max_retries {
                    return Err(format!("Failed after {} attempts: {}", max_retries, err));
                }
                eprintln!("Attempt {} failed: {}. Retrying...", attempt, err);
                sleep(retry_delay).await;
            }
        }
    }
    Err("Unexpected error in retry logic".into())
}

// Tests
#[tokio::test]
async fn test_network_timeout() {
    let simulated_delay = Duration::from_secs(5); 
    let start_time = std::time::Instant::now();

    let result = send_message_with_retries("example-message", || {
        sleep(simulated_delay).await;
        Err("Timeout error".to_string())
    })
    .await;

    assert!(result.is_err(), "Expected timeout error");
    assert!(start_time.elapsed() < Duration::from_secs(6), "Test ran longer than expected");
}

#[tokio::test]
async fn test_retry_logic_on_intermittent_connectivity() {
    let mut retries = 0;

    let simulated_network = || {
        retries += 1;
        if retries < 3 {
            Err("Network failure".to_string())
        } else {
            Ok("Message delivered".to_string())
        }
    };

    let result = send_message_with_retries("example-message", simulated_network).await;

    assert_eq!(retries, 3, "Expected 3 retries before success");
    assert!(result.is_ok(), "Expected successful message delivery");
}

#[tokio::test]
async fn test_message_acknowledgment() {
    let mediator = Mediator::new();

    let message_id = mediator.send_message("example-message").await.unwrap();
    let ack = mediator.get_last_ack().await;

    assert!(ack.is_some(), "Expected acknowledgment to be present");
    assert_eq!(ack.unwrap(), message_id, "Acknowledgment ID should match message ID");
}

#[tokio::test]
async fn test_retry_mechanism() {
    let mut attempts = 0;

    let mock_network = || {
        attempts += 1;
        if attempts < 3 {
            Err("Simulated failure".to_string())
        } else {
            Ok("Simulated success".to_string())
        }
    };

    let result = send_message_with_retries("test-message", mock_network).await;

    assert!(result.is_ok(), "Expected success after retries");
    assert_eq!(attempts, 3, "Expected 3 attempts before success");
}

#[tokio::test]
async fn test_retry_and_acknowledgment_together() {
    let mediator = Mediator::new();
    let mut attempts = 0;

    let mock_network = || {
        attempts += 1;
        if attempts < 3 {
            Err("Simulated failure".to_string())
        } else {
            mediator.send_message("example-message")
        }
    };

    let result = send_message_with_retries("example-message", mock_network).await;

    assert!(result.is_ok(), "Message should succeed after retries");
    assert!(mediator.get_last_ack().await.is_some(), "Acknowledgment should be present");
}
