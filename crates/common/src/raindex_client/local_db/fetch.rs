use super::{LocalDb, LocalDbError};
use crate::rpc_client::{LogEntryResponse, RpcClientError};
use alloy::{primitives::U256, sol_types::SolEvent};
use backon::{ConstantBuilder, Retryable};
use futures::{StreamExt, TryStreamExt};
use rain_orderbook_bindings::{
    IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    },
    OrderBook::MetaV1_2,
};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct FetchConfig {
    pub chunk_size: u64,
    pub max_concurrent_requests: usize,
    pub max_concurrent_blocks: usize,
    pub max_retry_attempts: usize,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            chunk_size: 5000,
            max_concurrent_requests: 10,
            max_concurrent_blocks: 14,
            max_retry_attempts: 3,
        }
    }
}

impl LocalDb {
    pub async fn fetch_events(
        &self,
        contract_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        self.fetch_events_with_config(
            contract_address,
            start_block,
            end_block,
            &FetchConfig::default(),
        )
        .await
    }

    pub async fn fetch_events_with_config(
        &self,
        contract_address: &str,
        start_block: u64,
        end_block: u64,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        let topics = Some(vec![Some(vec![
            AddOrderV3::SIGNATURE_HASH.to_string(),
            TakeOrderV3::SIGNATURE_HASH.to_string(),
            WithdrawV2::SIGNATURE_HASH.to_string(),
            DepositV2::SIGNATURE_HASH.to_string(),
            RemoveOrderV3::SIGNATURE_HASH.to_string(),
            ClearV3::SIGNATURE_HASH.to_string(),
            AfterClearV2::SIGNATURE_HASH.to_string(),
            MetaV1_2::SIGNATURE_HASH.to_string(),
        ])]);

        let mut chunks = Vec::new();
        let mut current_block = start_block;
        let chunk_size = config.chunk_size.max(1);
        let chunk_span = chunk_size.saturating_sub(1);
        while current_block <= end_block {
            let to_block = current_block.saturating_add(chunk_span).min(end_block);
            chunks.push((current_block, to_block));
            if to_block == u64::MAX {
                break;
            }
            current_block = to_block.saturating_add(1);
        }

        let contract_address = contract_address.to_string();
        let concurrency = config.max_concurrent_requests.max(1);
        let client = self.rpc_client().clone();
        let results: Vec<Vec<LogEntryResponse>> = futures::stream::iter(chunks)
            .map(|(from_block, to_block)| {
                let topics = topics.clone();
                let contract_address = contract_address.clone();
                let client = client.clone();
                let max_attempts = config.max_retry_attempts;

                async move {
                    let from_block_hex = format!("0x{:x}", from_block);
                    let to_block_hex = format!("0x{:x}", to_block);

                    let response = retry_with_attempts(
                        || {
                            client.get_logs(
                                &from_block_hex,
                                &to_block_hex,
                                &contract_address,
                                topics.clone(),
                            )
                        },
                        max_attempts,
                    )
                    .await?;

                    Ok::<_, LocalDbError>(response)
                }
            })
            .buffer_unordered(concurrency)
            .try_collect()
            .await?;

        let mut all_events: Vec<LogEntryResponse> = results.into_iter().flatten().collect();

        all_events.sort_by(|a, b| {
            let block_a = extract_block_number_from_entry(a).unwrap_or(0);
            let block_b = extract_block_number_from_entry(b).unwrap_or(0);
            block_a.cmp(&block_b)
        });

        self.backfill_missing_timestamps(&mut all_events, config)
            .await?;
        Ok(all_events)
    }

    async fn fetch_block_timestamps(
        &self,
        block_numbers: Vec<u64>,
        config: &FetchConfig,
    ) -> Result<HashMap<u64, String>, LocalDbError> {
        if block_numbers.is_empty() {
            return Ok(HashMap::new());
        }

        let concurrency = config.max_concurrent_blocks.max(1);
        let client = self.rpc_client().clone();
        let results: Vec<Result<(u64, String), LocalDbError>> =
            futures::stream::iter(block_numbers)
                .map(|block_number| {
                    let client = client.clone();
                    let max_attempts = config.max_retry_attempts;
                    async move {
                        let block_response = retry_with_attempts(
                            || client.get_block_by_number(block_number),
                            max_attempts,
                        )
                        .await?;

                        let block_data =
                            block_response.ok_or_else(|| LocalDbError::MissingField {
                                field: "result".to_string(),
                            })?;

                        Ok((block_number, block_data.timestamp))
                    }
                })
                .buffer_unordered(concurrency)
                .collect()
                .await;

        results.into_iter().collect()
    }

