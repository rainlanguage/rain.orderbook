use super::{LocalDb, LocalDbError, RAINTERPRETER_STORE_SET_TOPIC};
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
            block_a
                .cmp(&block_b)
                .then_with(|| a.log_index.cmp(&b.log_index))
        });

        self.backfill_missing_timestamps(&mut all_events, config)
            .await?;
        Ok(all_events)
    }

    pub async fn fetch_store_set_events(
        &self,
        store_addresses: &[String],
        start_block: u64,
        end_block: u64,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        if store_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let unique_addresses: Vec<String> = {
            let mut dedup = HashSet::new();
            store_addresses
                .iter()
                .filter_map(|addr| {
                    let lower = addr.to_ascii_lowercase();
                    if dedup.insert(lower.clone()) {
                        Some(lower)
                    } else {
                        None
                    }
                })
                .collect()
        };

        if unique_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let mut jobs = Vec::new();
        let chunk_size = config.chunk_size.max(1);
        let chunk_span = chunk_size.saturating_sub(1);
        for address in unique_addresses.into_iter() {
            let mut current_block = start_block;
            while current_block <= end_block {
                let to_block = current_block.saturating_add(chunk_span).min(end_block);
                jobs.push((address.clone(), current_block, to_block));
                current_block = to_block.saturating_add(1);
                if to_block == u64::MAX {
                    break;
                }
            }
        }

        let topics = Some(vec![Some(vec![RAINTERPRETER_STORE_SET_TOPIC.to_string()])]);
        let concurrency = config.max_concurrent_requests.max(1);
        let client = self.rpc_client().clone();
        let results: Vec<Vec<LogEntryResponse>> = futures::stream::iter(jobs)
            .map(|(address, from_block, to_block)| {
                let topics = topics.clone();
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
                                &address,
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

        let mut events: Vec<LogEntryResponse> = results.into_iter().flatten().collect();
        events.sort_by(|a, b| {
            let block_a = extract_block_number_from_entry(a).unwrap_or(0);
            let block_b = extract_block_number_from_entry(b).unwrap_or(0);
            let log_a = parse_block_number_str(&a.log_index).unwrap_or(0);
            let log_b = parse_block_number_str(&b.log_index).unwrap_or(0);
            block_a.cmp(&block_b).then_with(|| log_a.cmp(&log_b))
        });

        self.backfill_missing_timestamps(&mut events, config)
            .await?;
        Ok(events)
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
        use alloy::hex;
        use httpmock::prelude::*;
        use serde_json::json;
        use std::str::FromStr;
        use url::Url;

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
                block.as_object_mut().unwrap().remove("timestamp");
            }
            block.to_string()
        }

        #[tokio::test]
        async fn fetch_events_with_config_fetches_and_sorts() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            let response_body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": [
                    {
                        "address": "0xabc",
                        "topics": [
                            format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))
                        ],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x2",
                        "transactionHash": "0x2",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x1",
                        "removed": false
                    },
                    {
                        "address": "0xabc",
                        "topics": [
                            format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))
                        ],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x1",
                        "transactionHash": "0x1",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x0",
                        "removed": false
                    }
                ]
            });

            let block_response = sample_block_response("0x1", Some("0x64"));
            let block_response_two = sample_block_response("0x2", Some("0x65"));

            server.mock(|when, then| {
                when.method(POST).path("/").body_contains("\"eth_getLogs\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response_body.to_string());
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getBlockByNumber\"")
                    .body_contains("\"0x1\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(block_response);
            });

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getBlockByNumber\"")
                    .body_contains("\"0x2\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(block_response_two);
            });

            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let events = db
                .fetch_events_with_config(
                    "0xabc",
                    1,
                    2,
                    &FetchConfig {
                        chunk_size: 1000,
                        max_concurrent_requests: 1,
                        max_concurrent_blocks: 1,
                        max_retry_attempts: 1,
                    },
                )
                .await
                .unwrap();

            assert_eq!(events.len(), 2);
            assert_eq!(events[0].block_number, "0x1");
            assert_eq!(events[1].block_number, "0x2");
            assert_eq!(events[0].block_timestamp.as_deref(), Some("0x64"));
            assert_eq!(events[1].block_timestamp.as_deref(), Some("0x65"));
        }

        #[tokio::test]
        async fn fetch_store_set_events_returns_empty_for_no_addresses() {
            let db = LocalDb::default();
            let events = db
                .fetch_store_set_events(&[], 0, 10, &FetchConfig::default())
                .await
                .unwrap();
            assert!(events.is_empty());
        }

        #[tokio::test]
        async fn backfill_missing_timestamps_adds_missing_blocks() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            server.mock(|when, then| {
                when.method(POST).path("/").json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "eth_getBlockByNumber",
                    "params": ["0x1", false]
                }));
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x1", Some("0x10")));
            });

            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let mut events = vec![
                make_log_entry_basic("0x1", None),
                make_log_entry_basic("0x2", Some("0x20")),
            ];

            db.backfill_missing_timestamps(&mut events, &FetchConfig::default())
                .await
                .unwrap();

            assert_eq!(events[0].block_timestamp.as_deref(), Some("0x10"));
            assert_eq!(events[1].block_timestamp.as_deref(), Some("0x20"));
        }
    }
}
