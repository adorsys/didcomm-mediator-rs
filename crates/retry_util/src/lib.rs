use std::future::Future;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

pub async fn retry_async_operation<F, Fut, T, E>(operation: F, max_retries: usize) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .factor(2)
        .max_delay(std::time::Duration::from_secs(2))
        .take(max_retries);

    Retry::spawn(retry_strategy, operation).await
}
