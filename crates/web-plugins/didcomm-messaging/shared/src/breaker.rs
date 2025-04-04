#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
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
/// *   **Half-Open:** After a timeout period, the circuit enters a half-open state, allowing a limited number of operations to be executed.
///     If the probe succeeds, the circuit closes; otherwise, it returns to the open state.
///
/// By default, the circuit breaker is configured with the following:
///
/// *   No retry attempt after a failure.
/// *   A default reset timeout of 30 seconds.
/// *   One retry attempt in half-open state.
/// *   No delay between retries.
///
/// # Configuration
///
/// The behavior of the circuit breaker can be customized using the following builder methods:
///
/// *   [`CircuitBreaker::retries(self, max_retries: usize)`]: Sets the maximum number of consecutive failures allowed before the circuit opens.
///     A value of 0 means the circuit will open on the first failure.
/// *   [`CircuitBreaker::half_open_max_failures(self, max_retries: usize)`]: Sets the maximum number of attempts in half-open state
///     before reopening the circuit.
/// *   [`CircuitBreaker::reset_timeout(self, reset_timeout: Duration)`]: Sets the duration the circuit remains open after tripping.
///     After this timeout, the circuit transitions to the half-open state.
/// *   [`CircuitBreaker::exponential_backoff(self, initial_delay: Duration)`]: Configures an exponential backoff strategy for retries.
///     The delay between retries increases exponentially. This overrides any previously set backoff.
/// *   [`CircuitBreaker::constant_backoff(self, delay: Duration)`]: Configures a constant backoff strategy for retries.
///     The delay between retries remains constant. This overrides any previously set backoff.
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
///         .half_open_max_failures(2)
///         .reset_timeout(Duration::from_secs(10))
///         .exponential_backoff(Duration::from_millis(100));
///
///     async fn operation() -> Result<(), std::io::Error> {
///         // The operation logic here
///         Ok(())
///     }
///
///     match breaker.call(|| operation()).await {
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
    inner: Arc<Mutex<CircuitBreakerConfig>>,
}

#[derive(Debug)]
struct CircuitBreakerConfig {
    // Current state of the circuit
    state: CircuitState,
    // Failure threshold before opening the circuit
    max_retries: usize,
    // Maximum number of retries allowed in half-open state
    half_open_max_retries: usize,
    // Time to wait before closing the circuit again
    reset_timeout: Duration,
    // Tracks failure count
    failure_count: usize,
    // Tracks failure count in half-open state
    half_open_failure_count: usize,
    // Timestamp when circuit was opened
    opened_at: Option<Instant>,
    // Set delay between retries
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

#[derive(Debug, Error, PartialEq, Eq)]
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
            inner: Arc::new(Mutex::new(CircuitBreakerConfig {
                state: CircuitState::Closed,
                max_retries: 0,
                half_open_max_retries: 1,
                reset_timeout: Duration::from_secs(30),
                failure_count: 0,
                half_open_failure_count: 0,
                opened_at: None,
                backoff: BackoffStrategy::NoBackoff,
            })),
        }
    }

    /// Specify the maximum number of attempts for the future
    pub fn retries(self, max_retries: usize) -> Self {
        self.inner.lock().max_retries = max_retries;
        self
    }

    /// Specify the maximum number of attempts for the half-open state
    /// before reopening the circuit
    pub fn half_open_max_failures(self, max_retries: usize) -> Self {
        self.inner.lock().half_open_max_retries = max_retries;
        self
    }

    /// Specify the duration to wait before closing the circuit again
    pub fn reset_timeout(self, reset_timeout: Duration) -> Self {
        self.inner.lock().reset_timeout = reset_timeout;
        self
    }

    /// Set the exponential backoff strategy with the given initial delay for the retry
    ///
    /// The delay will be doubled after each retry
    pub fn exponential_backoff(self, initial_delay: Duration) -> Self {
        self.inner.lock().backoff = BackoffStrategy::Exponential(initial_delay);
        self
    }

    /// Set the fixed backoff strategy with the given delay for the retry
    pub fn constant_backoff(self, delay: Duration) -> Self {
        self.inner.lock().backoff = BackoffStrategy::Constant(delay);
        self
    }

    // Call the future and handle the result depending on the circuit
    pub fn call<F, Fut, T, E>(&self, f: F) -> ResultFuture<F, Fut>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        ResultFuture {
            factory: f,
            state: State::Initial,
            breaker: self.clone(),
        }
    }

    /// Check if the circuit breaker should allow a call
    pub fn should_allow_call(&self) -> bool {
        let mut config = self.inner.lock();

        match config.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let transition = {
                    config
                        .opened_at
                        .map(|time| time.elapsed() >= config.reset_timeout)
                        .unwrap_or(false)
                };

                if transition {
                    config.state = CircuitState::HalfOpen;
                    config.failure_count = 0;
                    config.half_open_failure_count = 0;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn success(&self) {
        let mut config = self.inner.lock();

        match config.state {
            CircuitState::Closed => {
                config.failure_count = 0;
                config.half_open_failure_count = 0;
            }
            CircuitState::HalfOpen => {
                config.state = CircuitState::Closed;
                config.failure_count = 0;
                config.half_open_failure_count = 0;
                config.opened_at = None;
            }
            CircuitState::Open => {}
        }
    }

    fn failure(&self) {
        let mut config = self.inner.lock();

        match config.state {
            CircuitState::Open => {}
            CircuitState::Closed => {
                config.failure_count += 1;
                if config.failure_count > config.max_retries {
                    config.state = CircuitState::Open;
                    config.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                config.half_open_failure_count += 1;
                config.failure_count += 1;
                if config.half_open_failure_count >= config.half_open_max_retries {
                    config.state = CircuitState::Open;
                    config.opened_at = Some(Instant::now());
                    config.half_open_failure_count = 0;
                }
            }
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

pin_project! {
    /// A future that can be retried based on the circuit breaker state
    pub struct ResultFuture<FutFactory, F>
    {
        factory: FutFactory,
        #[pin]
        state: State<F>,
        breaker: CircuitBreaker,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<F> {
        Initial,
        Running { #[pin] future: F },
        Delaying { #[pin] delay: Sleep },
    }
}

impl<F, Fut, T, E> Future for ResultFuture<F, Fut>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    type Output = Result<T, Error<E>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let state = match this.state.project() {
                StateProj::Initial => State::Running {
                    future: (this.factory)(),
                },
                StateProj::Delaying { delay } => {
                    ready!(delay.poll(cx));
                    State::Running {
                        future: (this.factory)(),
                    }
                }
                StateProj::Running { future } => {
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

                            let guard = this.breaker.inner.lock();
                            if guard.failure_count > guard.max_retries {
                                return Poll::Ready(Err(Error::Inner(error)));
                            }

                            let delay = match guard.backoff {
                                BackoffStrategy::NoBackoff => Duration::ZERO,
                                BackoffStrategy::Constant(delay) => delay,
                                BackoffStrategy::Exponential(initial) => {
                                    let failures = guard.failure_count;
                                    initial
                                        .checked_mul(2u32.pow(failures.saturating_sub(1) as u32))
                                        .unwrap_or(Duration::MAX)
                                }
                            };
                            let delay = tokio::time::sleep(delay);

                            State::Delaying { delay }
                        }
                    }
                }
            };
            self.as_mut().project().state.set(state);
        }
    }
}
