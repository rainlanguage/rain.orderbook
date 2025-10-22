use anyhow::Result;
use rain_orderbook_common::local_db::decode::{DecodedEvent, DecodedEventData};
use url::Url;

use super::{
    data_source::{SyncDataSource, TokenMetadataFetcher},
    storage::{ensure_schema, fetch_existing_store_addresses},
};
use crate::commands::local_db::executor::RusqliteExecutor;
use rain_orderbook_common::local_db::query::LocalDbQueryExecutor;

use self::{
    apply::{
        decode_events, fetch_events, prepare_sql, DecodedEvents, FetchResult, PrepareSqlParams,
    },
    window::compute_sync_window,
};

use rain_orderbook_common::local_db::tokens::collect_store_addresses;
use std::collections::BTreeSet;

mod apply;
mod window;

pub(crate) struct SyncRunner<'a, D, T> {
    db_path: &'a str,
    data_source: &'a D,
    metadata_rpc_urls: Vec<Url>,
    token_fetcher: &'a T,
}

pub(crate) struct SyncParams<'a> {
    pub(crate) chain_id: u32,
    pub(crate) orderbook_address: &'a str,
    pub(crate) deployment_block: u64,
    pub(crate) start_block: Option<u64>,
    pub(crate) end_block: Option<u64>,
}

impl<'a, D, T> SyncRunner<'a, D, T>
where
    D: SyncDataSource + Send + Sync,
    T: TokenMetadataFetcher + Send + Sync,
{
    pub(crate) fn new(
        db_path: &'a str,
        data_source: &'a D,
        metadata_rpc_urls: Vec<Url>,
        token_fetcher: &'a T,
    ) -> Self {
        Self {
            db_path,
            data_source,
            metadata_rpc_urls,
            token_fetcher,
        }
    }

    pub(crate) async fn run(&self, params: &SyncParams<'_>) -> Result<()> {
        let schema_applied = ensure_schema(self.db_path).await?;
        if schema_applied {
            println!("Database schema initialized at {}", self.db_path);
        }

        let window = compute_sync_window(self.db_path, self.data_source, params).await?;
        println!("Current last_synced_block: {}", window.last_synced_block);
        if let Some(adjustment) = &window.start_adjustment {
            println!("{}", adjustment.message(window.last_synced_block));
        }
        println!("Network latest block: {}", window.latest_block);
        if let Some(clamp) = &window.end_clamp {
            println!("{}", clamp.message());
        }
        if window.noop {
            println!(
                "Nothing to do (start block {} exceeds target block {})",
                window.start_block, window.target_block
            );
            return Ok(());
        }

        println!(
            "Fetching events for {} from block {} to {}",
            params.orderbook_address, window.start_block, window.target_block
        );
        let FetchResult { events, raw_count } = fetch_events(
            self.data_source,
            params.orderbook_address,
            window.start_block,
            window.target_block,
        )
        .await?;
        println!("Fetched {} raw events", raw_count);

        let mut raw_events = events.clone();

        println!("Decoding events");
        let DecodedEvents {
            decoded: mut decoded_events,
            mut decoded_count,
        } = decode_events(self.data_source, events)?;
        println!("Decoded {} events", decoded_count);

        println!("Collecting interpreter store addresses");
        let mut store_addresses: BTreeSet<String> = collect_store_addresses(&decoded_events);
        let existing_stores = fetch_existing_store_addresses(self.db_path).await?;
        store_addresses.extend(existing_stores);

        if !store_addresses.is_empty() {
            let store_list: Vec<String> = store_addresses.into_iter().collect();
            println!(
                "Fetching interpreter store Set events for {} store(s)",
                store_list.len()
            );
            let store_events = self
                .data_source
                .fetch_store_set_events(&store_list, window.start_block, window.target_block)
                .await?;
            println!(
                "Fetched {} interpreter store Set events",
                store_events.len()
            );

            if !store_events.is_empty() {
                raw_events.extend(store_events.iter().cloned());
                let mut decoded_store = self.data_source.decode_events(&store_events)?;
                decoded_events.append(&mut decoded_store);
                sort_events_by_block_and_log(&mut decoded_events);
                decoded_count = decoded_events.len();
                println!("Decoded {} total events", decoded_count);
            }
        }

        println!("Preparing token metadata");
        let sql_batch = prepare_sql(
            self.data_source,
            self.token_fetcher,
            PrepareSqlParams {
                db_path: self.db_path.to_string(),
                metadata_rpc_urls: self.metadata_rpcs().to_vec(),
                chain_id: params.chain_id,
                decoded_events,
                raw_events,
                target_block: window.target_block,
            },
        )
        .await?;

        println!("Generating SQL for {} events", decoded_count);
        println!("Applying SQL to {}", self.db_path);
        let exec = RusqliteExecutor::new(self.db_path);
        exec.execute_batch(&sql_batch)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        println!(
            "Sync complete. last_synced_block is now {}",
            window.target_block
        );
        Ok(())
    }

    fn metadata_rpcs(&self) -> &[Url] {
        if self.metadata_rpc_urls.is_empty() {
            self.data_source.rpc_urls()
        } else {
            &self.metadata_rpc_urls
        }
    }
}

