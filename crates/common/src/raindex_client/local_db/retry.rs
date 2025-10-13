use super::LocalDbError;
use backon::{ConstantBuilder, Retryable};
use std::time::Duration;

pub(crate) const RETRY_DELAY_MILLIS: u64 = 100;

pub(crate) async fn retry_with_backoff<T, F, Fut, P>(
    operation: F,
    max_attempts: usize,
    should_retry: P,
) -> Result<T, LocalDbError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, LocalDbError>>,
    P: Fn(&LocalDbError) -> bool + Copy,
{
    if max_attempts == 0 {
        return Err(LocalDbError::Config {
            message: "max_attempts must be > 0".to_string(),
        });
    }

    let backoff = ConstantBuilder::default()
        .with_delay(Duration::from_millis(RETRY_DELAY_MILLIS))
        .with_max_times(max_attempts.saturating_sub(1));

    let retryable = || operation();

    retryable.retry(&backoff).when(should_retry).await
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn retry_succeeds_without_retries() {
        let attempts = Arc::new(AtomicUsize::new(0));

        let result = retry_with_backoff(
            || {
                let attempts = attempts.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, LocalDbError>(42)
                }
            },
            3,
            |_err| true,
        )
        .await
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_succeeds_after_transient_errors() {
        let attempts = Arc::new(AtomicUsize::new(0));

        let result = retry_with_backoff(
            || {
                let attempts = attempts.clone();
                async move {
                    let current = attempts.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err::<usize, LocalDbError>(LocalDbError::CustomError("boom".to_string()))
                    } else {
                        Ok::<_, LocalDbError>(current)
                    }
                }
            },
            4,
            |err| matches!(err, LocalDbError::CustomError(_)),
        )
        .await
        .unwrap();

        assert_eq!(result, 2);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_stops_when_condition_false() {
        let attempts = Arc::new(AtomicUsize::new(0));

        let err = retry_with_backoff(
            || {
                let attempts = attempts.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<usize, LocalDbError>(LocalDbError::CustomError(
                        "do-not-retry".to_string(),
                    ))
                }
            },
            5,
            |_err| false,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, LocalDbError::CustomError(_)));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_fails_after_max_attempts() {
        let attempts = Arc::new(AtomicUsize::new(0));

        let err = retry_with_backoff(
            || {
                let attempts = attempts.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<usize, LocalDbError>(LocalDbError::CustomError(
                        "still-failing".to_string(),
                    ))
                }
            },
            3,
            |_err| true,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, LocalDbError::CustomError(_)));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_with_zero_attempts_is_config_error() {
        let err = retry_with_backoff(|| async { Ok::<_, LocalDbError>(()) }, 0, |_err| true)
            .await
            .unwrap_err();

        assert!(matches!(err, LocalDbError::Config { .. }));
    }
}
