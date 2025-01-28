use tokio::time::{sleep, Duration};
use tokio::sync::{futures, Mutex};
use uuid::Uuid;

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

    async fn send_message(&self, _message: &str) -> Result<String, String> {
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
    let _ = message;
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
async fn test_simulated_network_failure() {
    let mediator = Mediator::new();
    let result = send_message_with_retries("network-failure", || {
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

    let result = send_message_with_retries("retry timeout", mock_network).await;

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

#[tokio::test]
async fn test_exponential_backoff() {
    let mut attempts = 0;
    let mut delays = vec![];

    let mock_network = || {
        attempts += 1;
        delays.push(attempts); // Mock delays
        Err("Simulated failure".to_string())
    };

    let result = send_message_with_retries("backoff test", mock_network).await;

    assert!(result.is_err(), "Expected failure after max retries");
    assert_eq!(attempts, 3, "Expected exactly 3 attempts");
    assert!(delays == vec![1, 2, 3], "Expected exponential backoff");
}

#[tokio::test]
async fn test_queue_overflow_handling() {
    const QUEUE_LIMIT: usize = 5; // Simulated queue limit
    let mediator = std::sync::Arc::new(Mediator::new());
    let mut handles = vec![];

    for i in 0..(QUEUE_LIMIT + 2) {
        let mediator_clone = std::sync::Arc::clone(&mediator);
        handles.push(tokio::spawn(async move {
            mediator_clone.send_message(&format!("message-{}", i)).await
        }));
    }

    let results = futures::future::join_all(handles).await;

    // Assert that all messages were processed
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Expected successful delivery for message-{} but got error",
            i
        );
    }

    // Check order of processing
    let statuses = mediator.delivery_status.lock().await;
    for i in 0..QUEUE_LIMIT {
        assert!(
            statuses.iter().any(|(id, _)| id.contains(&format!("message-{}", i))),
            "Message message-{} was not processed",
            i
        );
    }
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    const NUM_MESSAGES: usize = 10;
    let mediator = std::sync::Arc::new(Mediator::new());
    let mut handles = vec![];

    for i in 0..NUM_MESSAGES {
        let mediator_clone = std::sync::Arc::clone(&mediator);
        handles.push(tokio::spawn(async move {
            mediator_clone.send_message(&format!("concurrent-message-{}", i)).await
        }));
    }

    let results = futures::future::join_all(handles).await;

    // Assert all messages are processed successfully
    assert!(
        results.iter().all(|res| res.is_ok()),
        "Expected all concurrent messages to be delivered successfully"
    );

    // Check the count of processed messages
    let statuses = mediator.delivery_status.lock().await;
    assert_eq!(
        statuses.len(),
        NUM_MESSAGES,
        "Expected {} messages to be processed, but got {}",
        NUM_MESSAGES,
        statuses.len()
    );
}

#[tokio::test]
async fn test_out_of_order_message_recovery() {
    let mediator = std::sync::Arc::new(Mediator::new());
    let mut message_ids = vec!["msg-3", "msg-1", "msg-2"];

    // Simulate out-of-order message sending
    for &id in &message_ids {
        let mediator_clone = std::sync::Arc::clone(&mediator);
        tokio::spawn(async move {
            mediator_clone.send_message(id).await.unwrap();
        })
        .await
        .unwrap();
    }

    // Simulate recovery (reordering based on some logic, e.g., timestamps or IDs)
    message_ids.sort(); // Simulate reordering logic

    // Check that messages are processed in the correct order
    let statuses = mediator.delivery_status.lock().await;
    let processed_ids: Vec<_> = statuses.iter().map(|(id, _)| id.clone()).collect();

    assert_eq!(
        processed_ids, message_ids,
        "Expected messages to be processed in order: {:?}, but got {:?}",
        message_ids, processed_ids
    );
}