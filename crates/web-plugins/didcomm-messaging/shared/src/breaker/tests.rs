use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::sleep;

#[tokio::test]
async fn test_configuration_overrides() {
    let breaker = CircuitBreaker::new()
        .retries(2)
        .retries(3)
        .reset_timeout(Duration::from_millis(100))
        .reset_timeout(Duration::from_secs(1))
        .constant_backoff(Duration::from_millis(200))
        .exponential_backoff(Duration::from_millis(100))
        .exponential_backoff(Duration::from_millis(200))
        .half_open_max_failures(5)
        .half_open_max_failures(10);

    assert_eq!(breaker.inner.lock().max_retries, 3);
    assert_eq!(breaker.inner.lock().reset_timeout, Duration::from_secs(1));
    assert_eq!(
        breaker.inner.lock().backoff,
        BackoffStrategy::Exponential(Duration::from_millis(200))
    );
    assert_eq!(breaker.inner.lock().half_open_max_retries, 10);
}

#[tokio::test]
async fn test_default_config_with_successful_future() {
    let breaker = CircuitBreaker::new();
    assert!(breaker.should_allow_call());
    let result = breaker.call(|| async { Ok::<_, ()>(1) }).await;
    assert_eq!(breaker.inner.lock().state, CircuitState::Closed);
    assert_eq!(breaker.inner.lock().failure_count, 0);
    assert_eq!(result, Ok(1));
}

#[tokio::test]
async fn test_default_config_with_failed_future() {
    let breaker = CircuitBreaker::new();
    assert!(breaker.should_allow_call());
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(matches!(result, Err(Error::Inner(_))));
    assert!(!breaker.should_allow_call());
}

#[tokio::test]
async fn test_circuit_open_rejection() {
    let breaker = CircuitBreaker::new();

    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(!breaker.should_allow_call());

    // The circuit is now open, so the call should be rejected
    let result = breaker.call(|| async { Ok::<_, ()>(1) }).await;
    assert!(matches!(result, Err(Error::CircuitOpen)));
}

#[tokio::test]
async fn test_retry_configuration() {
    let breaker = CircuitBreaker::new().retries(2);

    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let retry_operation = || {
        let attempts = attempts_clone.clone();
        async move {
            attempts.fetch_add(1, Ordering::AcqRel);
            Err::<(), ()>(()) // Simulate a failure
        }
    };

    let result = breaker.call(retry_operation).await;

    // Verify the result matches the expected error type
    assert!(matches!(result, Err(Error::Inner(_))));
    // Verify that the total number of attempts was 2
    assert_eq!(attempts.load(Ordering::Relaxed), 2);
}


#[tokio::test]
async fn test_timeout_reset() {
    let breaker = CircuitBreaker::new().reset_timeout(Duration::from_millis(100));

    // Open the circuit
    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(!breaker.should_allow_call());

    // Wait for reset timeout
    sleep(Duration::from_millis(100)).await;

    // At this level the circuit should be in half-open state
    let result = breaker.call(|| async { Ok::<_, ()>(1) }).await;
    // Circuit should be closed after successful call
    assert_eq!(result, Ok(1));
    assert_eq!(breaker.inner.lock().state, CircuitState::Closed);
}

#[tokio::test]
async fn test_exponential_backoff() {
    let breaker = CircuitBreaker::new()
        .retries(3)
        .exponential_backoff(Duration::from_millis(100));

    let start = Instant::now();
    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    let elapsed = start.elapsed();

    // After the first failure, we wait 100ms before retrying
    // After the second failure, we wait 200ms before retrying
    // The total elapsed time should be near 300ms
    assert!(elapsed >= Duration::from_millis(300) && elapsed < Duration::from_millis(400));
    assert!(!breaker.should_allow_call());
}

#[tokio::test]
async fn test_constant_backoff() {
    let breaker = CircuitBreaker::new()
        .retries(3)
        .constant_backoff(Duration::from_millis(100));

    let start = Instant::now();
    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    let elapsed = start.elapsed();

    // We wait 100ms after the first failure
    // We wait for another 100ms after the second failure
    // After the third failure, the total elapsed time should be near 200ms
    assert!(elapsed >= Duration::from_millis(200) && elapsed < Duration::from_millis(300));
    assert!(!breaker.should_allow_call());
}

#[tokio::test]
async fn test_half_open_state_failure() {
    let breaker = CircuitBreaker::new()
        .retries(1)
        .reset_timeout(Duration::from_millis(100));

    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(!breaker.should_allow_call());

    // We wait for reset timeout
    sleep(Duration::from_millis(150)).await;

    // The circuit should be in half-open state
    assert!(breaker.should_allow_call());
    assert_eq!(breaker.inner.lock().state, CircuitState::HalfOpen);

    // The circuit should open again after the next failure
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert_eq!(result, Err(Error::CircuitOpen));
}

#[tokio::test]
async fn test_half_open_multiple_failures_allowed() {
    let breaker = CircuitBreaker::new()
        .half_open_max_failures(2)
        .reset_timeout(Duration::from_millis(100));

    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(matches!(result, Err(Error::Inner(_))));
    assert_eq!(breaker.inner.lock().state, CircuitState::Open);

    sleep(Duration::from_millis(100)).await;

    // The circuit should be in half-open state
    // and should allow 2 more failures
    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(matches!(result, Err(Error::Inner(_))));

    // Additional failure in half-open should open circuit
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(matches!(result, Err(Error::CircuitOpen)));
}
