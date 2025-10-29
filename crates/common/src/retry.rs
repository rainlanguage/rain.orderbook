use crate::{erc20::Error as ERC20Error, local_db::LocalDbError};
use backon::{ExponentialBuilder, Retryable};
use std::{future::Future, time::Duration};

pub const DEFAULT_BASE_DELAY_MILLIS: u64 = 500;

#[derive(Debug)]
pub enum RetryError<E> {
    InvalidMaxAttempts,
    Operation(E),
}

impl From<RetryError<LocalDbError>> for LocalDbError {
    fn from(err: RetryError<LocalDbError>) -> Self {
        match err {
            RetryError::InvalidMaxAttempts => LocalDbError::InvalidRetryMaxAttemps,
            RetryError::Operation(inner) => inner,
        }
    }
}

impl From<RetryError<ERC20Error>> for ERC20Error {
    fn from(err: RetryError<ERC20Error>) -> Self {
        match err {
            RetryError::InvalidMaxAttempts => ERC20Error::InvalidRetryMaxAttemps,
            RetryError::Operation(inner) => inner,
        }
    }
}

pub async fn retry_with_backoff<T, F, Fut, E, ShouldRetry>(
    operation: F,
    max_attempts: usize,
    should_retry: ShouldRetry,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
{
    if max_attempts == 0 {
        return Err(RetryError::InvalidMaxAttempts);
    }

    let backoff = ExponentialBuilder::default()
        .with_min_delay(Duration::from_millis(DEFAULT_BASE_DELAY_MILLIS))
        .with_max_times(max_attempts.saturating_sub(1));

    let retryable = || async { operation().await.map_err(RetryError::Operation) };

    retryable
        .retry(&backoff)
        .when(|error: &RetryError<E>| matches!(error, RetryError::Operation(err) if should_retry(err)))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    enum TestError {
        Rpc,
        Json,
    }

    #[tokio::test]
    async fn retries_and_succeeds_after_transient_error() {
        let attempts = AtomicUsize::new(0);
        let result = retry_with_backoff(
            || async {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err(TestError::Rpc)
                } else {
                    Ok(42u32)
                }
            },
            3,
            |err| matches!(err, TestError::Rpc),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn stops_after_max_attempts() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Rpc)
            },
            2,
            |err| matches!(err, TestError::Rpc),
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Rpc) => {}
            other => panic!("expected Rpc error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable_error() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Json)
            },
            3,
            |err| matches!(err, TestError::Rpc),
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Json) => {}
            other => panic!("expected Json error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn zero_attempts_is_config_error() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok::<u32, TestError>(1)
            },
            0,
            |_err| true,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::InvalidMaxAttempts => {}
            other => panic!("expected config error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
    }
}
