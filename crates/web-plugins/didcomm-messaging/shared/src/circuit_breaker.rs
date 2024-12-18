use std::time::{Duration, Instant};

#[derive(Debug)]
enum State {
    // The circuit breaker is closed and allowing requests
    // to pass through
    Closed,
    // The circuit breaker is open and blocking requests
    Open,
    // The circuit breaker is half-open and allowing a limited
    // number of requests to pass through
    HalfOpen,
}

pub struct CircuitBreaker {
    state: State,
    // The duration to wait before transitioning from the
    // open state to the half-open state
    trip_timeout: Duration,
    // The maximum number of requests allowed through in
    // the closed state
    max_failures: usize,
    // The number of consecutive failures in the closed
    // state
    consecutive_failures: usize,
    // The time when the circuit breaker transitioned to the
    // open state
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(max_failures: usize, trip_timeout: Duration) -> CircuitBreaker {
        CircuitBreaker {
            state: State::Closed,
            max_failures,
            trip_timeout,
            consecutive_failures: 0,
            opened_at: None,
        }
    }

    pub async fn call_async<F, Fut, T, E>(&mut self, f: F) -> Option<Result<T, E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        match self.state {
            State::Closed => {
                if self.consecutive_failures < self.max_failures {
                    let result = f().await;
                    if result.is_err() {
                        self.record_failure();
                    }
                    Some(result)
                } else {
                    self.opened_at = Some(Instant::now());
                    self.state = State::Open;
                    self.consecutive_failures = 0;
                    None
                }
            }
            State::Open => {
                if let Some(opened_at) = self.opened_at {
                    if Instant::now().duration_since(opened_at) >= self.trip_timeout {
                        self.state = State::HalfOpen;
                        self.opened_at = None;
                    }
                }
                None
            }
            State::HalfOpen => {
                let result = f().await;
                if result.is_err() {
                    self.state = State::Open;
                } else {
                    self.state = State::Closed;
                }
                Some(result)
            }
        }
    }

    fn record_failure(&mut self) {
        match self.state {
            State::Closed => self.consecutive_failures += 1,
            State::Open => (),
            State::HalfOpen => self.consecutive_failures += 1,
        }
    }
}
