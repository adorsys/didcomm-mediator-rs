#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future, pin::Pin, sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        }
    };
    use tokio::time::sleep;

    type FailingOperationFuture = Pin<Box<dyn Future<Output = Result<(), &'static str>>>>;
    // Helper function to create a failing operation that counts calls
    fn make_failing_operation() -> (
        Arc<AtomicUsize>,
        impl Fn() -> FailingOperationFuture,
    ) {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let operation = move || {
            let counter = counter_clone.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err("operation failed")
            }
        };
        (counter, operation)
    }

    // Helper function to create an operation that fails N times then succeeds
    fn make_eventually_successful_operation(
        failures: usize,
    ) -> impl Fn() -> impl Future<Output = Result<(), &'static str>> {
        let attempts = Arc::new(AtomicUsize::new(0));
        move || {
            let attempts = attempts.clone();
            async move {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current < failures {
                    Err("operation failed")
                } else {
                    Ok(())
                }
            }
        }
    }

    #[tokio::test]
    async fn test_default_configuration() {
        let breaker = CircuitBreaker::new();
        let (counter, operation) = make_failing_operation();

        // With default configuration (0 retries), should fail immediately
        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::Inner(_))));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let breaker = CircuitBreaker::new().retries(3);
        let (counter, operation) = make_failing_operation();

        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::Inner(_))));
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Should try 3 times
    }

    #[tokio::test]
    async fn test_eventual_success() {
        let breaker = CircuitBreaker::new().retries(5);
        let operation = make_eventually_successful_operation(2);

        let result = breaker.call(operation).await;
        assert!(matches!(result, Ok(())));
    }

    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let breaker = CircuitBreaker::new().retries(2);
        let (counter, operation) = make_failing_operation();

        // First call should fail after retries
        let result = breaker.call(operation.clone()).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        // Second call should find circuit open
        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::CircuitOpen)));

        // Should only have been called during first attempt
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_half_open_state() {
        let breaker = CircuitBreaker::new()
            .retries(1)
            .reset_timeout(Duration::from_millis(100));

        let operation = make_eventually_successful_operation(1);

        // First call fails and opens circuit
        let result = breaker.call(operation.clone()).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        // Wait for reset timeout
        sleep(Duration::from_millis(150)).await;

        // Next call should succeed and close circuit
        let result = breaker.call(operation).await;
        assert!(matches!(result, Ok(())));
    }

    #[tokio::test]
    async fn test_constant_backoff() {
        let start = Instant::now();
        let breaker = CircuitBreaker::new()
            .retries(2)
            .constant_backoff(Duration::from_millis(100));

        let (counter, operation) = make_failing_operation();

        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(200)); // Should wait 100ms between each retry
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let start = Instant::now();
        let breaker = CircuitBreaker::new()
            .retries(3)
            .exponential_backoff(Duration::from_millis(100));

        let (counter, operation) = make_failing_operation();

        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        let elapsed = start.elapsed();
        // Should wait: 100ms + 200ms + 400ms = 700ms total
        assert!(elapsed >= Duration::from_millis(700));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_successful_reset() {
        let breaker = CircuitBreaker::new().retries(2);

        // Create an operation that succeeds
        let operation = || async { Ok::<(), &'static str>(()) };

        // First call succeeds
        let result = breaker.call(operation.clone()).await;
        assert!(matches!(result, Ok(())));

        // Circuit should remain closed
        assert!(breaker.should_allow_call());

        // Second call should also succeed
        let result = breaker.call(operation).await;
        assert!(matches!(result, Ok(())));
    }

    #[tokio::test]
    async fn test_half_open_failure() {
        let breaker = CircuitBreaker::new()
            .retries(1)
            .reset_timeout(Duration::from_millis(100));

        let (counter, operation) = make_failing_operation();

        // First call fails and opens circuit
        let result = breaker.call(operation.clone()).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        // Wait for reset timeout
        sleep(Duration::from_millis(150)).await;

        // Next call should fail and reopen circuit
        let result = breaker.call(operation).await;
        assert!(matches!(result, Err(Error::Inner(_))));

        // Circuit should be open again
        assert!(!breaker.should_allow_call());
    }

    #[tokio::test]
    async fn test_concurrent_calls() {
        let breaker = CircuitBreaker::new().retries(2);
        let (counter, operation) = make_failing_operation();

        // Launch multiple concurrent calls
        let results = tokio::join!(
            breaker.call(operation.clone()),
            breaker.call(operation.clone()),
            breaker.call(operation.clone())
        );

        // All calls should fail
        assert!(matches!(results.0, Err(_)));
        assert!(matches!(results.1, Err(_)));
        assert!(matches!(results.2, Err(_)));

        // Circuit should be open
        assert!(!breaker.should_allow_call());
    }
}
