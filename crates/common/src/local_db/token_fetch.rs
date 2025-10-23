use crate::erc20::{Error as TokenError, TokenInfo, ERC20};
use crate::local_db::{FetchConfig, LocalDbError};
use crate::retry::{retry_with_backoff, RetryError};
use alloy::primitives::Address;
use futures::StreamExt;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};

fn should_retry_token_error(err: &TokenError) -> bool {
    matches!(
        err,
        TokenError::ReadProviderError(_)
            | TokenError::ContractCallError(_)
            | TokenError::MulticallError(_)
    )
}

pub async fn fetch_erc20_metadata_concurrent(
    rpcs: Vec<url::Url>,
    missing_addrs: Vec<Address>,
    config: &FetchConfig,
) -> Result<Vec<(Address, TokenInfo)>, LocalDbError> {
    let concurrency = config.max_concurrent_requests();
    let max_attempts = config.max_retry_attempts();

    let results: Vec<Result<(Address, TokenInfo), LocalDbError>> =
        futures::stream::iter(missing_addrs.into_iter().map(|addr| {
            let rpcs = rpcs.clone();
            async move {
                let erc20 = ERC20::new(rpcs, addr);
                let attempt_counter = Arc::new(AtomicUsize::new(0));
                let result = retry_with_backoff(
                    || {
                        let erc20 = erc20.clone();
                        let attempt_counter = Arc::clone(&attempt_counter);
                        async move {
                            attempt_counter.fetch_add(1, Ordering::Relaxed);
                            erc20.token_info(None).await
                        }
                    },
                    max_attempts,
                    should_retry_token_error,
                )
                .await;

                match result {
                    Ok(info) => Ok((addr, info)),
                    Err(RetryError::Operation(err)) => {
                        let attempts = attempt_counter.load(Ordering::Relaxed);
                        Err(LocalDbError::TokenMetadataFetchFailed {
                            address: addr,
                            attempts,
                            source: Box::new(err),
                        })
                    }
                    Err(RetryError::Config { message }) => Err(LocalDbError::Config { message }),
                }
            }
        }))
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut successes: Vec<(Address, TokenInfo)> = Vec::new();
    for r in results {
        match r {
            Ok(pair) => successes.push(pair),
            Err(e) => return Err(e),
        }
    }
    Ok(successes)
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use crate::erc20::Error as TokenError;
        use crate::local_db::token_fetch::fetch_erc20_metadata_concurrent;
        use crate::local_db::{FetchConfig, LocalDbError};
        use alloy::primitives::Address;
        use rain_orderbook_test_fixtures::LocalEvm;
        use url::Url;

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_fetch_erc20_metadata_concurrent_success() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let rpcs = vec![Url::parse(&local_evm.url()).unwrap()];
            let token = local_evm.tokens[0].clone();
            let addrs = vec![*token.address()];

            let out = fetch_erc20_metadata_concurrent(rpcs, addrs, &FetchConfig::default())
                .await
                .unwrap();
            assert_eq!(out.len(), 1);
            assert_eq!(out[0].1.decimals, 18);
            assert_eq!(out[0].1.name, "Token1");
            assert_eq!(out[0].1.symbol, "TOKEN1");
        }

        #[tokio::test]
        async fn test_fetch_erc20_metadata_concurrent_failure_retries() {
            let rpcs = vec![Url::parse("http://127.0.0.1:1").unwrap()];
            let addrs = vec![Address::ZERO];

            let res = fetch_erc20_metadata_concurrent(rpcs, addrs, &FetchConfig::default()).await;
            assert!(res.is_err());
            match res.err().unwrap() {
                LocalDbError::TokenMetadataFetchFailed {
                    address,
                    attempts,
                    source,
                } => {
                    assert_eq!(address, Address::ZERO);
                    assert_eq!(attempts, FetchConfig::default().max_retry_attempts());
                    assert!(
                        matches!(
                            source.as_ref(),
                            TokenError::ReadProviderError(_) | TokenError::MulticallError(_)
                        ),
                        "unexpected source error: {:?}",
                        source
                    );
                }
                other => panic!("Expected TokenMetadataFetchFailed, got {other:?}"),
            }
        }
    }
}
