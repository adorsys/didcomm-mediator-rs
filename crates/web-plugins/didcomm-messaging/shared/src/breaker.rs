#[cfg(test)]
mod tests;

// use futures_core::ready;
use parking_lot::{Mutex, RwLock};
use pin_project_lite::pin_project;
use std::{
    future::{ready, Future},
    pin::Pin,
    sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    task::{ready, Context, Poll},
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
/// *   A default reset timeout of 30 seconds.
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
/// # use shared::breaker::{CircuitBreaker, Error as BreakerError};
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
///     match breaker.call(operation()).await {
///         Ok(_) => println!("Operation succeeded!"),
///         Err(e) => match e {
///             BreakerError::CircuitOpen => println!("Circuit breaker is open"),
///             BreakerError::Inner(e) => println!("Operation failed: {e}"),
///         },
///     };
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    inner: Arc<RwLock<CircuitBreakerConfig>>,
}

#[derive(Debug)]
struct CircuitBreakerConfig {
    // Current state of the circuit
    state: CircuitState,
    // Failure threshold before opening the circuit
    max_retries: usize,
    // Time to wait before closing the circuit again
    reset_timeout: Duration,
    // Tracks failure count
    failure_count: usize,
    // Timestamp when circuit was opened
    opened_at: Arc<Mutex<Option<Instant>>>,
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
        Self {
            inner: Arc::new(RwLock::new(CircuitBreakerConfig {
                state: CircuitState::Closed,
                max_retries: 0,
                reset_timeout: Duration::from_secs(30),
                failure_count: 0,
                opened_at: Arc::new(Mutex::new(None)),
                backoff: BackoffStrategy::NoBackoff,
            })),
        }
    }

    /// Specify the maximum number of attempts for the future
    pub fn retries(self, max_retries: usize) -> Self {
        {
            let mut config = self.inner.write();
            config.max_retries = max_retries;
        }
        self
    }

    /// Specify the duration to wait before closing the circuit again
    pub fn reset_timeout(self, reset_timeout: Duration) -> Self {
        {
            let mut config = self.inner.write();
            config.reset_timeout = reset_timeout;
        }
        self
    }

    /// Set the exponential backoff strategy with the given initial delay for the retry
    ///
    /// The delay will be doubled after each retry
    pub fn exponential_backoff(self, initial_delay: Duration) -> Self {
        {
            let mut config = self.inner.write();
            config.backoff = BackoffStrategy::Exponential(initial_delay);
        }
        self
    }

    /// Set the fixed backoff strategy with the given delay for the retry
    pub fn constant_backoff(self, delay: Duration) -> Self {
        {
            let mut config = self.inner.write();
            config.backoff = BackoffStrategy::Constant(delay);
        }
        self
    }

    // fn state(&self) -> CircuitState {
    //     self.inner.read().state
    // }

    fn success(&self) {
        let state = {
            let inner = self.inner.read();
            inner.state
        };
        match state {
            CircuitState::HalfOpen => {
                let mut config = self.inner.write();
                config.state = CircuitState::Closed;
                config.failure_count = 0;
                *config.opened_at.lock() = None;
            }
            CircuitState::Closed => {
                let mut config = self.inner.write();
                config.state = CircuitState::Closed;
            }
            CircuitState::Open => {}
        }
    }

    fn failure(&self) {
        let state = {
            let inner = self.inner.read();
            inner.state
        };

        match state {
            CircuitState::Closed => {
                let mut config = self.inner.write();
                config.failure_count += 1;
                if config.failure_count >= config.max_retries {
                    config.state = CircuitState::Open;
                    *config.opened_at.lock() = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                let mut config = self.inner.write();
                config.state = CircuitState::Open;
                *config.opened_at.lock() = Some(Instant::now());
            }
            CircuitState::Open => {}
        }
    }

    /// Check if the circuit breaker should allow a call
    pub fn should_allow_call(&self) -> bool {
        let guard = self.inner.read();

        match guard.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let should_transition = {
                    let opened_at = guard.opened_at.lock();
                    let reset_timeout = guard.reset_timeout;
                    opened_at
                        .as_ref()
                        .map(|instant| instant.elapsed() >= reset_timeout)
                        .unwrap_or(false)
                };
                drop(guard);

                if should_transition {
                    self.inner.write().state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    // Call the future and handle the result depending on the circuit
    pub fn call<F, Fut, T, E>(&self, f: F) -> ResultFuture<F, Fut>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        ResultFuture {
            derived_fut: f,
            state: RetryState::Initial,
            breaker: self.clone(),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

pin_project! {
    pub struct ResultFuture<DerF, F>
    {
        derived_fut: DerF,
        #[pin]
        state: RetryState<F>,
        breaker: CircuitBreaker,
    }
}

pin_project! {
    #[project = RetryStateProj]
    enum RetryState<F> {
        Initial,
        Running { #[pin] future: F },
        Delaying { #[pin] delay: Sleep },
    }
}

impl<'a, F, Fut, T, E> Future for ResultFuture<F, Fut>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<T, Error<E>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let state = match this.state.project() {
                RetryStateProj::Initial => RetryState::Running {
                    future: (this.derived_fut)(),
                },
                RetryStateProj::Delaying { delay } => {
                    ready!(delay.poll(cx));
                    RetryState::Running {
                        future: (this.derived_fut)(),
                    }
                }
                RetryStateProj::Running { future } => {
                    if !this.breaker.should_allow_call() {
                        return Poll::Ready(Err(Error::CircuitOpen));
                    }

                    match ready!(future.poll(cx)) {
                        Ok(output) => {
                            this.breaker.success();
                            return Poll::Ready(Ok(output));
                        }
                        Err(error) => {
                            this.breaker.failure();

                            let guard = this.breaker.inner.read();

                            if guard.failure_count >= guard.max_retries {
                                return Poll::Ready(Err(Error::Inner(error)));
                            }

                            let delay = match guard.backoff {
                                BackoffStrategy::NoBackoff => Duration::ZERO,
                                BackoffStrategy::Constant(delay) => delay,
                                BackoffStrategy::Exponential(initial) => {
                                    let attempt = guard.failure_count;
                                    initial
                                        .checked_mul(2u32.pow(attempt.saturating_sub(1) as u32))
                                        .unwrap_or(Duration::MAX)
                                }
                            };
                            let delay = tokio::time::sleep(delay);

                            RetryState::Delaying { delay }
                        }
                    }
                }
            };
            self.as_mut().project().state.set(state);
        }
    }
}
