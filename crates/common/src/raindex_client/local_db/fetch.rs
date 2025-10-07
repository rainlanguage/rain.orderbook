use super::{LocalDb, LocalDbError};
use crate::hyper_rpc::{HyperRpcError, LogEntryResponse};
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
        let results: Vec<Vec<LogEntryResponse>> = futures::stream::iter(chunks)
            .map(|(from_block, to_block)| {
                let topics = topics.clone();
                let contract_address = contract_address.clone();
                let client = self.client.clone();
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

        // Flatten chunked results
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
        let results: Vec<Result<(u64, String), LocalDbError>> =
            futures::stream::iter(block_numbers)
                .map(|block_number| {
                    let client = self.client.clone();
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
    Fut: std::future::Future<Output = Result<T, HyperRpcError>>,
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
            Err(HyperRpcError::JsonSerialization(err)) => Err(LocalDbError::JsonParse(err)),
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
        use crate::hyper_rpc::HyperRpcClient;
        use httpmock::prelude::*;
        use serde_json::json;

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

        fn logs_response(entries: Vec<serde_json::Value>) -> String {
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": entries,
            })
            .to_string()
        }

        fn sample_log_entry(block_number: &str, timestamp: Option<&str>) -> serde_json::Value {
            let mut entry = json!({
                "address": "0x123",
                "topics": ["0xabc"],
                "data": "0xdeadbeef",
                "blockNumber": block_number,
                "blockTimestamp": timestamp.unwrap_or("0x5"),
                "transactionHash": "0xtransaction",
                "transactionIndex": "0x0",
                "blockHash": "0xblock",
                "logIndex": "0x0",
                "removed": false
            });

            if timestamp.is_none() {
                if let serde_json::Value::Object(ref mut obj) = entry {
                    obj.remove("blockTimestamp");
                }
            }

            entry
        }

        fn log_entry_with(
            block_number: &str,
            transaction_hash: &str,
            data: &str,
            block_timestamp: Option<&str>,
        ) -> serde_json::Value {
            let mut entry = sample_log_entry(block_number, block_timestamp);
            if let serde_json::Value::Object(ref mut obj) = entry {
                obj.insert("transactionHash".to_string(), json!(transaction_hash));
                obj.insert("data".to_string(), json!(data));
            }
            entry
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_retry_with_attempts_success_first_try() {
            let result = retry_with_attempts(|| async { Ok::<i32, HyperRpcError>(42) }, 3).await;
            assert!(result.is_ok());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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
                            Err(HyperRpcError::RpcError {
                                message: "temporary error".to_string(),
                            })
                        } else {
                            Ok::<i32, HyperRpcError>(42)
                        }
                    }
                }
            };

            let result = retry_with_attempts(operation, 5).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_retry_with_attempts_all_fail() {
            let operation = || async {
                Err::<i32, HyperRpcError>(HyperRpcError::RpcError {
                    message: "always fails".to_string(),
                })
            };

            let result = retry_with_attempts(operation, 3).await;
            assert!(matches!(
                result,
                Err(LocalDbError::Rpc(HyperRpcError::RpcError { ref message })) if message == "always fails"
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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_empty_block_numbers() {
            let db = LocalDb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![], &config).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_rpc_client_creation_failure() {
            let result = LocalDb::new(999999, "test_token".to_string());
            assert!(matches!(result, Err(LocalDbError::Rpc(_))));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.len(), 1);
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_malformed_json_response() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body("invalid json");
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::JsonParse(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_missing_result_field() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","id":1,"error":"some error"}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::JsonParse(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 2,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            assert!(matches!(result.unwrap_err(), LocalDbError::Rpc(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_events_with_existing_timestamps() {
            let db = LocalDb::new(8453, "test_token".to_string()).unwrap();
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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_invalid_events_format() {
            let events = json!({
                "not": "an array"
            });

            let result: Result<Vec<LogEntryResponse>, _> = serde_json::from_value(events);
            assert!(result.is_err());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_block_number_extraction_failures() {
            let db = LocalDb::new(8453, "test_token".to_string()).unwrap();
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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_empty_events_array() {
            let db = LocalDb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();

            let mut events: Vec<LogEntryResponse> = Vec::new();

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            assert!(events.is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
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

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

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

            assert_eq!(event.block_timestamp.as_deref(), Some("0x64b8c123"));
            assert_eq!(event.data, original_data);
            assert_eq!(event.transaction_hash, original_tx_hash);
            assert_eq!(event.log_index, original_log_index);
            assert_eq!(event.block_number, "0x64");
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_event_sorting_by_block_number() {
            let server = MockServer::start();

            let logs_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getLogs""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![
                        log_entry_with("0x69", "0x789", "0xdata3", None),
                        log_entry_with("0x64", "0x123", "0xdata1", None),
                        log_entry_with("0x67", "0x456", "0xdata2", None),
                    ]));
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    105,
                    &config,
                )
                .await;

            logs_mock.assert();
            assert!(result.is_ok());

            let events = result.unwrap();
            let events_array = events.to_json_array();

            assert_eq!(events_array.len(), 3);
            assert_eq!(events_array[0]["blockNumber"], "0x64");
            assert_eq!(events_array[0]["transactionHash"], "0x123");
            assert_eq!(events_array[1]["blockNumber"], "0x67");
            assert_eq!(events_array[1]["transactionHash"], "0x456");
            assert_eq!(events_array[2]["blockNumber"], "0x69");
            assert_eq!(events_array[2]["transactionHash"], "0x789");
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_concurrency_limit_enforcement() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            static CONCURRENT_COUNTER: AtomicUsize = AtomicUsize::new(0);
            static MAX_CONCURRENT_SEEN: AtomicUsize = AtomicUsize::new(0);

            CONCURRENT_COUNTER.store(0, Ordering::Relaxed);
            MAX_CONCURRENT_SEEN.store(0, Ordering::Relaxed);

            let server = MockServer::start();

            for i in 0..6 {
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
                    .delay(std::time::Duration::from_millis(200));
            });
            }

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|_req| {
                    let current = CONCURRENT_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

                    let mut max_seen = MAX_CONCURRENT_SEEN.load(Ordering::Relaxed);
                    while current > max_seen {
                        match MAX_CONCURRENT_SEEN.compare_exchange_weak(
                            max_seen,
                            current,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(new_max) => max_seen = new_max,
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(200));
                    CONCURRENT_COUNTER.fetch_sub(1, Ordering::Relaxed);

                    false
                });
                then.status(200);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_concurrent_blocks: 2,
                ..FetchConfig::default()
            };

            let start_time = std::time::Instant::now();
            let result = db
                .fetch_block_timestamps(vec![100, 101, 102, 103, 104, 105], &config)
                .await;
            let duration = start_time.elapsed();

            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.len(), 6);

            let max_concurrent = MAX_CONCURRENT_SEEN.load(Ordering::Relaxed);
            assert!(
                max_concurrent <= 2,
                "Expected max concurrent requests <= 2, but saw {}",
                max_concurrent
            );

            assert!(
            duration >= std::time::Duration::from_millis(500),
            "Requests completed too quickly, suggesting concurrency limit not enforced. Duration: {:?}",
            duration
        );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_fails_when_chunk_fails_after_retries() {
            let server = MockServer::start();

            // All eth_getLogs calls return JSON-RPC error to simulate chunk failure
            server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32603,"message":"Internal error"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 5000,
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    1000,
                    15000,
                    &config,
                )
                .await;

            assert!(matches!(result, Err(LocalDbError::Rpc(_))));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events() {
            let server = MockServer::start();

            let logs_mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![log_entry_with(
                        "0x64", "0x123", "0xdata", None,
                    )]));
            });

            let block_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body_partial(json!({"method": "eth_getBlockByNumber"}).to_string());
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    100,
                    &config,
                )
                .await;

            logs_mock.assert_hits(1);
            block_mock.assert_hits(1);
            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_chunk_size_one() {
            let server = MockServer::start();

            let logs_mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![
                        log_entry_with("0x5", "0x111", "0x1", None),
                        log_entry_with("0x6", "0x222", "0x2", None),
                        log_entry_with("0x7", "0x333", "0x3", None),
                    ]));
            });

            let block_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .json_body_partial(json!({"method": "eth_getBlockByNumber"}).to_string());
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x5", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 1,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    5,
                    7,
                    &config,
                )
                .await;

            logs_mock.assert_hits(3);
            block_mock.assert_hits(3);
            assert!(result.is_ok());
            let events = result.unwrap();
            let events_array = events.to_json_array();
            assert!(events_array.len() >= 3);
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x5"));
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x6"));
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x7"));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_exact_chunk_boundaries() {
            let server = MockServer::start();

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![
                        log_entry_with("0x3e8", "0x111", "0x1", None),
                        log_entry_with("0x2af7", "0x222", "0x2", None),
                    ]));
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x3e8", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 5000,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    1000,
                    10999,
                    &config,
                )
                .await;

            assert!(result.is_ok());
            let events = result.unwrap();
            let events_array = events.to_json_array();
            assert!(events_array.len() >= 2);
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x3e8"));
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x2af7"));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_realistic_rpc_responses() {
            let server = MockServer::start();

            let logs_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": [
                        {
                            "address": "0x742d35cc6634c0532925a3b8c176000000000000",
                            "topics": [
                                "0xe90b7bceb6e7df5418fb78d8ee546e97c83a08bbccc01a0644d599ccd2a7c2e0",
                                "0x000000000000000000000000a0b86a33e6ba3c93c4c84b9e164e3c8f74c8b9a8"
                            ],
                            "data": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000de0b6b3a7640000",
                            "blockNumber": "0x1234567",
                            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                            "transactionIndex": "0x12",
                            "blockHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                            "logIndex": "0x5",
                            "removed": false
                        },
                        {
                            "address": "0x742d35cc6634c0532925a3b8c176000000000000",
                            "topics": [
                                "0xe90b7bceb6e7df5418fb78d8ee546e97c83a08bbccc01a0644d599ccd2a7c2e0"
                            ],
                            "data": "0x0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
                            "blockNumber": "0x1234568",
                            "transactionHash": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
                            "transactionIndex": "0x8",
                            "blockHash": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
                            "logIndex": "0x3",
                            "removed": false
                        }
                    ]
                }"#);
        });

            let _block_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getBlockByNumber""#)
                    } else {
                        false
                    }
                });
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "number": "0x1234567",
                        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                        "parentHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                        "nonce": "0x0000000000000000",
                        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
                        "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                        "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                        "stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544",
                        "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
                        "miner": "0x0000000000000000000000000000000000000000",
                        "difficulty": "0x0",
                        "totalDifficulty": "0x0",
                        "extraData": "0x",
                        "size": "0x220",
                        "gasLimit": "0x1c9c380",
                        "gasUsed": "0x5208",
                        "timestamp": "0x64b8c123",
                        "transactions": [],
                        "uncles": []
                    }
                }"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c176000000000000",
                    19088743,
                    19088744,
                    &config,
                )
                .await;

            logs_mock.assert();

            assert!(result.is_ok());
            let events = result.unwrap();
            let events_array = events.to_json_array();

            assert_eq!(events_array.len(), 2);

            assert_eq!(
                events_array[0]["address"],
                "0x742d35cc6634c0532925a3b8c176000000000000"
            );
            assert_eq!(events_array[0]["logIndex"], "0x5");
            assert_eq!(events_array[0]["transactionIndex"], "0x12");
            assert_eq!(events_array[0]["removed"], false);
            assert!(!events_array[0]["topics"].as_array().unwrap().is_empty());
            assert!(events_array[0]["data"].as_str().unwrap().starts_with("0x"));

            let block1 = parse_block_number_str(&events[0].block_number).unwrap();
            let block2 = parse_block_number_str(&events[1].block_number).unwrap();
            assert!(block1 <= block2, "Events should be sorted by block number");
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_retry_with_real_network_failure_scenarios() {
            let server = MockServer::start();
            use std::sync::atomic::{AtomicUsize, Ordering};

            static ATTEMPT_COUNTER: AtomicUsize = AtomicUsize::new(0);
            ATTEMPT_COUNTER.store(0, Ordering::Relaxed);

            let error_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| {
                        let count = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
                        count == 0
                    });
                then.status(500)
                    .header("content-type", "application/json")
                    .body(r#"{"error":"Internal Server Error"}"#);
            });

            let success_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| ATTEMPT_COUNTER.load(Ordering::Relaxed) > 0);
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 3,
                ..FetchConfig::default()
            };

            let result = db.fetch_block_timestamps(vec![100], &config).await;

            error_mock.assert_hits(1);
            success_mock.assert_hits(1);
            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_retry_with_actual_timeout_simulation() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            static TIMEOUT_ATTEMPT_COUNTER: AtomicUsize = AtomicUsize::new(0);
            TIMEOUT_ATTEMPT_COUNTER.store(0, Ordering::Relaxed);

            let server = MockServer::start();

            let timeout_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| TIMEOUT_ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed) == 0);
                then.status(408) // Request Timeout
                    .header("content-type", "application/json")
                    .body(r#"{"error":"Request timeout"}"#);
            });

            let success_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| TIMEOUT_ATTEMPT_COUNTER.load(Ordering::Relaxed) > 0);
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 3,
                ..FetchConfig::default()
            };

            let result = db.fetch_block_timestamps(vec![100], &config).await;

            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));

            timeout_mock.assert_hits(1);
            success_mock.assert_hits(1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_retry_with_rate_limiting_simulation() {
            let server = MockServer::start();
            use std::sync::atomic::{AtomicUsize, Ordering};

            static RATE_LIMIT_COUNTER: AtomicUsize = AtomicUsize::new(0);
            RATE_LIMIT_COUNTER.store(0, Ordering::Relaxed);

            let rate_limit_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| {
                        let count = RATE_LIMIT_COUNTER.fetch_add(1, Ordering::Relaxed);
                        count == 0
                    });
                then.status(429)
                    .header("content-type", "application/json")
                    .header("retry-after", "1")
                    .body(r#"{"error":"Too Many Requests"}"#);
            });

            let success_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    })
                    .matches(|_req| RATE_LIMIT_COUNTER.load(Ordering::Relaxed) > 0);
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x64b8c123")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 3,
                ..FetchConfig::default()
            };

            let result = db.fetch_block_timestamps(vec![100], &config).await;

            rate_limit_mock.assert_hits(1);
            success_mock.assert_hits(1);
            assert!(result.is_ok());
            let timestamps = result.unwrap();
            assert_eq!(timestamps.get(&100), Some(&"0x64b8c123".to_string()));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_empty_rpc_results() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![]));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    105,
                    &config,
                )
                .await;

            mock.assert();
            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 0);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_start_block_greater_than_end_block() {
            let server = MockServer::start();

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    110,
                    105,
                    &config,
                )
                .await;

            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 0);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_malformed_json_response() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body("invalid json");
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    105,
                    &config,
                )
                .await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::JsonParse(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_missing_result_field() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","id":1,"error":"some error"}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    105,
                    &config,
                )
                .await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), LocalDbError::Rpc(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_max_concurrent_requests_limit() {
            let server = MockServer::start();
            use std::sync::atomic::{AtomicUsize, Ordering};

            static CONCURRENT_COUNTER: AtomicUsize = AtomicUsize::new(0);
            static MAX_CONCURRENT_SEEN: AtomicUsize = AtomicUsize::new(0);

            CONCURRENT_COUNTER.store(0, Ordering::Relaxed);
            MAX_CONCURRENT_SEEN.store(0, Ordering::Relaxed);

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![]))
                    .delay(std::time::Duration::from_millis(200));
            });

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|_req| {
                    let current = CONCURRENT_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

                    let mut max_seen = MAX_CONCURRENT_SEEN.load(Ordering::Relaxed);
                    while current > max_seen {
                        match MAX_CONCURRENT_SEEN.compare_exchange_weak(
                            max_seen,
                            current,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(new_max) => max_seen = new_max,
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(200));
                    CONCURRENT_COUNTER.fetch_sub(1, Ordering::Relaxed);

                    false
                });
                then.status(200);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 1000,
                max_concurrent_requests: 2,
                ..FetchConfig::default()
            };

            let start_time = std::time::Instant::now();
            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    1000,
                    3999,
                    &config,
                )
                .await;
            let duration = start_time.elapsed();

            assert!(result.is_ok());

            let max_concurrent = MAX_CONCURRENT_SEEN.load(Ordering::Relaxed);
            assert!(
                max_concurrent <= 2,
                "Expected max concurrent requests <= 2, but saw {}",
                max_concurrent
            );

            assert!(
            duration >= std::time::Duration::from_millis(300),
            "Requests completed too quickly, suggesting concurrency limit not enforced. Duration: {:?}",
            duration
        );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_different_chunk_sizes() {
            let server = MockServer::start();

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![log_entry_with(
                        "0x64", "0x123", "0x1", None,
                    )]));
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config_small = FetchConfig {
                chunk_size: 10,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    150,
                    &config_small,
                )
                .await;

            assert!(result.is_ok());
            let events = result.unwrap();
            assert!(!events.is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_very_large_range() {
            let server = MockServer::start();

            server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![]));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 1000,
                max_concurrent_requests: 5,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    1000000,
                    1050000,
                    &config,
                )
                .await;

            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 0);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_wrapper_uses_default_config() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![log_entry_with(
                        "0x64", "0x123", "0x1", None,
                    )]));
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let result = db
                .fetch_events("0x742d35Cc6634C0532925a3b8c17600000000000", 100, 105)
                .await;

            mock.assert();
            assert!(result.is_ok());
            let events = result.unwrap();
            assert!(!events.is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_chunk_size_zero() {
            let server = MockServer::start();
            server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getLogs""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![
                        log_entry_with("0x64", "0x123", "0x1", None),
                        log_entry_with("0x65", "0x456", "0x2", None),
                    ]));
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("content-type", "application/json")
                    .matches(|req| {
                        if let Some(ref body) = req.body {
                            let body_str = String::from_utf8_lossy(body);
                            body_str.contains(r#""method":"eth_getBlockByNumber""#)
                        } else {
                            false
                        }
                    });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x64", Some("0x123456")));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 0,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    100,
                    101,
                    &config,
                )
                .await;

            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 4);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_end_block_u64_max() {
            let server = MockServer::start();

            let logs_mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        if body_str.contains(r#""method":"eth_getLogs""#) {
                            assert!(
                                body_str.contains(r#""toBlock":"0xffffffffffffffff""#),
                                "expected toBlock to be 0xffffffffffffffff, got {}",
                                body_str
                            );
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response(vec![]));
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = LocalDb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                chunk_size: 5000,
                ..FetchConfig::default()
            };

            let result = db
                .fetch_events_with_config(
                    "0x742d35Cc6634C0532925a3b8c17600000000000",
                    u64::MAX,
                    u64::MAX,
                    &config,
                )
                .await;

            assert!(result.is_ok());
            logs_mock.assert_hits(1);
        }
    }
}