    async fn backfill_missing_timestamps(
        &self,
        events: &mut [LogEntryResponse],
        config: &FetchConfig,
    ) -> Result<(), LocalDbError> {
        let mut missing_blocks = HashSet::new();

        for event in events.iter() {
            let has_timestamp = event.block_timestamp.as_ref().is_some();

            if !has_timestamp {
                if let Ok(block_number) = extract_block_number_from_entry(event) {
                    missing_blocks.insert(block_number);
                }
            }
        }

        if missing_blocks.is_empty() {
            return Ok(());
        }

        let block_numbers: Vec<u64> = missing_blocks.into_iter().collect();
        let timestamps = self.fetch_block_timestamps(block_numbers, config).await?;

        for event in events.iter_mut() {
            let has_timestamp = event.block_timestamp.as_ref().is_some();

            if has_timestamp {
                continue;
            }

            if let Ok(block_number) = extract_block_number_from_entry(event) {
                if let Some(timestamp) = timestamps.get(&block_number) {
                    event.block_timestamp = Some(timestamp.clone());
                }
            }
        }

        Ok(())
    }
}

const RETRY_DELAY_MILLIS: u64 = 100;

async fn retry_with_attempts<T, F, Fut>(
    operation: F,
    max_attempts: usize,
) -> Result<T, LocalDbError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, RpcClientError>>,
{
    if max_attempts == 0 {
        return Err(LocalDbError::Config {
            message: "max_attempts must be > 0".to_string(),
        });
    }

    let backoff = ConstantBuilder::default()
        .with_delay(Duration::from_millis(RETRY_DELAY_MILLIS))
        .with_max_times(max_attempts.saturating_sub(1));

    let retryable = || async {
        match operation().await {
            Ok(result) => Ok(result),
            Err(RpcClientError::JsonSerialization(err)) => Err(LocalDbError::JsonParse(err)),
            Err(err) => Err(LocalDbError::Rpc(err)),
        }
    };

    retryable
        .retry(&backoff)
        .when(|error: &LocalDbError| matches!(error, LocalDbError::Rpc(_)))
        .await
}

fn extract_block_number_from_entry(event: &LogEntryResponse) -> Result<u64, LocalDbError> {
    parse_block_number_str(&event.block_number)
}

