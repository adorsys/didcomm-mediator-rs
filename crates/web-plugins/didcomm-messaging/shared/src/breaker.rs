#[cfg(test)]
mod tests;

use futures_core::TryFuture;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    task::{Context, Poll},
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::time::Sleep;

/// # Circuit Breaker
///
/// This struct implements a circuit breaker pattern, which helps prevent cascading failures when interacting with unreliable services.
/// It monitors the success and failure of operations and transitions between three states:
///
/// *   **Closed:** The circuit is operating normally, and operations are allowed to proceed.
/// *   **Open:** Operations are immediately rejected without being executed. This prevents overloading the failing service.
/// *   **Half-Open:** After a timeout period, the circuit enters a half-open state, allowing a single operation to attempt execution.
///       If the probe succeeds, the circuit closes; otherwise, it returns to the open state.
///
/// By default, the circuit breaker is configured with the following:
///
/// *   A single retry attempt.
/// *   A default reset timeout of 5 seconds.
/// *   No delay between retries.
///
/// # Configuration
///
/// The behavior of the circuit breaker can be customized using the following builder methods:
///
/// *   [`retries(self, max_retries: usize)`](retries): Sets the maximum number of consecutive failures allowed before the circuit opens.
///     A value of 0 means the circuit will open on the first failure.
/// *   [`reset_timeout(self, reset_timeout: Duration)`](timeout): Sets the duration the circuit remains open after tripping.
///     After this timeout, the circuit transitions to the half-open state.
/// *   [`exponential_backoff(self, initial_delay: Duration)`](exponential_backoff): Configures an exponential backoff strategy for retries.
///     The delay between retries increases exponentially. This overrides any previously set backoff.
/// *   [`constant_backoff(self, delay: Duration)`](constant_backoff): Configures a constant backoff strategy for retries.
///
/// # Example
///
/// ```rust
/// # use breaker::CircuitBreaker;
/// # use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), std::io::Error> {
///     let breaker = CircuitBreaker::new()
///         .retries(3)
///         .reset_timeout(Duration::from_secs(10))
///         .exponential_backoff(Duration::from_millis(100));
///
///     async fn operation() -> Result<(), std::io::Error> {
///         // The operation logic here
///         Ok(())
///     }
///
///     match breaker.call(operation).await {
///         Ok(_) => println!("Operation succeeded!"),
///         Err(e) if e == breaker::Error::CircuitOpen => {
///             println!("Circuit breaker is open!");
///         }
///         Err(e) => {
///             println!("Operation failed: {}", e);
///         }
///     }
/// #   Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CircuitBreaker {
    // Current state of the circuit
    state: AtomicU32,
    // Failure threshold before opening the circuit
    max_retries: usize,
    // Time to wait before closing the circuit again
    reset_timeout: Duration,
    // Tracks failure count
    failure_count: AtomicUsize,
    // Timestamp when circuit was opened
    opened_at: AtomicU64,
    // Retry config
    backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackoffStrategy {
    NoBackoff,
    Constant(Duration),
    Exponential(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Error)]
