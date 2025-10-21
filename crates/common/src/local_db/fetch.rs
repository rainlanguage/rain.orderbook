use super::{LocalDb, LocalDbError};
use crate::rpc_client::{BlockRange, LogEntryResponse, RpcClientError, Topics};
use alloy::primitives::{Address, U256};
use backon::{ConstantBuilder, Retryable};
use futures::{StreamExt, TryStreamExt};
use rain_orderbook_bindings::topics::{orderbook_event_topics, store_set_topics};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

#[derive(Debug, Clone)]
struct LogFilter {
    pub addresses: Vec<Address>,
    pub topics: Topics,
    pub range: BlockRange,
}

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
    async fn collect_logs(
        &self,
        filter: &LogFilter,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        if filter.range.start > filter.range.end {
            return Err(LocalDbError::Rpc(RpcClientError::InvalidBlockRange {
                start: filter.range.start,
                end: filter.range.end,
            }));
        }

        if filter.addresses.is_empty() {
            return Ok(Vec::new());
        }

        let unique_addresses = Self::dedupe_addresses(&filter.addresses);
        if unique_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let filter = LogFilter {
            addresses: unique_addresses,
            topics: filter.topics.clone(),
            range: filter.range,
        };

        let jobs = Self::build_log_jobs(&filter.addresses, filter.range, config)?;
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        let mut events = self
            .fetch_logs_for_jobs(jobs, &filter.topics, config)
            .await?;
        Self::sort_events_by_block_and_log(&mut events);
        self.backfill_missing_timestamps(&mut events, config)
            .await?;
        Ok(events)
    }

    pub async fn fetch_orderbook_events(
        &self,
        address: Address,
        range: BlockRange,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        let filter = LogFilter {
            addresses: vec![address],
            topics: Topics::from_b256_list(orderbook_event_topics()),
            range,
        };
        self.collect_logs(&filter, config).await
    }

    pub async fn fetch_store_events(
        &self,
        addresses: &[Address],
        range: BlockRange,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        let filter = LogFilter {
            addresses: addresses.to_vec(),
            topics: Topics::from_b256_list(store_set_topics()),
            range,
        };
        self.collect_logs(&filter, config).await
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

    fn dedupe_addresses(addresses: &[Address]) -> Vec<Address> {
        let mut dedup = HashSet::new();
        addresses
            .iter()
            .copied()
            .filter(|addr| dedup.insert(*addr))
            .collect()
    }

    fn build_log_jobs(
        addresses: &[Address],
        range: BlockRange,
        config: &FetchConfig,
    ) -> Result<Vec<LogFetchJob>, LocalDbError> {
        let chunk_size = config.chunk_size.max(1);
        let chunk_span = chunk_size.saturating_sub(1);
        let mut jobs = Vec::new();

        for &address in addresses {
            let mut current_block = range.start;
            while current_block <= range.end {
                let to_block = current_block.saturating_add(chunk_span).min(range.end);
                let chunk_range = BlockRange::inclusive(current_block, to_block)?;
                jobs.push(LogFetchJob {
                    address,
                    range: chunk_range,
                });

                if to_block == range.end || to_block == u64::MAX {
                    break;
                }

                current_block = to_block.saturating_add(1);
            }
        }

        Ok(jobs)
    }

    async fn fetch_logs_for_jobs(
        &self,
        jobs: Vec<LogFetchJob>,
        topics: &Topics,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        let concurrency = config.max_concurrent_requests.max(1);
        let client = self.rpc_client().clone();
        let results: Vec<Vec<LogEntryResponse>> = futures::stream::iter(jobs)
            .map(|job| {
                let topics = topics.clone();
                let client = client.clone();
                let max_attempts = config.max_retry_attempts;

                async move {
                    let response = retry_with_attempts(
                        || client.get_logs(job.address, &topics, job.range),
                        max_attempts,
                    )
                    .await?;

                    Ok::<_, LocalDbError>(response)
                }
            })
            .buffer_unordered(concurrency)
            .try_collect()
            .await?;

        Ok(results.into_iter().flatten().collect())
    }

    fn sort_events_by_block_and_log(events: &mut [LogEntryResponse]) {
        events.sort_by(|a, b| {
            let block_a = extract_block_number_from_entry(a).unwrap_or(0);
            let block_b = extract_block_number_from_entry(b).unwrap_or(0);
            let log_a = parse_block_number_str(&a.log_index).unwrap_or(0);
            let log_b = parse_block_number_str(&b.log_index).unwrap_or(0);
            block_a.cmp(&block_b).then_with(|| log_a.cmp(&log_b))
        });
    }
}