fn parse_block_number_str(block_number_str: &str) -> Result<u64, LocalDbError> {
    let block_u256 = if let Some(hex_digits) = block_number_str
        .strip_prefix("0x")
        .or_else(|| block_number_str.strip_prefix("0X"))
    {
        if hex_digits.is_empty() {
            return Err(LocalDbError::invalid_block_number(
                block_number_str,
                alloy::primitives::ruint::ParseError::InvalidDigit('\0'),
            ));
        }
        U256::from_str_radix(hex_digits, 16)
            .map_err(|e| LocalDbError::invalid_block_number(block_number_str, e))?
    } else {
        U256::from_str_radix(block_number_str, 10)
            .map_err(|e| LocalDbError::invalid_block_number(block_number_str, e))?
    };

    Ok(block_u256.to::<u64>())
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod tokio_tests {
        use super::*;
        use httpmock::prelude::*;
        use serde_json::json;
        use std::str::FromStr;
        use url::Url;

        trait LogEntryResponseSliceExt {
            fn to_json_array(&self) -> Vec<serde_json::Value>;
        }

        impl LogEntryResponseSliceExt for [LogEntryResponse] {
            fn to_json_array(&self) -> Vec<serde_json::Value> {
                self.iter()
                    .map(|entry| serde_json::to_value(entry).expect("serialize log entry"))
                    .collect()
            }
        }

        fn make_log_entry_basic(block_number: &str, timestamp: Option<&str>) -> LogEntryResponse {
            LogEntryResponse {
                address: "0x123".to_string(),
                topics: vec!["0xabc".to_string()],
                data: "0xdeadbeef".to_string(),
                block_number: block_number.to_string(),
                block_timestamp: timestamp.map(|ts| ts.to_string()),
                transaction_hash: "0xtransaction".to_string(),
                transaction_index: "0x0".to_string(),
                block_hash: "0xblock".to_string(),
                log_index: "0x0".to_string(),
                removed: false,
            }
        }

        fn sample_block_response(number: &str, timestamp: Option<&str>) -> String {
            let mut block = json!({
                "mixHash": "0xmix",
                "difficulty": "0x1",
                "extraData": "0xextra",
                "gasLimit": "0xffff",
                "gasUsed": "0xff",
                "hash": "0xhash",
                "logsBloom": "0x0",
                "miner": "0xminer",
                "nonce": "0xnonce",
                "number": number,
                "parentHash": "0xparent",
                "receiptsRoot": "0xreceipts",
                "sha3Uncles": "0xsha3",
                "size": "0x1",
                "stateRoot": "0xstate",
                "timestamp": timestamp.unwrap_or("0x0"),
                "totalDifficulty": "0x2",
                "transactionsRoot": "0xtransactions",
                "uncles": [],
                "transactions": [],
            });

            if timestamp.is_none() {
                if let serde_json::Value::Object(ref mut obj) = block {
                    obj.remove("timestamp");
                }
            }

            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": block,
            })
            .to_string()
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_retry_with_attempts_success_first_try() {
            let result = retry_with_attempts(|| async { Ok::<i32, RpcClientError>(42) }, 3).await;
            assert!(result.is_ok());
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_retry_with_attempts_success_after_retry() {
            use std::sync::{Arc, Mutex};
            let attempt_count = Arc::new(Mutex::new(0));
            let operation = {
                let attempt_count = attempt_count.clone();
                move || {
                    let attempt_count = attempt_count.clone();
                    async move {
                        let mut count = attempt_count.lock().unwrap();
                        *count += 1;
                        let current_attempt = *count;
                        drop(count);

                        if current_attempt < 3 {
                            Err(RpcClientError::RpcError {
                                message: "temporary error".to_string(),
                            })
                        } else {
                            Ok::<i32, RpcClientError>(42)
                        }
                    }
                }
            };

            let result = retry_with_attempts(operation, 5).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_retry_with_attempts_all_fail() {
            let operation = || async {
                Err::<i32, RpcClientError>(RpcClientError::RpcError {
                    message: "always fails".to_string(),
                })
            };

            let result = retry_with_attempts(operation, 3).await;
            assert!(matches!(
                result,
                Err(LocalDbError::Rpc(RpcClientError::RpcError { ref message })) if message == "always fails"
            ));
        }

        #[test]
        fn test_fetch_config_defaults() {
            let config = FetchConfig::default();
            assert_eq!(config.chunk_size, 5000);
            assert_eq!(config.max_concurrent_requests, 10);
            assert_eq!(config.max_concurrent_blocks, 14);
            assert_eq!(config.max_retry_attempts, 3);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_empty_block_numbers() {
            let db = LocalDb::default();
            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![], &config).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_single_block_success() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x64",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.len(), 1);
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_multiple_blocks_success() {
            let server = MockServer::start();
            let mock1 = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x64",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let mock2 = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x65",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x65", Some("0x64b8c124")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100, 101], &config).await;

            mock1.assert();
            mock2.assert();
            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.len(), 2);
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));
            assert_eq!(timestamps.get(&101), Some(&"0x64b8c124".to_string()));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_malformed_json_response() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body("invalid json");
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::JsonParse(_)));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_missing_result_field() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","id":1,"error":"some error"}"#);
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::Rpc(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_null_result() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":null}"#);
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(
                result.unwrap_err(),
                LocalDbError::MissingField { ref field } if field == "result"
            ));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_missing_timestamp_field() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", None));
            });

            let mut db = LocalDb::new_with_hyper_rpc(8453, "test_token".to_string()).unwrap();
            #[cfg(all(test, not(target_family = "wasm")))]
            db.update_rpc_urls(vec![Url::from_str(&server.base_url()).unwrap()]);

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::JsonParse(_)));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_concurrent_requests_limit() {
            let server = MockServer::start();

            for i in 0..5 {
                server.mock(|when, then| {
                    when.method(POST)
                        .path("/")
                        .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":[format!("0x{:x}", 100 + i),false]}));
                    then.status(200)
                        .header("content-type", "application/json")
                        .body({
                            let number_hex = format!("0x{:x}", 100 + i);
                            let timestamp_hex = format!("0x64b8c{:03x}", 123 + i);
                            sample_block_response(number_hex.as_str(), Some(timestamp_hex.as_str()))
                        })
                        .delay(std::time::Duration::from_millis(100));
                });
            }

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig {
                max_concurrent_blocks: 2,
                ..FetchConfig::default()
            };
            let result = db
                .fetch_block_timestamps(vec![100, 101, 102, 103, 104], &config)
                .await;

            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.len(), 5);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_retry_exhaustion() {
            let server = MockServer::start();

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":1}"#);
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig {
                max_retry_attempts: 2,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            assert!(matches!(result.unwrap_err(), LocalDbError::Rpc(_)));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_fetch_block_timestamps_retry_with_eventual_success() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static RETRY_COUNTER: AtomicUsize = AtomicUsize::new(0);

            RETRY_COUNTER.store(0, Ordering::Relaxed);

            let server = MockServer::start();

            let error_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|_req| {
                        RETRY_COUNTER.fetch_add(1, Ordering::Relaxed) == 0
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":1}"#);
            });

            let success_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|_req| RETRY_COUNTER.load(Ordering::Relaxed) > 0);
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig {
                max_retry_attempts: 3,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));

            error_mock.assert_hits(1);
            success_mock.assert_hits(1);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_events_with_existing_timestamps() {
            let db = LocalDb::default();
            let config = FetchConfig::default();

            let mut events = vec![
                {
                    let mut entry = make_log_entry_basic("0x64", Some("0x64b8c123"));
                    entry.data = "some data".to_string();
                    entry
                },
                {
                    let mut entry = make_log_entry_basic("0x65", Some("0x64b8c124"));
                    entry.data = "other data".to_string();
                    entry
                },
            ];

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            let events_array = events.to_json_array();
            assert_eq!(events_array[0]["blockTimestamp"], "0x64b8c123");
            assert_eq!(events_array[1]["blockTimestamp"], "0x64b8c124");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_events_missing_timestamps() {
            let server = MockServer::start();
            let mock1 = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x64",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let mock2 = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x65",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x65", Some("0x64b8c124")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();

            let mut events = vec![
                {
                    let mut entry = make_log_entry_basic("0x64", None);
                    entry.data = "some data".to_string();
                    entry
                },
                {
                    let mut entry = make_log_entry_basic("0x65", None);
                    entry.data = "other data".to_string();
                    entry
                },
            ];

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock1.assert();
            mock2.assert();

            let events_array = events.to_json_array();
            assert_eq!(events_array[0]["blockTimestamp"], "0x64b8c123");
            assert_eq!(events_array[1]["blockTimestamp"], "0x64b8c124");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_invalid_events_format() {
            let events = json!({
                "not": "an array"
            });

            let result: Result<Vec<LogEntryResponse>, _> = serde_json::from_value(events);
            assert!(result.is_err());
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_mixed_events() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x65",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x65", Some("0x64b8c124")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();

            let mut events = vec![
                {
                    let mut entry = make_log_entry_basic("0x64", Some("0x64b8c123"));
                    entry.data = "has timestamp".to_string();
                    entry
                },
                {
                    let mut entry = make_log_entry_basic("0x65", None);
                    entry.data = "missing timestamp".to_string();
                    entry
                },
                {
                    let mut entry = make_log_entry_basic("0x66", Some("0x64b8c125"));
                    entry.data = "has timestamp".to_string();
                    entry
                },
            ];

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock.assert();

            let events_array = events.to_json_array();
            assert_eq!(events_array[0]["blockTimestamp"], "0x64b8c123");
            assert_eq!(events_array[1]["blockTimestamp"], "0x64b8c124");
            assert_eq!(events_array[2]["blockTimestamp"], "0x64b8c125");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_block_number_extraction_failures() {
            let db = LocalDb::default();
            let config = FetchConfig::default();

            let mut events = vec![
                {
                    let mut entry = make_log_entry_basic("invalid_hex", None);
                    entry.data = "bad block number".to_string();
                    entry
                },
                {
                    let mut entry = make_log_entry_basic("0x", None);
                    entry.data = "empty block number".to_string();
                    entry
                },
            ];

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            assert!(events[0].block_timestamp.is_none());
            assert!(events[1].block_timestamp.is_none());
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_empty_events_array() {
            let db = LocalDb::default();
            let config = FetchConfig::default();

            let mut events: Vec<LogEntryResponse> = Vec::new();

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            assert!(events.is_empty());
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_timestamp_fetch_failures() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x64",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":1}"#);
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };

            let mut events = vec![{
                let mut entry = make_log_entry_basic("0x64", None);
                entry.data = "missing timestamp".to_string();
                entry
            }];

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(matches!(result.unwrap_err(), LocalDbError::Rpc(_)));
            mock.assert();
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn test_backfill_missing_timestamps_event_mutation_verification() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x64",false]}));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let db =
                LocalDb::new_with_regular_rpcs(vec![Url::from_str(&server.base_url()).unwrap()])
                    .expect("create local db");

            let config = FetchConfig::default();

            let mut events = vec![{
                let mut entry = make_log_entry_basic("0x64", None);
                entry.data = "original data".to_string();
                entry.transaction_hash = "0xabc123".to_string();
                entry.log_index = "0x0".to_string();
                entry
            }];

            let original_data = events[0].data.clone();
            let original_tx_hash = events[0].transaction_hash.clone();
            let original_log_index = events[0].log_index.clone();

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock.assert();

            let event = &events[0];

            assert_eq!(event.data, original_data);
            assert_eq!(event.transaction_hash, original_tx_hash);
            assert_eq!(event.log_index, original_log_index);
            assert!(event.block_timestamp.is_some());
        }
    }
}
