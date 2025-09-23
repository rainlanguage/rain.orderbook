use super::{SqliteWeb, SqliteWebError};
use crate::hyper_rpc::HyperRpcError;
use alloy::{primitives::U256, sol_types::SolEvent};
use futures::{StreamExt, TryStreamExt};
use rain_orderbook_bindings::{
    IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    },
    OrderBook::MetaV1_2,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RpcEnvelope<T> {
    Result { result: Option<T> },
    Error { error: serde_json::Value },
}

#[derive(Debug, Deserialize)]
struct BlockResponse {
    timestamp: Option<String>,
}

impl SqliteWeb {
    pub async fn fetch_events(
        &self,
        contract_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<serde_json::Value, SqliteWebError> {
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
    ) -> Result<serde_json::Value, SqliteWebError> {
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
        while current_block <= end_block {
            let to_block = std::cmp::min(
                current_block.saturating_add(chunk_size).saturating_sub(1),
                end_block,
            );
            chunks.push((current_block, to_block));
            current_block = to_block.saturating_add(1);
            if to_block == u64::MAX {
                break;
            }
        }

        let contract_address = contract_address.to_string();
        let concurrency = config.max_concurrent_requests.max(1);
        let results: Vec<Vec<serde_json::Value>> = futures::stream::iter(chunks)
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

                    let rpc_envelope: RpcEnvelope<Vec<serde_json::Value>> =
                        serde_json::from_str(&response)?;

                    let logs = match rpc_envelope {
                        RpcEnvelope::Result { result } => result.unwrap_or_default(),
                        RpcEnvelope::Error { error } => {
                            return Err(SqliteWebError::Rpc(HyperRpcError::RpcError {
                                message: error.to_string(),
                            }));
                        }
                    };

                    Ok::<_, SqliteWebError>(logs)
                }
            })
            .buffer_unordered(concurrency)
            .try_collect()
            .await?;

        // Flatten chunked results
        let mut all_events: Vec<serde_json::Value> = results.into_iter().flatten().collect();

        all_events.sort_by(|a, b| {
            let block_a = extract_block_number(a).unwrap_or(0);
            let block_b = extract_block_number(b).unwrap_or(0);
            block_a.cmp(&block_b)
        });

        let mut events_array = serde_json::Value::Array(all_events);
        self.backfill_missing_timestamps(&mut events_array, config)
            .await?;
        Ok(events_array)
    }

    async fn fetch_block_timestamps(
        &self,
        block_numbers: Vec<u64>,
        config: &FetchConfig,
    ) -> Result<HashMap<u64, String>, SqliteWebError> {
        if block_numbers.is_empty() {
            return Ok(HashMap::new());
        }

        let concurrency = config.max_concurrent_blocks.max(1);
        let results: Vec<Result<(u64, String), SqliteWebError>> =
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

                        let rpc_envelope: RpcEnvelope<BlockResponse> =
                            serde_json::from_str(&block_response)?;

                        let block_data = match rpc_envelope {
                            RpcEnvelope::Result { result } => {
                                result.ok_or_else(|| SqliteWebError::MissingField {
                                    field: "result".to_string(),
                                })?
                            }
                            RpcEnvelope::Error { error } => {
                                return Err(SqliteWebError::Rpc(HyperRpcError::RpcError {
                                    message: error.to_string(),
                                }));
                            }
                        };

                        let timestamp =
                            block_data
                                .timestamp
                                .ok_or_else(|| SqliteWebError::MissingField {
                                    field: "timestamp".to_string(),
                                })?;

                        Ok((block_number, timestamp))
                    }
                })
                .buffer_unordered(concurrency)
                .collect()
                .await;

        results.into_iter().collect()
    }

    async fn backfill_missing_timestamps(
        &self,
        events: &mut serde_json::Value,
        config: &FetchConfig,
    ) -> Result<(), SqliteWebError> {
        let events_array = match events.as_array_mut() {
            Some(array) => array,
            None => return Err(SqliteWebError::InvalidEventsFormat),
        };

        let mut missing_blocks = HashSet::new();

        for event in events_array.iter() {
            if event.get("blockTimestamp").is_none() {
                if let Ok(block_number) = extract_block_number(event) {
                    missing_blocks.insert(block_number);
                }
            }
        }

        if missing_blocks.is_empty() {
            return Ok(());
        }

        let block_numbers: Vec<u64> = missing_blocks.into_iter().collect();
        let timestamps = self.fetch_block_timestamps(block_numbers, config).await?;

        for event in events_array.iter_mut() {
            if event.get("blockTimestamp").is_none() {
                if let Ok(block_number) = extract_block_number(event) {
                    if let Some(timestamp) = timestamps.get(&block_number) {
                        if let Some(event_obj) = event.as_object_mut() {
                            event_obj.insert(
                                "blockTimestamp".to_string(),
                                serde_json::Value::String(timestamp.clone()),
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

async fn retry_with_attempts<T, F, Fut>(
    operation: F,
    max_attempts: usize,
) -> Result<T, SqliteWebError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, HyperRpcError>>,
{
    let mut last_error = SqliteWebError::MissingField {
        field: "Not attempted".to_string(),
    };

    for _attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => last_error = SqliteWebError::Rpc(e),
        }
    }

    Err(last_error)
}

fn extract_block_number(event: &serde_json::Value) -> Result<u64, SqliteWebError> {
    let block_number_str = event
        .get("blockNumber")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SqliteWebError::MissingField {
            field: "blockNumber".to_string(),
        })?;

    let block_u256 = if let Some(hex_digits) = block_number_str
        .strip_prefix("0x")
        .or_else(|| block_number_str.strip_prefix("0X"))
    {
        if hex_digits.is_empty() {
            return Err(SqliteWebError::invalid_block_number(
                block_number_str,
                alloy::primitives::ruint::ParseError::InvalidDigit('\0'),
            ));
        }
        U256::from_str_radix(hex_digits, 16)
            .map_err(|e| SqliteWebError::invalid_block_number(block_number_str, e))?
    } else {
        U256::from_str_radix(block_number_str, 10)
            .map_err(|e| SqliteWebError::invalid_block_number(block_number_str, e))?
    };

    Ok(block_u256.to::<u64>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_block_number_valid_hex() {
        let event = json!({
            "blockNumber": "0x123"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 0x123);

        let event = json!({
            "blockNumber": "0xabc"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 0xabc);

        let event = json!({
            "blockNumber": "0x0"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 0);
    }

    #[test]
    fn test_extract_block_number_decimal() {
        let event = json!({
            "blockNumber": "123"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 123);

        let event = json!({
            "blockNumber": "0"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 0);

        let event = json!({
            "blockNumber": "999999"
        });
        assert_eq!(extract_block_number(&event).unwrap(), 999999);
    }

    #[test]
    fn test_extract_block_number_invalid() {
        let event = json!({
            "blockNumber": "invalid"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "0x"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "hello0x123"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "not_a_number"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "0xGHI"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "12.5"
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": "-123"
        });
        assert!(extract_block_number(&event).is_err());
    }

    #[test]
    fn test_extract_block_number_missing_field() {
        let event = json!({
            "otherField": "value"
        });
        assert!(matches!(
            extract_block_number(&event),
            Err(SqliteWebError::MissingField { ref field }) if field == "blockNumber"
        ));
    }

    #[test]
    fn test_extract_block_number_non_string() {
        let event = json!({
            "blockNumber": null
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": 123
        });
        assert!(extract_block_number(&event).is_err());

        let event = json!({
            "blockNumber": {}
        });
        assert!(extract_block_number(&event).is_err());
    }

    #[cfg(not(target_family = "wasm"))]
    mod tokio_tests {
        use super::*;
        use crate::hyper_rpc::HyperRpcClient;
        use httpmock::prelude::*;

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
                Err(SqliteWebError::Rpc(HyperRpcError::RpcError { ref message })) if message == "always fails"
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
            let db = SqliteWeb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![], &config).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_rpc_client_creation_failure() {
            let result = SqliteWeb::new(999999, "test_token".to_string());
            assert!(matches!(result, Err(SqliteWebError::Rpc(_))));
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
        });

            let mock2 = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .header("content-type", "application/json")
                .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x65",false]}));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c124"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), SqliteWebError::JsonParse(_)));
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
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(matches!(result.unwrap_err(), SqliteWebError::Rpc(_)));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_block_timestamps_missing_timestamp_field() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
                when.method(POST).path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"number":"0x64"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            mock.assert();
            assert!(result.is_err());
            assert!(
                matches!(result.unwrap_err(), SqliteWebError::MissingField { ref field } if field == "timestamp")
            );
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
                    .body(format!(r#"{{"jsonrpc":"2.0","id":1,"result":{{"timestamp":"0x64b8c{:03x}"}}}}"#, 123 + i))
                    .delay(std::time::Duration::from_millis(100));
            });
            }

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 2,
                ..FetchConfig::default()
            };
            let result = db.fetch_block_timestamps(vec![100], &config).await;

            assert!(matches!(result.unwrap_err(), SqliteWebError::Rpc(_)));
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let db = SqliteWeb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();

            let mut events = json!([
                {
                    "blockNumber": "0x64",
                    "blockTimestamp": "0x64b8c123",
                    "data": "some data"
                },
                {
                    "blockNumber": "0x65",
                    "blockTimestamp": "0x64b8c124",
                    "data": "other data"
                }
            ]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            let events_array = events.as_array().unwrap();
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
        });

            let mock2 = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .json_body(json!({"id":1,"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["0x65",false]}));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c124"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();

            let mut events = json!([
                {
                    "blockNumber": "0x64",
                    "data": "some data"
                },
                {
                    "blockNumber": "0x65",
                    "data": "other data"
                }
            ]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock1.assert();
            mock2.assert();

            let events_array = events.as_array().unwrap();
            assert_eq!(events_array[0]["blockTimestamp"], "0x64b8c123");
            assert_eq!(events_array[1]["blockTimestamp"], "0x64b8c124");
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_invalid_events_format() {
            let db = SqliteWeb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();

            let mut events = json!({
                "not": "an array"
            });

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(matches!(
                result.unwrap_err(),
                SqliteWebError::InvalidEventsFormat
            ));
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c124"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();

            let mut events = json!([
                {
                    "blockNumber": "0x64",
                    "blockTimestamp": "0x64b8c123",
                    "data": "has timestamp"
                },
                {
                    "blockNumber": "0x65",
                    "data": "missing timestamp"
                },
                {
                    "blockNumber": "0x66",
                    "blockTimestamp": "0x64b8c125",
                    "data": "has timestamp"
                }
            ]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock.assert();

            let events_array = events.as_array().unwrap();
            assert_eq!(events_array[0]["blockTimestamp"], "0x64b8c123");
            assert_eq!(events_array[1]["blockTimestamp"], "0x64b8c124");
            assert_eq!(events_array[2]["blockTimestamp"], "0x64b8c125");
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_block_number_extraction_failures() {
            let db = SqliteWeb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();

            let mut events = json!([
                {
                    "blockNumber": "invalid_hex",
                    "data": "bad block number"
                },
                {
                    "missingBlockNumber": "0x65",
                    "data": "no block number field"
                },
                {
                    "blockNumber": 123,
                    "data": "non-string block number"
                }
            ]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            let events_array = events.as_array().unwrap();
            assert!(events_array[0].get("blockTimestamp").is_none());
            assert!(events_array[1].get("blockTimestamp").is_none());
            assert!(events_array[2].get("blockTimestamp").is_none());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_backfill_missing_timestamps_empty_events_array() {
            let db = SqliteWeb::new(8453, "test_token".to_string()).unwrap();
            let config = FetchConfig::default();

            let mut events = json!([]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            let events_array = events.as_array().unwrap();
            assert!(events_array.is_empty());
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
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig {
                max_retry_attempts: 1,
                ..FetchConfig::default()
            };

            let mut events = json!([
                {
                    "blockNumber": "0x64",
                    "data": "missing timestamp"
                }
            ]);

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(matches!(result.unwrap_err(), SqliteWebError::Rpc(_)));
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
        });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let config = FetchConfig::default();

            let mut events = json!([
                {
                    "blockNumber": "0x64",
                    "data": "original data",
                    "transactionHash": "0xabc123",
                    "logIndex": "0x0"
                }
            ]);

            let original_data = events[0]["data"].clone();
            let original_tx_hash = events[0]["transactionHash"].clone();
            let original_log_index = events[0]["logIndex"].clone();

            let result = db.backfill_missing_timestamps(&mut events, &config).await;
            assert!(result.is_ok());

            mock.assert();

            let events_array = events.as_array().unwrap();
            let event = &events_array[0];

            assert_eq!(event["blockTimestamp"], "0x64b8c123");
            assert_eq!(event["data"], original_data);
            assert_eq!(event["transactionHash"], original_tx_hash);
            assert_eq!(event["logIndex"], original_log_index);
            assert_eq!(event["blockNumber"], "0x64");
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
                    .body(
                        r#"{"jsonrpc":"2.0","id":1,"result":[
                    {
                        "blockNumber": "0x69",
                        "transactionHash": "0x789",
                        "logIndex": "0x0",
                        "data": "0xdata3"
                    },
                    {
                        "blockNumber": "0x64",
                        "transactionHash": "0x123",
                        "logIndex": "0x0",
                        "data": "0xdata1"
                    },
                    {
                        "blockNumber": "0x67",
                        "transactionHash": "0x456",
                        "logIndex": "0x0",
                        "data": "0xdata2"
                    }
                ]}"#,
                    );
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x123456"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let events_array = events.as_array().unwrap();

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
                    .body(format!(r#"{{"jsonrpc":"2.0","id":1,"result":{{"timestamp":"0x64b8c{:03x}"}}}}"#, 123 + i))
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
            let mut db = SqliteWeb::new_with_client(client);
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
            let mut db = SqliteWeb::new_with_client(client);
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

            assert!(matches!(result, Err(SqliteWebError::Rpc(_))));
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
                    .body(
                        r#"{"jsonrpc":"2.0","id":1,"result":[
                    {
                        "blockNumber": "0x64",
                        "transactionHash": "0x123",
                        "logIndex": "0x0",
                        "data": "0xdata"
                    }
                ]}"#,
                    );
            });

            let block_mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getBlockByNumber""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"jsonrpc":"2.0","id":1,"result":{"number":"0x64","timestamp":"0x123456"}}"#,
                );
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert_eq!(events.as_array().unwrap().len(), 1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_chunk_size_one() {
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[
                    {"blockNumber": "0x5", "transactionHash": "0x111", "logIndex": "0x0", "data": "0x1"},
                    {"blockNumber": "0x6", "transactionHash": "0x222", "logIndex": "0x0", "data": "0x2"},
                    {"blockNumber": "0x7", "transactionHash": "0x333", "logIndex": "0x0", "data": "0x3"}
                ]}"#);
        });

            let block_mock = server.mock(|when, then| {
                when.method(POST).path("/").matches(|req| {
                    if let Some(ref body) = req.body {
                        let body_str = String::from_utf8_lossy(body);
                        body_str.contains(r#""method":"eth_getBlockByNumber""#)
                    } else {
                        false
                    }
                });
                then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"jsonrpc":"2.0","id":1,"result":{"number":"0x5","timestamp":"0x123456"}}"#,
                );
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let events_array = events.as_array().unwrap();
            assert!(events_array.len() >= 3);
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x5"));
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x6"));
            assert!(events_array.iter().any(|e| e["blockNumber"] == "0x7"));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_exact_chunk_boundaries() {
            let server = MockServer::start();

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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[
                    {"blockNumber": "0x3e8", "transactionHash": "0x111", "logIndex": "0x0", "data": "0x1"},
                    {"blockNumber": "0x2af7", "transactionHash": "0x222", "logIndex": "0x0", "data": "0x2"}
                ]}"#);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x123456"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            let events_array = events.as_array().unwrap();
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
            let mut db = SqliteWeb::new_with_client(client);
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
            let events_array = events.as_array().unwrap();

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

            let block1 = extract_block_number(&events_array[0]).unwrap();
            let block2 = extract_block_number(&events_array[1]).unwrap();
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x64b8c123"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":[]}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert_eq!(events.as_array().unwrap().len(), 0);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_start_block_greater_than_end_block() {
            let server = MockServer::start();

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert_eq!(events.as_array().unwrap().len(), 0);
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
            let mut db = SqliteWeb::new_with_client(client);
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
            assert!(matches!(result.unwrap_err(), SqliteWebError::JsonParse(_)));
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
            let mut db = SqliteWeb::new_with_client(client);
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
            assert!(matches!(result.unwrap_err(), SqliteWebError::Rpc(_)));
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":[]}"#)
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
            let mut db = SqliteWeb::new_with_client(client);
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[
                    {"blockNumber": "0x64", "transactionHash": "0x123", "logIndex": "0x0", "data": "0x1"}
                ]}"#);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x123456"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert!(!events.as_array().unwrap().is_empty());
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":[]}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert_eq!(events.as_array().unwrap().len(), 0);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_wrapper_uses_default_config() {
            let server = MockServer::start();
            let mock = server.mock(|when, then| {
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[
                    {"blockNumber": "0x64", "transactionHash": "0x123", "logIndex": "0x0", "data": "0x1"}
                ]}"#);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x123456"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
            db.client_mut().update_rpc_url(server.base_url());

            let result = db
                .fetch_events("0x742d35Cc6634C0532925a3b8c17600000000000", 100, 105)
                .await;

            mock.assert();
            assert!(result.is_ok());
            let events = result.unwrap();
            assert!(!events.as_array().unwrap().is_empty());
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_fetch_events_with_config_chunk_size_zero() {
            let server = MockServer::start();
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
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[
                    {"blockNumber": "0x64", "transactionHash": "0x123", "logIndex": "0x0", "data": "0x1"},
                    {"blockNumber": "0x65", "transactionHash": "0x456", "logIndex": "0x0", "data": "0x2"}
                ]}"#);
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
                    .body(r#"{"jsonrpc":"2.0","id":1,"result":{"timestamp":"0x123456"}}"#);
            });

            let client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
            let mut db = SqliteWeb::new_with_client(client);
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
            assert_eq!(events.as_array().unwrap().len(), 4);
        }
    }
}