const RETRY_DELAY_MILLIS: u64 = 100;

#[derive(Clone)]
struct LogFetchJob {
    address: Address,
    range: BlockRange,
}

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
            return Err(LocalDbError::InvalidBlockNumber {
                value: block_number_str.to_string(),
                source: alloy::primitives::ruint::ParseError::InvalidDigit('\0'),
            });
        }
        U256::from_str_radix(hex_digits, 16).map_err(|e| LocalDbError::InvalidBlockNumber {
            value: block_number_str.to_string(),
            source: e,
        })?
    } else {
        U256::from_str_radix(block_number_str, 10).map_err(|e| {
            LocalDbError::InvalidBlockNumber {
                value: block_number_str.to_string(),
                source: e,
            }
        })?
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
        use crate::rpc_client::BlockRange;
        use alloy::hex;
        use alloy::primitives::Address;
        use alloy::sol_types::SolEvent;
        use httpmock::prelude::*;
        use rain_orderbook_bindings::{IInterpreterStoreV3::Set, IOrderBookV5::AddOrderV3};
        use serde_json::json;
        use std::str::FromStr;
        use std::sync::atomic::{AtomicUsize, Ordering};
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
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": block
            })
            .to_string()
        }

        #[tokio::test]
        async fn fetch_orderbook_events_fetches_and_sorts() {
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

            let range = BlockRange::inclusive(1, 2).expect("valid range");
            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let events = db
                .fetch_orderbook_events(
                    addr,
                    range,
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
        async fn fetch_store_events_returns_empty_for_no_addresses() {
            let db = LocalDb::default();
            let range = BlockRange::inclusive(0, 10).expect("valid range");
            let addresses: Vec<Address> = vec![];
            let events = db
                .fetch_store_events(&addresses, range, &FetchConfig::default())
                .await
                .unwrap();
            assert!(events.is_empty());
        }

        #[tokio::test]
        async fn fetch_store_events_handles_duplicates_and_sorts() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            let logs_response = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": [
                    {
                        "address": "0xstore",
                        "topics": [Set::SIGNATURE_HASH.to_string()],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x2",
                        "blockTimestamp": "0x65",
                        "transactionHash": "0x2",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x1",
                        "removed": false
                    },
                    {
                        "address": "0xstore",
                        "topics": [Set::SIGNATURE_HASH.to_string()],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x1",
                        "blockTimestamp": "0x64",
                        "transactionHash": "0x1",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x0",
                        "removed": false
                    }
                ]
            });

            let addr = Address::from_str("0x0000000000000000000000000000000000000aBc").unwrap();
            let expected_addr = format!("{:#x}", addr);
            let log_mock = server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getLogs\"")
                    .body_contains(&expected_addr);
                then.status(200)
                    .header("content-type", "application/json")
                    .body(logs_response.to_string());
            });

            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let range = BlockRange::inclusive(1, 2).expect("valid range");
            let addresses = vec![addr, addr, addr];
            let events = db
                .fetch_store_events(
                    &addresses,
                    range,
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
            assert_eq!(events[0].log_index, "0x0");
            assert_eq!(events[1].block_number, "0x2");
            assert_eq!(events[1].log_index, "0x1");
            assert_eq!(events[0].block_timestamp.as_deref(), Some("0x64"));
            assert_eq!(events[1].block_timestamp.as_deref(), Some("0x65"));
            assert_eq!(log_mock.hits(), 1);
        }

        #[tokio::test]
        async fn fetch_store_events_returns_error_for_inverted_range() {
            let db = LocalDb::default();
            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let range = BlockRange { start: 10, end: 1 };
            let err = db
                .fetch_store_events(&[addr], range, &FetchConfig::default())
                .await
                .unwrap_err();
            match err {
                LocalDbError::Rpc(RpcClientError::InvalidBlockRange { start, end }) => {
                    assert_eq!(start, 10);
                    assert_eq!(end, 1);
                }
                other => panic!("expected InvalidBlockRange, got {other:?}"),
            }
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

        #[tokio::test]
        async fn fetch_orderbook_events_returns_error_for_inverted_range() {
            let db = LocalDb::default();
            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let range = BlockRange { start: 10, end: 1 };
            let err = db
                .fetch_orderbook_events(addr, range, &FetchConfig::default())
                .await
                .unwrap_err();
            match err {
                LocalDbError::Rpc(RpcClientError::InvalidBlockRange { start, end }) => {
                    assert_eq!(start, 10);
                    assert_eq!(end, 1);
                }
                other => panic!("expected InvalidBlockRange, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn fetch_orderbook_events_sorts_numeric_log_index_within_block() {
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
                        "blockNumber": "0x1",
                        "transactionHash": "0xtx1",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x10",
                        "removed": false
                    },
                    {
                        "address": "0xabc",
                        "topics": [
                            format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))
                        ],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x1",
                        "transactionHash": "0xtx2",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0x2",
                        "removed": false
                    },
                    {
                        "address": "0xabc",
                        "topics": [
                            format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))
                        ],
                        "data": "0xdeadbeef",
                        "blockNumber": "0x1",
                        "transactionHash": "0xtx3",
                        "transactionIndex": "0x0",
                        "blockHash": "0x0",
                        "logIndex": "0xA",
                        "removed": false
                    }
                ]
            });

            // Mock eth_getLogs
            server.mock(|when, then| {
                when.method(POST).path("/").body_contains("\"eth_getLogs\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response_body.to_string());
            });

            // Backfill timestamp for block 0x1
            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getBlockByNumber\"")
                    .body_contains("\"0x1\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x1", Some("0x64")));
            });

            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let range = BlockRange::inclusive(1, 1).expect("valid");
            let events = db
                .fetch_orderbook_events(
                    addr,
                    range,
                    &FetchConfig {
                        chunk_size: 1000,
                        max_concurrent_requests: 1,
                        max_concurrent_blocks: 1,
                        max_retry_attempts: 1,
                    },
                )
                .await
                .unwrap();

            assert_eq!(events.len(), 3);
            // Expect numeric sort of logIndex within the same block: 0x2, 0xA, 0x10
            assert_eq!(events[0].log_index, "0x2");
            assert_eq!(events[1].log_index, "0xA");
            assert_eq!(events[2].log_index, "0x10");
            assert!(events.iter().all(|e| e.block_number == "0x1"));
        }

        #[tokio::test]
        async fn retry_with_attempts_retries_on_rpc_then_succeeds() {
            let attempts = AtomicUsize::new(0);
            let res: Result<u32, LocalDbError> = retry_with_attempts(
                || async {
                    let i = attempts.fetch_add(1, Ordering::SeqCst);
                    if i == 0 {
                        Err(RpcClientError::RpcError {
                            message: "first fail".into(),
                        })
                    } else {
                        Ok(42u32)
                    }
                },
                3,
            )
            .await;

            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 42);
            assert_eq!(
                attempts.load(Ordering::SeqCst),
                2,
                "should retry exactly once"
            );
        }

        #[tokio::test]
        async fn retry_with_attempts_gives_up_after_max_attempts() {
            let attempts = AtomicUsize::new(0);
            let err = retry_with_attempts(
                || async {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(RpcClientError::RpcError {
                        message: "always fail".into(),
                    })
                },
                2,
            )
            .await
            .unwrap_err();

            match err {
                LocalDbError::Rpc(RpcClientError::RpcError { .. }) => {}
                other => panic!("expected LocalDbError::Rpc, got {other:?}"),
            }
            assert_eq!(attempts.load(Ordering::SeqCst), 2, "should attempt twice");
        }

        #[tokio::test]
        async fn retry_with_attempts_does_not_retry_on_json_parse_error() {
            let attempts = AtomicUsize::new(0);
            let err = retry_with_attempts(
                || async {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    let e = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
                    Err::<(), _>(RpcClientError::JsonSerialization(e))
                },
                3,
            )
            .await
            .unwrap_err();

            match err {
                LocalDbError::JsonParse(_) => {}
                other => panic!("expected LocalDbError::JsonParse, got {other:?}"),
            }
            assert_eq!(
                attempts.load(Ordering::SeqCst),
                1,
                "should not retry json errors"
            );
        }

        #[tokio::test]
        async fn retry_with_attempts_zero_attempts_is_config_error() {
            let attempts = AtomicUsize::new(0);
            let err = retry_with_attempts(
                || async {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Ok::<u32, RpcClientError>(1)
                },
                0,
            )
            .await
            .unwrap_err();

            match err {
                LocalDbError::Config { .. } => {}
                other => panic!("expected LocalDbError::Config, got {other:?}"),
            }
            assert_eq!(attempts.load(Ordering::SeqCst), 0, "operation must not run");
        }

        #[test]
        fn parse_block_number_str_variants() {
            // hex lower/upper and decimal
            assert_eq!(parse_block_number_str("0xA").unwrap(), 10);
            assert_eq!(parse_block_number_str("0X10").unwrap(), 16);
            assert_eq!(parse_block_number_str("42").unwrap(), 42);
        }

        #[test]
        fn parse_block_number_str_invalid_inputs() {
            // invalid hex prefix only
            match parse_block_number_str("0x").unwrap_err() {
                LocalDbError::InvalidBlockNumber { .. } => {}
                other => panic!("expected InvalidBlockNumber, got {other:?}"),
            }

            // invalid decimal
            match parse_block_number_str("notanumber").unwrap_err() {
                LocalDbError::InvalidBlockNumber { .. } => {}
                other => panic!("expected InvalidBlockNumber, got {other:?}"),
            }
        }

        #[test]
        fn build_store_jobs_chunking_basic() {
            let jobs = LocalDb::build_log_jobs(
                &[Address::ZERO],
                BlockRange::inclusive(1, 10).expect("valid range"),
                &FetchConfig {
                    chunk_size: 3,
                    ..Default::default()
                },
            )
            .unwrap();

            assert_eq!(jobs.len(), 4);
            assert_eq!(jobs[0].range.start, 1);
            assert_eq!(jobs[0].range.end, 3);
            assert_eq!(jobs[1].range.start, 4);
            assert_eq!(jobs[1].range.end, 6);
            assert_eq!(jobs[2].range.start, 7);
            assert_eq!(jobs[2].range.end, 9);
            assert_eq!(jobs[3].range.start, 10);
            assert_eq!(jobs[3].range.end, 10);
            assert!(jobs.iter().all(|j| j.address == Address::ZERO));
        }

        #[test]
        fn build_store_jobs_handles_u64_max_boundary() {
            let start = u64::MAX - 50;
            let end = u64::MAX;
            let jobs = LocalDb::build_log_jobs(
                &[Address::ZERO],
                BlockRange::inclusive(start, end).expect("valid range"),
                &FetchConfig {
                    chunk_size: 100,
                    ..Default::default()
                },
            )
            .unwrap();

            // Should create a single job and not overflow or loop
            assert_eq!(jobs.len(), 1);
            assert_eq!(jobs[0].range.start, start);
            assert_eq!(jobs[0].range.end, end);
        }

        #[tokio::test]
        async fn fetch_block_timestamps_empty_input_returns_empty_map() {
            let db = LocalDb::default();
            let map = db
                .fetch_block_timestamps(vec![], &FetchConfig::default())
                .await
                .unwrap();
            assert!(map.is_empty());
        }

        #[tokio::test]
        async fn fetch_block_timestamps_missing_result_field() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getBlockByNumber\"")
                    .body_contains("\"0x1\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(json!({"jsonrpc":"2.0","id":1,"result":null}).to_string());
            });

            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let err = db
                .fetch_block_timestamps(
                    vec![1],
                    &FetchConfig {
                        max_retry_attempts: 1,
                        ..Default::default()
                    },
                )
                .await
                .unwrap_err();

            match err {
                LocalDbError::MissingField { field } => assert_eq!(field, "result"),
                other => panic!("expected MissingField('result'), got {other:?}"),
            }
        }

        #[tokio::test]
        async fn fetch_block_timestamps_missing_timestamp_maps_to_json_error() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            // Respond with a block object missing the required `timestamp` field
            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getBlockByNumber\"")
                    .body_contains("\"0x1\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(sample_block_response("0x1", None));
            });

            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let err = db
                .fetch_block_timestamps(
                    vec![1],
                    &FetchConfig {
                        max_retry_attempts: 1,
                        ..Default::default()
                    },
                )
                .await
                .unwrap_err();

            match err {
                LocalDbError::JsonParse(_) => {}
                other => panic!("expected JsonParse, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn fetch_store_events_chunking_multiple_jobs_merged_and_sorted() {
            let server = MockServer::start();
            let url = Url::from_str(&server.url("/")).unwrap();

            // Job 1: from 0x1 to 0x1
            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getLogs\"")
                    .body_contains("\"0x1\"")
                    .body_contains("\"0x1\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(
                        json!({
                            "jsonrpc":"2.0",
                            "id":1,
                            "result":[{
                                "address":"0xstore",
                                "topics":[Set::SIGNATURE_HASH.to_string()],
                                "data":"0xdeadbeef",
                                "blockNumber":"0x1",
                                "blockTimestamp":"0x64",
                                "transactionHash":"0xtx1",
                                "transactionIndex":"0x0",
                                "blockHash":"0x0",
                                "logIndex":"0x1",
                                "removed":false
                            }]
                        })
                        .to_string(),
                    );
            });

            // Job 2: from 0x2 to 0x2
            server.mock(|when, then| {
                when.method(POST)
                    .path("/")
                    .body_contains("\"eth_getLogs\"")
                    .body_contains("\"0x2\"")
                    .body_contains("\"0x2\"");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(
                        json!({
                            "jsonrpc":"2.0",
                            "id":1,
                            "result":[{
                                "address":"0xstore",
                                "topics":[Set::SIGNATURE_HASH.to_string()],
                                "data":"0xdeadbeef",
                                "blockNumber":"0x2",
                                "blockTimestamp":"0x65",
                                "transactionHash":"0xtx2",
                                "transactionIndex":"0x0",
                                "blockHash":"0x0",
                                "logIndex":"0x0",
                                "removed":false
                            }]
                        })
                        .to_string(),
                    );
            });

            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let mut db = LocalDb::new_with_regular_rpc(url).unwrap();
            db.update_rpc_urls(vec![Url::from_str(&server.url("/")).unwrap()]);

            let range = BlockRange::inclusive(1, 2).expect("valid");
            let events = db
                .fetch_store_events(
                    &[addr],
                    range,
                    &FetchConfig {
                        chunk_size: 1, // forces per-block jobs
                        max_concurrent_requests: 2,
                        max_concurrent_blocks: 1,
                        max_retry_attempts: 1,
                    },
                )
                .await
                .unwrap();

            assert_eq!(events.len(), 2);
            assert_eq!(events[0].block_number, "0x1");
            assert_eq!(events[0].log_index, "0x1");
            assert_eq!(events[1].block_number, "0x2");
            assert_eq!(events[1].log_index, "0x0");
            assert_eq!(events[0].block_timestamp.as_deref(), Some("0x64"));
            assert_eq!(events[1].block_timestamp.as_deref(), Some("0x65"));
        }
    }
}
