use super::{LocalDb, LocalDbError};
use crate::{
    retry::retry_with_backoff,
    rpc_client::{LogEntryResponse, RpcClientError},
};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Filter;
use futures::{StreamExt, TryStreamExt};
use rain_orderbook_bindings::topics::{ORDERBOOK_EVENT_TOPICS, STORE_SET_TOPICS};
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

impl LocalDb {
    async fn collect_logs(
        &self,
        filter: &Filter,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        filter.block_option.ensure_valid_block_range()?;

        if filter.address.is_empty() {
            return Ok(Vec::new());
        }

        let filters = Self::build_log_filters(filter, config)?;
        if filters.is_empty() {
            return Ok(Vec::new());
        }

        let mut events = self.fetch_logs_for_filters(filters, config).await?;

        Self::sort_events_by_block_and_log(&mut events);
        self.backfill_missing_timestamps(&mut events, config)
            .await?;

        Ok(events)
    }

    pub async fn fetch_orderbook_events(
        &self,
        address: Address,
        from_block: u64,
        to_block: u64,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        let filter = Filter::new()
            .address(address)
            .from_block(from_block)
            .to_block(to_block)
            .event_signature(ORDERBOOK_EVENT_TOPICS.to_vec());
        self.collect_logs(&filter, config).await
    }

