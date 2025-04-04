#![allow(clippy::doc_overindented_list_items)]
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
    let attempts = attempts.clone();

    let retry_operation = || async {
        attempts.fetch_add(1, Ordering::AcqRel);
        async { Err::<(), ()>(()) }.await
    };

    let result = breaker.call(retry_operation).await;
    assert!(matches!(result, Err(Error::Inner(_))));
    // the total number of attempts should be 3 (initial attempt + 2 retries)
    assert_eq!(attempts.load(Ordering::Relaxed), 3);
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

    // After the first failure, we wait 100ms before retrying (first retry)
    // After the second failure, we wait 200ms before retrying (second retry)
    // After the third failure, we wait 400ms before retrying (third retry)
    // The total elapsed time should be near 700ms
    assert!(elapsed >= Duration::from_millis(700) && elapsed < Duration::from_millis(800));
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

    // We wait 100ms after the first failure (first retry)
    // We wait for another 100ms after the second failure (second retry)
    // We wait for another 100ms after the third failure (third retry)
    // After the third failure, the total elapsed time should be near 300ms
    assert!(elapsed >= Duration::from_millis(300) && elapsed < Duration::from_millis(400));
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
    sleep(Duration::from_millis(100)).await;

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
    assert!(breaker.should_allow_call());
    assert_eq!(breaker.inner.lock().state, CircuitState::HalfOpen);
    // and should allow 2 more failures
    let _ = breaker.call(|| async { Err::<(), ()>(()) }).await;
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert_eq!(breaker.inner.lock().failure_count, 2);
    assert!(matches!(result, Err(Error::Inner(_))));

    // At this point, the circuit should be open again and should reject the call
    let result = breaker.call(|| async { Err::<(), ()>(()) }).await;
    assert!(matches!(result, Err(Error::CircuitOpen)));
}
