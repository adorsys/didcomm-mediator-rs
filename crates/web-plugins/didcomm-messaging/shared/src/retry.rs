use std::time::Duration;
use tokio::time::sleep;

pub struct RetryOptions {
    retries: usize,
    fixed_backoff: Option<Duration>,
    exponential_backoff: Option<Duration>,
    max_delay: Option<Duration>,
}

impl RetryOptions {
    pub fn new() -> Self {
        Self {
            retries: 3,
            fixed_backoff: None,
            exponential_backoff: None,
            max_delay: None,
        }
    }

    pub fn retries(mut self, count: usize) -> Self {
        self.retries = count;
        self
    }

    pub fn fixed_backoff(mut self, delay: Duration) -> Self {
        self.fixed_backoff = Some(delay);
        self
    }

    pub fn exponential_backoff(mut self, initial_delay: Duration) -> Self {
        self.exponential_backoff = Some(initial_delay);
        self
    }

    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = Some(delay);
        self
    }
}

pub async fn retry_async<F, Fut, T, E>(mut operation: F, options: RetryOptions) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let RetryOptions {
        retries,
        fixed_backoff,
        exponential_backoff,
        max_delay,
    } = options;

    let mut attempt = 0;
    let mut delay = exponential_backoff.unwrap_or_default();
    let max_delay = max_delay.unwrap_or_else(|| Duration::from_secs(60)); // Default max delay of 60 seconds

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) if attempt <= retries => {
                if let Some(fixed) = fixed_backoff {
                    sleep(fixed).await;
                } else if delay > Duration::ZERO {
                    let next_delay = delay.min(max_delay);
                    sleep(next_delay).await;
                    delay = (delay * 2).min(max_delay);
                }
            }
            Err(err) => return Err(err),
        }
    }
}