pub enum Error<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error(transparent)]
    Inner(#[from] E),
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new() -> Self {
        CircuitBreaker {
            state: AtomicU32::new(CircuitState::Closed as u32),
            max_retries: 0,
            reset_timeout: Duration::from_secs(5),
            failure_count: AtomicUsize::new(0),
            opened_at: AtomicU64::new(0),
            backoff: BackoffStrategy::NoBackoff,
        }
    }

    /// Specify the maximum number of attempts for the future
    pub fn retries(self, max_retries: usize) -> Self {
        Self {
            max_retries,
            ..self
        }
    }

    /// Specify the duration to wait before closing the circuit again
    pub fn reset_timeout(self, reset_timeout: Duration) -> Self {
        Self {
            reset_timeout,
            ..self
        }
    }

    /// Set the exponential backoff strategy with the given initial delay for the retry
    ///
    /// The delay will be doubled after each retry
    pub fn exponential_backoff(self, initial_delay: Duration) -> Self {
        Self {
            backoff: BackoffStrategy::Exponential(initial_delay),
            ..self
        }
    }

    /// Set the fixed backoff strategy with the given delay for the retry
    pub fn constant_backoff(self, delay: Duration) -> Self {
        Self {
            backoff: BackoffStrategy::Constant(delay),
            ..self
        }
    }

    fn state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => unreachable!(),
        }
    }

    fn success(&self) {
        match self.state() {
            CircuitState::HalfOpen => {
                self.state
                    .store(CircuitState::Closed as u32, Ordering::SeqCst);
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {}
        }
    }

    fn failure(&self) {
        let now = Instant::now().elapsed().as_secs();

        match self.state() {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.max_retries {
                    self.state
                        .store(CircuitState::Open as u32, Ordering::SeqCst);
                    self.opened_at.store(now, Ordering::SeqCst);
                }
            }
            CircuitState::HalfOpen => {
                self.state
                    .store(CircuitState::Open as u32, Ordering::SeqCst);
                self.opened_at.store(now, Ordering::SeqCst);
            }
            CircuitState::Open => {}
        }
    }

    /// Check if the circuit breaker should allow a call
    pub fn should_allow_call(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let now = Instant::now().elapsed().as_secs();
                let opened_at = self.opened_at.load(Ordering::SeqCst);
                let elapsed = now - opened_at;
                if elapsed >= self.reset_timeout.as_secs() {
                    self.state
                        .store(CircuitState::HalfOpen as u32, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn call<F>(&self, f: F) -> ResultFuture<F>
    where
        F: TryFuture,
    {
        ResultFuture {
            future: f,
            state: RetryState::Initial,
            backoff_delay: None,
            breaker: self,
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

pin_project! {
    pub struct ResultFuture<'a, F>
    {
        #[pin]
        future: F,
        state: RetryState,
        backoff_delay: Option<Pin<Box<Sleep>>>,
        breaker: &'a CircuitBreaker,
    }
}

#[derive(Debug, Clone, Copy)]
enum RetryState {
    Initial,
    Running,
    Delaying,
    Done,
}

impl<'a, F> Future for ResultFuture<'a, F>
where
    F: TryFuture,
{
    type Output = Result<F::Ok, Error<F::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            match *this.state {
                RetryState::Initial | RetryState::Running => {
                    if !this.breaker.should_allow_call() {
                        *this.state = RetryState::Done;
                        return Poll::Ready(Err(Error::CircuitOpen));
                    }

                    match this.future.as_mut().try_poll(cx) {
                        Poll::Ready(Ok(output)) => {
                            this.breaker.success();
                            *this.state = RetryState::Done;
                            return Poll::Ready(Ok(output));
                        }
                        Poll::Ready(Err(error)) => {
                            this.breaker.failure();

                            if this.breaker.failure_count.load(Ordering::SeqCst)
                                >= this.breaker.max_retries
                            {
                                *this.state = RetryState::Done;
                                return Poll::Ready(Err(Error::Inner(error)));
                            }

                            let delay = match this.breaker.backoff {
                                BackoffStrategy::NoBackoff => Duration::ZERO,
                                BackoffStrategy::Constant(delay) => delay,
                                BackoffStrategy::Exponential(initial) => {
                                    let attempt = this.breaker.failure_count.load(Ordering::SeqCst);
                                    initial * 2u32.pow(attempt as u32)
                                }
                            };

                            let delay = tokio::time::sleep(delay);
                            *this.backoff_delay = Some(Box::pin(delay));
                            *this.state = RetryState::Delaying;
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                RetryState::Delaying => {
                    if let Some(delay) = this.backoff_delay.as_mut() {
                        match delay.as_mut().poll(cx) {
                            Poll::Ready(_) => {
                                *this.state = RetryState::Running;
                                *this.backoff_delay = None;
                                continue;
                            }
                            Poll::Pending => return Poll::Pending,
                        }
                    }
                }
                RetryState::Done => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}
