use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct CircuitBreaker {
    state: AtomicBool, // true = Open, false = Closed
    failure_count: AtomicUsize,
    last_failure_time: Mutex<Option<Instant>>,
    threshold: usize,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    /// Creating a new CircuitBreaker with the given failure threshold and reset timeout.
    pub fn new(threshold: usize, reset_timeout: Duration) -> Self {
        Self {
            state: AtomicBool::new(false),
            failure_count: AtomicUsize::new(0),
            last_failure_time: Mutex::new(None),
            threshold,
            reset_timeout,
        }
    }

    pub fn is_open(&self) -> bool {
        if self.state.load(Ordering::Relaxed) {
            let mut last_failure_time = self.last_failure_time.lock().unwrap();
            if let Some(last_time) = *last_failure_time {
                if last_time.elapsed() > self.reset_timeout {
                    self.state.store(false, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    *last_failure_time = None;
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    pub fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.threshold {
            self.state.store(true, Ordering::Relaxed);
            let mut last_failure_time = self.last_failure_time.lock().unwrap();
            *last_failure_time = Some(Instant::now());
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(false, Ordering::Relaxed);
    }

    pub fn call<F, T, E>(&self, f: F) -> Result<Option<Result<T, E>>, String>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if self.is_open() {
            return Ok(None);
        }

        let result = f();
        match result {
            Ok(_) => self.record_success(),
            Err(_) => self.record_failure(),
        }

        Ok(Some(result))
    }
}

// pub fn request(dice: u32) -> Result<u32, String> {
//     if dice > 6 {
//         Err("400: Bad request.".to_string())
//     } else {
//         Ok(dice)
//     }
// }

// // Example usage
// fn main() {
//     let breaker = CircuitBreaker::new(3, Duration::from_secs(5));

//     for i in 1..=10 {
//         let result = breaker.call(|| request(i));

//         match result {
//             Ok(Some(Ok(value))) => println!("Request succeeded with value: {}", value),
//             Ok(Some(Err(err))) => println!("Request failed: {}", err),
//             Ok(None) => println!("Circuit breaker is open."),
//             Err(err) => println!("Error: {}", err),
//         }

//         // Simulate a delay between requests
//         thread::sleep(Duration::from_millis(500));
//     }
// }