fn sort_events_by_block_and_log(events: &mut [DecodedEventData<DecodedEvent>]) {
    events.sort_by(|a, b| {
        let block_a = parse_block_number(&a.block_number);
        let block_b = parse_block_number(&b.block_number);
        block_a
            .cmp(&block_b)
            .then_with(|| parse_block_number(&a.log_index).cmp(&parse_block_number(&b.log_index)))
    });
}

fn parse_block_number(value: &str) -> u64 {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        trimmed.parse::<u64>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, FixedBytes, U256};
    use async_trait::async_trait;
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;
    use rain_orderbook_common::erc20::TokenInfo;
    use rain_orderbook_common::local_db::decode::{
        DecodedEvent, DecodedEventData, EventType, InterpreterStoreSetEvent,
    };
    use rain_orderbook_common::local_db::query::{SqlStatement, SqlStatementBatch};
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::TempDir;

    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;

    const RAW_SQL_STUB: &str = r#"INSERT INTO raw_events (
        block_number,
        block_timestamp,
        transaction_hash,
        log_index,
        address,
        topics,
        data,
        raw_json
    ) VALUES (
        0,
        NULL,
        '0x0',
        0,
        '0x0',
        '[]',
        '0x',
        '{}'
    );
"#;

    struct TestFetcher {
        metadata: Vec<(Address, TokenInfo)>,
        calls: Mutex<Vec<Vec<Address>>>,
    }

    #[async_trait]
    impl TokenMetadataFetcher for TestFetcher {
        async fn fetch(
            &self,
            _: &[Url],
            missing: Vec<Address>,
        ) -> Result<Vec<(Address, TokenInfo)>> {
            self.calls.lock().unwrap().push(missing.clone());
            Ok(self.metadata.clone())
        }
    }

    struct TestDataSource {
        latest_block: u64,
        rpc_urls: Vec<Url>,
        fetch_logs: Vec<LogEntryResponse>,
        store_logs: Vec<LogEntryResponse>,
        decode_responses: Mutex<Vec<Vec<DecodedEventData<DecodedEvent>>>>,
        sql_result: String,
        fetch_calls: Mutex<Vec<(String, u64, u64)>>,
        fetch_store_calls: Mutex<Vec<(Vec<String>, u64, u64)>>,
        sql_calls: Mutex<Vec<usize>>,
        decimals: Mutex<Vec<HashMap<Address, u8>>>,
        raw_statements: Vec<SqlStatement>,
        raw_calls: Mutex<Vec<Vec<LogEntryResponse>>>,
    }

    #[async_trait]
    impl SyncDataSource for TestDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(self.latest_block)
        }

        async fn fetch_events(
            &self,
            orderbook_address: &str,
            start_block: u64,
            end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            self.fetch_calls.lock().unwrap().push((
                orderbook_address.to_string(),
                start_block,
                end_block,
            ));
            Ok(self.fetch_logs.clone())
        }

        async fn fetch_store_set_events(
            &self,
            store_addresses: &[String],
            start_block: u64,
            end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            self.fetch_store_calls.lock().unwrap().push((
                store_addresses.to_vec(),
                start_block,
                end_block,
            ));
            Ok(self.store_logs.clone())
        }

        fn decode_events(
            &self,
            _events: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
            let mut guard = self.decode_responses.lock().unwrap();
            if guard.is_empty() {
                return Ok(vec![]);
            }
            Ok(guard.remove(0))
        }

        fn events_to_sql(
            &self,
            decoded_events: &[DecodedEventData<DecodedEvent>],
            decimals_by_token: &HashMap<Address, u8>,
        ) -> Result<SqlStatementBatch> {
            self.sql_calls.lock().unwrap().push(decoded_events.len());
            self.decimals
                .lock()
                .unwrap()
                .push(decimals_by_token.clone());

            let mut statements = Vec::new();
            if !self.sql_result.is_empty() {
                statements.push(SqlStatement::new(self.sql_result.clone()));
            }
            Ok(SqlStatementBatch::from(statements))
        }

        fn raw_events_to_statements(
            &self,
            raw_events: &[LogEntryResponse],
        ) -> Result<SqlStatementBatch> {
            self.raw_calls.lock().unwrap().push(raw_events.to_vec());
            Ok(SqlStatementBatch::from(self.raw_statements.clone()))
        }

        fn rpc_urls(&self) -> &[Url] {
            &self.rpc_urls
        }
    }

    fn sample_decoded_event(token: Address) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: EventType::DepositV2,
            block_number: "0x1".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0xabc".into(),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                sender: Address::from([0x11; 20]),
                token,
                vaultId: U256::from(1).into(),
                depositAmountUint256: U256::from(5),
            })),
        }
    }

    fn sample_store_decoded_event(store: Address) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: EventType::InterpreterStoreSet,
            block_number: "0x2".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0xstore".into(),
            log_index: "0x1".into(),
            decoded_data: DecodedEvent::InterpreterStoreSet(Box::new(InterpreterStoreSetEvent {
                store_address: store,
                namespace: FixedBytes::from([0xaa; 32]),
                key: FixedBytes::from([0xbb; 32]),
                value: FixedBytes::from([0xcc; 32]),
            })),
        }
    }

    fn sample_log() -> LogEntryResponse {
        LogEntryResponse {
            address: "0xfeed".into(),
            topics: vec!["0x0".into()],
            data: "0x".into(),
            block_number: "0x1".into(),
            block_timestamp: Some("0x0".into()),
            transaction_hash: "0xabc".into(),
            transaction_index: "0x0".into(),
            block_hash: "0x123".into(),
            log_index: "0x0".into(),
            removed: false,
        }
    }

    fn sample_store_log() -> LogEntryResponse {
        LogEntryResponse {
            address: "0xdead".into(),
            topics: vec!["0x0".into()],
            data: "0x".into(),
            block_number: "0x2".into(),
            block_timestamp: Some("0x0".into()),
            transaction_hash: "0xstore".into(),
            transaction_index: "0x0".into(),
            block_hash: "0x456".into(),
            log_index: "0x1".into(),
            removed: false,
        }
    }

    #[tokio::test]
    async fn run_appends_store_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sync.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let base_event = sample_store_decoded_event(Address::from([0x11; 20]));
        let store_event = sample_store_decoded_event(Address::from([0x55; 20]));

        let data_source = TestDataSource {
            latest_block: 200,
            rpc_urls: vec![Url::parse("http://event.rpc").unwrap()],
            fetch_logs: vec![sample_log()],
            store_logs: vec![sample_store_log()],
            decode_responses: Mutex::new(vec![vec![base_event.clone()], vec![store_event.clone()]]),
            sql_result: String::new(),
            fetch_calls: Mutex::new(vec![]),
            fetch_store_calls: Mutex::new(vec![]),
            sql_calls: Mutex::new(vec![]),
            decimals: Mutex::new(vec![]),
            raw_statements: vec![SqlStatement::new(RAW_SQL_STUB)],
            raw_calls: Mutex::new(vec![]),
        };

        let fetcher = TestFetcher {
            metadata: vec![],
            calls: Mutex::new(vec![]),
        };

        let runner = SyncRunner::new(
            &db_path_str,
            &data_source,
            vec![Url::parse("http://metadata.rpc").unwrap()],
            &fetcher,
        );

        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xorder",
            deployment_block: 150,
            start_block: None,
            end_block: Some(160),
        };

        runner.run(&params).await.unwrap();

        let store_calls = data_source.fetch_store_calls.lock().unwrap();
        assert_eq!(store_calls.len(), 1);
        assert_eq!(store_calls[0].1, 150);
        assert_eq!(store_calls[0].2, 160);
        assert_eq!(
            store_calls[0].0,
            vec!["0x1111111111111111111111111111111111111111".to_string()]
        );

        let sql_calls = data_source.sql_calls.lock().unwrap();
        assert_eq!(sql_calls.len(), 1);
        assert_eq!(sql_calls[0], 2);

        let raw_calls = data_source.raw_calls.lock().unwrap();
        assert_eq!(raw_calls.len(), 1);
        assert_eq!(raw_calls[0].len(), 2);
    }

    #[test]
    fn metadata_rpcs_falls_back_to_data_source() {
        let data_source = TestDataSource {
            latest_block: 0,
            rpc_urls: vec![Url::parse("http://event.rpc").unwrap()],
            fetch_logs: vec![],
            store_logs: vec![],
            decode_responses: Mutex::new(vec![vec![]]),
            sql_result: String::new(),
            fetch_calls: Mutex::new(vec![]),
            fetch_store_calls: Mutex::new(vec![]),
            sql_calls: Mutex::new(vec![]),
            decimals: Mutex::new(vec![]),
            raw_statements: Vec::new(),
            raw_calls: Mutex::new(vec![]),
        };
        let fetcher = TestFetcher {
            metadata: vec![],
            calls: Mutex::new(vec![]),
        };

        let runner = SyncRunner::new("/tmp/db", &data_source, vec![], &fetcher);
        assert_eq!(runner.metadata_rpcs(), data_source.rpc_urls());

        let override_runner = SyncRunner::new(
            "/tmp/db",
            &data_source,
            vec![Url::parse("http://override").unwrap()],
            &fetcher,
        );
        assert_eq!(override_runner.metadata_rpcs().len(), 1);
        assert_eq!(
            override_runner.metadata_rpcs()[0].as_str(),
            "http://override/"
        );
    }

    #[tokio::test]
    async fn sync_runner_fetches_store_set_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("stores.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();
        exec.query_text(&SqlStatement::new(
            r#"INSERT INTO interpreter_store_sets (
                store_address,
                transaction_hash,
                log_index,
                block_number,
                block_timestamp,
                namespace,
                key,
                value
            ) VALUES (
                '0x2222222222222222222222222222222222222222',
                '0x1',
                0,
                1,
                0,
                '0x0',
                '0x0',
                '0x0'
            );
"#,
        ))
        .await
        .unwrap();

        let decoded = vec![sample_decoded_event(Address::from([0xaa; 20]))];

        let data_source = TestDataSource {
            latest_block: 190,
            rpc_urls: vec![Url::parse("http://event.rpc").unwrap()],
            fetch_logs: vec![sample_log()],
            store_logs: vec![sample_store_log()],
            decode_responses: Mutex::new(vec![decoded]),
            sql_result: String::new(),
            fetch_calls: Mutex::new(vec![]),
            fetch_store_calls: Mutex::new(vec![]),
            sql_calls: Mutex::new(vec![]),
            decimals: Mutex::new(vec![]),
            raw_statements: vec![SqlStatement::new(RAW_SQL_STUB)],
            raw_calls: Mutex::new(vec![]),
        };
        let fetcher = TestFetcher {
            metadata: vec![],
            calls: Mutex::new(vec![]),
        };
        let runner = SyncRunner::new(&db_path_str, &data_source, vec![], &fetcher);

        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xorder",
            deployment_block: 180,
            start_block: None,
            end_block: Some(185),
        };

        runner.run(&params).await.unwrap();

        let store_calls = data_source.fetch_store_calls.lock().unwrap();
        assert_eq!(store_calls.len(), 1);
        assert_eq!(store_calls[0].1, 180);
        assert_eq!(store_calls[0].2, 185);
        assert!(store_calls[0]
            .0
            .contains(&"0x2222222222222222222222222222222222222222".to_string()));
    }
}
