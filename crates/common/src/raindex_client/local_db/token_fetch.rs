use crate::erc20::{TokenInfo, ERC20};
use crate::raindex_client::local_db::LocalDbError;
use alloy::primitives::Address;
use futures::StreamExt;
use std::time::Duration;

pub async fn fetch_erc20_metadata_concurrent(
    rpcs: Vec<url::Url>,
    missing_addrs: Vec<Address>,
) -> Result<Vec<(Address, TokenInfo)>, LocalDbError> {
    const MAX_TOKEN_RETRY_ATTEMPTS: u32 = 3;
    const TOKEN_RETRY_DELAY_MS: u64 = 500;
    const TOKEN_CONCURRENCY: usize = 16;

    async fn fetch_with_retries(
        rpcs: Vec<url::Url>,
        addr: Address,
    ) -> Result<(Address, TokenInfo), LocalDbError> {
        let erc20 = ERC20::new(rpcs.clone(), addr);
        let mut attempt: u32 = 0;
        loop {
            match erc20.token_info(None).await {
                Ok(info) => return Ok((addr, info)),
                Err(e) => {
                    attempt += 1;
                    if attempt >= MAX_TOKEN_RETRY_ATTEMPTS {
                        return Err(LocalDbError::CustomError(format!(
                            "Failed to fetch token info for 0x{:x} after {} attempts: {}",
                            addr, MAX_TOKEN_RETRY_ATTEMPTS, e
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(TOKEN_RETRY_DELAY_MS)).await;
                }
            }
        }
    }

    let results: Vec<Result<(Address, TokenInfo), LocalDbError>> =
        futures::stream::iter(missing_addrs.into_iter().map(|addr| {
            let rpcs = rpcs.clone();
            async move { fetch_with_retries(rpcs, addr).await }
        }))
        .buffer_unordered(TOKEN_CONCURRENCY)
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
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use rain_orderbook_test_fixtures::LocalEvm;
        use url::Url;

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_fetch_erc20_metadata_concurrent_success() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let rpcs = vec![Url::parse(&local_evm.url()).unwrap()];
            let token = local_evm.tokens[0].clone();
            let addrs = vec![*token.address()];

            let out = fetch_erc20_metadata_concurrent(rpcs, addrs).await.unwrap();
            assert_eq!(out.len(), 1);
            assert_eq!(out[0].1.decimals, 18);
            assert_eq!(out[0].1.name, "Token1");
            assert_eq!(out[0].1.symbol, "TOKEN1");
        }

        #[tokio::test]
        async fn test_fetch_erc20_metadata_concurrent_failure_retries() {
            let rpcs = vec![Url::parse("http://127.0.0.1:1").unwrap()];
            let addrs = vec![Address::ZERO];

            let res = fetch_erc20_metadata_concurrent(rpcs, addrs).await;
            assert!(res.is_err());
            match res.err().unwrap() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to fetch token info"));
                }
                other => panic!("Expected CustomError, got {other:?}"),
            }
        }
    }
}