    pub async fn fetch_store_events(
        &self,
        addresses: &[Address],
        from_block: u64,
        to_block: u64,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        let filter = Filter::new()
            .address(addresses.to_vec())
            .from_block(from_block)
            .to_block(to_block)
            .event_signature(STORE_SET_TOPICS.to_vec());
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
                        let block_response = retry_with_backoff(
                            || {
                                let client = client.clone();
                                async move {
                                    client
                                        .get_block_by_number(block_number)
                                        .await
                                        .map_err(map_rpc_error)
                                }
                            },
                            max_attempts,
                            should_retry_local_db_error,
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

    fn build_log_filters(
        filter: &Filter,
        config: &FetchConfig,
    ) -> Result<Vec<Filter>, LocalDbError> {
        let chunk_size = config.chunk_size.max(1);
        let chunk_span = chunk_size.saturating_sub(1);
        let mut filters = Vec::new();

        let from_block = filter
            .block_option
            .get_from_block()
            .ok_or(LocalDbError::MissingBlockFilter("from".to_string()))?
            .as_number()
            .ok_or(LocalDbError::NonNumberBlockNumber("from".to_string()))?;
        let to_block = filter
            .block_option
            .get_to_block()
            .ok_or(LocalDbError::MissingBlockFilter("to".to_string()))?
            .as_number()
            .ok_or(LocalDbError::NonNumberBlockNumber("to".to_string()))?;

        for &address in filter.address.iter() {
            let mut current_block = from_block;
            while current_block <= to_block {
                let end = current_block.saturating_add(chunk_span).min(to_block);

                let all_topics = filter.topics.clone();
                let topics = all_topics
                    .first()
                    .ok_or(LocalDbError::MissingTopicsFilter)?;

                let mut filter = Filter::new()
                    .address(address)
                    .from_block(current_block)
                    .to_block(end);
                filter = filter.event_signature(topics.clone());
                filters.push(filter);

                if end == to_block || end == u64::MAX {
                    break;
                }

                current_block = end.saturating_add(1);
            }
        }

        Ok(filters)
    }

    async fn fetch_logs_for_filters(
        &self,
        filters: Vec<Filter>,
        config: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        if filters.is_empty() {
            return Ok(Vec::new());
        }

        let concurrency = config.max_concurrent_requests.max(1);
        let client = self.rpc_client().clone();
        let results: Vec<Vec<LogEntryResponse>> = futures::stream::iter(filters)
            .map(|filter| {
                let client = client.clone();
                let max_attempts = config.max_retry_attempts;

                async move {
                    let response = retry_with_backoff(
                        || {
                            let client = client.clone();
                            let filter = filter.clone();
                            async move { client.get_logs(&filter).await.map_err(map_rpc_error) }
                        },
                        max_attempts,
                        should_retry_local_db_error,
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

fn map_rpc_error(error: RpcClientError) -> LocalDbError {
    match error {
        RpcClientError::JsonSerialization(err) => LocalDbError::JsonParse(err),
        other => LocalDbError::Rpc(other),
    }
}

fn should_retry_local_db_error(error: &LocalDbError) -> bool {
    matches!(error, LocalDbError::Rpc(_))
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
        use alloy::hex;
        use alloy::primitives::Address;
        use alloy::rpc::types::FilterBlockError;
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

            let addr = Address::from_str("0x0000000000000000000000000000000000000abc").unwrap();
            let events = db
                .fetch_orderbook_events(
                    addr,
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
        async fn fetch_store_events_returns_empty_for_no_addresses() {
            let db = LocalDb::default();
            let addresses: Vec<Address> = vec![];
            let events = db
                .fetch_store_events(&addresses, 0, 10, &FetchConfig::default())
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

            let addresses = vec![addr, addr, addr];
            let events = db
                .fetch_store_events(
                    &addresses,
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
            let err = db
                .fetch_store_events(&[addr], 10, 1, &FetchConfig::default())
                .await
                .unwrap_err();
            match err {
                LocalDbError::FilterBlockError(FilterBlockError::FromBlockGreaterThanToBlock {
                    from,
                    to,
                }) => {
                    assert_eq!(from, 10);
                    assert_eq!(to, 1);
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
            let err = db
                .fetch_orderbook_events(addr, 10, 1, &FetchConfig::default())
                .await
                .unwrap_err();
            match err {
                LocalDbError::FilterBlockError(FilterBlockError::FromBlockGreaterThanToBlock {
                    from,
                    to,
                }) => {
                    assert_eq!(from, 10);
                    assert_eq!(to, 1);
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

            let events = db
                .fetch_orderbook_events(
                    addr,
                    1,
                    1,
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
        async fn retry_helper_returns_rpc_error_after_exhaustion() {
            let attempts = AtomicUsize::new(0);
            let err = retry_with_backoff(
                || async {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(map_rpc_error(RpcClientError::RpcError {
                        message: "always fail".into(),
                    }))
                },
                2,
                should_retry_local_db_error,
            )
            .await
            .unwrap_err();

            let err: LocalDbError = err.into();
            match err {
                LocalDbError::Rpc(RpcClientError::RpcError { .. }) => {}
                other => panic!("expected LocalDbError::Rpc, got {other:?}"),
            }
            assert_eq!(attempts.load(Ordering::SeqCst), 2, "should attempt twice");
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
        fn build_store_filters_chunking_basic() {
            let filter = Filter::new()
                .address(vec![Address::ZERO])
                .from_block(1)
                .to_block(10)
                .event_signature(STORE_SET_TOPICS.to_vec());

            let filters = LocalDb::build_log_filters(
                &filter,
                &FetchConfig {
                    chunk_size: 3,
                    ..Default::default()
                },
            )
            .unwrap();

            assert_eq!(filters.len(), 4);
            assert_eq!(filters[0].get_from_block().unwrap(), 1);
            assert_eq!(filters[0].get_to_block().unwrap(), 3);
            assert_eq!(filters[1].get_from_block().unwrap(), 4);
            assert_eq!(filters[1].get_to_block().unwrap(), 6);
            assert_eq!(filters[2].get_from_block().unwrap(), 7);
            assert_eq!(filters[2].get_to_block().unwrap(), 9);
            assert_eq!(filters[3].get_from_block().unwrap(), 10);
            assert_eq!(filters[3].get_to_block().unwrap(), 10);
        }

        #[test]
        fn build_store_filters_handles_u64_max_boundary() {
            let start = u64::MAX - 50;
            let end = u64::MAX;

            let filter = Filter::new()
                .address(vec![Address::ZERO])
                .from_block(start)
                .to_block(end)
                .event_signature(STORE_SET_TOPICS.to_vec());

            let filters = LocalDb::build_log_filters(
                &filter,
                &FetchConfig {
                    chunk_size: 100,
                    ..Default::default()
                },
            )
            .unwrap();

            // Should create a single job and not overflow or loop
            assert_eq!(filters.len(), 1);
            assert_eq!(filters[0].get_from_block().unwrap(), start);
            assert_eq!(filters[0].get_to_block().unwrap(), end);
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
        async fn fetch_store_events_chunking_multiple_filters_merged_and_sorted() {
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

            let events = db
                .fetch_store_events(
                    &[addr],
                    1,
                    2,
                    &FetchConfig {
                        chunk_size: 1, // forces per-block filters
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
