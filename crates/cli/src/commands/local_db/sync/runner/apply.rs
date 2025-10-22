use anyhow::Result;
use rain_orderbook_common::local_db::decode::{DecodedEvent, DecodedEventData};
use rain_orderbook_common::local_db::query::{
    update_last_synced_block::build_update_last_synced_block_stmt, SqlStatementBatch,
};
use rain_orderbook_common::rpc_client::LogEntryResponse;
use url::Url;

use super::super::data_source::{SyncDataSource, TokenMetadataFetcher};
use super::super::token::prepare_token_metadata;

pub(super) struct FetchResult {
    pub(super) events: Vec<LogEntryResponse>,
    pub(super) raw_count: usize,
}

pub(super) struct DecodedEvents {
    pub(super) decoded: Vec<DecodedEventData<DecodedEvent>>,
    pub(super) decoded_count: usize,
}

pub(super) async fn fetch_events<D>(
    data_source: &D,
    orderbook_address: &str,
    start_block: u64,
    target_block: u64,
) -> Result<FetchResult>
where
    D: SyncDataSource + Send + Sync,
{
    let events = data_source
        .fetch_events(orderbook_address, start_block, target_block)
        .await?;
    let raw_count = events.len();
    Ok(FetchResult { events, raw_count })
}

pub(super) fn decode_events<D>(
    data_source: &D,
    events: Vec<LogEntryResponse>,
) -> Result<DecodedEvents>
where
    D: SyncDataSource + Send + Sync,
{
    let decoded = data_source.decode_events(&events)?;
    let decoded_count = decoded.len();
    Ok(DecodedEvents {
        decoded,
        decoded_count,
    })
}

pub(super) struct PrepareSqlParams {
    pub(super) db_path: String,
    pub(super) metadata_rpc_urls: Vec<Url>,
    pub(super) chain_id: u32,
    pub(super) decoded_events: Vec<DecodedEventData<DecodedEvent>>,
    pub(super) raw_events: Vec<LogEntryResponse>,
    pub(super) target_block: u64,
}

pub(super) async fn prepare_sql<D, T>(
    data_source: &D,
    token_fetcher: &T,
    params: PrepareSqlParams,
) -> Result<SqlStatementBatch>
where
    D: SyncDataSource + Send + Sync,
    T: TokenMetadataFetcher + Send + Sync,
{
    let PrepareSqlParams {
        db_path,
        metadata_rpc_urls,
        chain_id,
        decoded_events,
        raw_events,
        target_block,
    } = params;

    let metadata_rpc_slice = if metadata_rpc_urls.is_empty() {
        data_source.rpc_urls()
    } else {
        &metadata_rpc_urls
    };

    let mut batch = data_source.raw_events_to_statements(&raw_events)?;

    let token_prep = prepare_token_metadata(
        &db_path,
        metadata_rpc_slice,
        chain_id,
        &decoded_events,
        token_fetcher,
    )
    .await?;

    let events_batch = data_source.events_to_sql(
        &decoded_events,
        &token_prep.decimals_by_addr,
        &token_prep.tokens_prefix_sql,
    )?;
    batch.extend(events_batch);
    batch.add(build_update_last_synced_block_stmt(target_block));

    batch
        .into_transaction()
        .map_err(|e| anyhow::anyhow!("Failed to wrap SQL in transaction: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use async_trait::async_trait;
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;
    use rain_orderbook_common::erc20::TokenInfo;
    use rain_orderbook_common::local_db::decode::{DecodedEvent, DecodedEventData, EventType};
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::executor::RusqliteExecutor;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;
    use rain_orderbook_common::local_db::query::{
        LocalDbQueryExecutor, SqlStatement, SqlStatementBatch, SqlValue,
    };

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

    fn batch_to_string(batch: &SqlStatementBatch) -> String {
        // TODO: LEGACY SUPPORT flatten statements into SQL until all call sites consume batches directly.
        let mut out = String::new();
        for stmt in batch.statements() {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            let sql = stmt.sql();
            out.push_str(sql);
            if !sql.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }

    struct MockDataSource {
        sql_result: String,
        rpc_urls: Vec<Url>,
        captured_prefixes: Mutex<Vec<String>>,
        captured_events: Mutex<Vec<Vec<DecodedEventData<DecodedEvent>>>>,
        captured_decimals: Mutex<Vec<HashMap<Address, u8>>>,
        raw_statements: Vec<SqlStatement>,
        captured_raw: Mutex<Vec<Vec<LogEntryResponse>>>,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(0)
        }

        async fn fetch_events(
            &self,
            _orderbook_address: &str,
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            Ok(vec![])
        }

        async fn fetch_store_set_events(
            &self,
            _store_addresses: &[String],
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            Ok(vec![])
        }

        fn decode_events(
            &self,
            _events: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
            Ok(vec![])
        }

        fn events_to_sql(
            &self,
            decoded_events: &[DecodedEventData<DecodedEvent>],
            decimals_by_token: &HashMap<Address, u8>,
            prefix_sql: &str,
        ) -> Result<SqlStatementBatch> {
            self.captured_prefixes
                .lock()
                .unwrap()
                .push(prefix_sql.to_string());
            self.captured_events
                .lock()
                .unwrap()
                .push(decoded_events.to_vec());
            self.captured_decimals
                .lock()
                .unwrap()
                .push(decimals_by_token.clone());

            let mut statements = Vec::new();
            if !prefix_sql.is_empty() {
                statements.push(SqlStatement::new(prefix_sql.to_string()));
            }
            if !self.sql_result.is_empty() {
                statements.push(SqlStatement::new(self.sql_result.clone()));
            }
            Ok(SqlStatementBatch::from(statements))
        }

        fn raw_events_to_statements(
            &self,
            raw_events: &[LogEntryResponse],
        ) -> Result<SqlStatementBatch> {
            self.captured_raw.lock().unwrap().push(raw_events.to_vec());
            Ok(SqlStatementBatch::from(self.raw_statements.clone()))
        }

        fn rpc_urls(&self) -> &[Url] {
            &self.rpc_urls
        }
    }

    struct MockFetcher {
        metadata: Vec<(Address, TokenInfo)>,
    }

    #[async_trait]
    impl TokenMetadataFetcher for MockFetcher {
        async fn fetch(&self, _: &[Url], _: Vec<Address>) -> Result<Vec<(Address, TokenInfo)>> {
            Ok(self.metadata.clone())
        }
    }

    #[tokio::test]
    async fn fetch_events_counts_results() {
        let data_source = MockDataSource {
            sql_result: String::new(),
            rpc_urls: vec![],
            captured_prefixes: Mutex::new(vec![]),
            captured_events: Mutex::new(vec![]),
            captured_decimals: Mutex::new(vec![]),
            raw_statements: Vec::new(),
            captured_raw: Mutex::new(vec![]),
        };

        let result = fetch_events(&data_source, "0xorder", 1, 10)
            .await
            .expect("fetch events");
        assert_eq!(result.events.len(), 0);
        assert_eq!(result.raw_count, 0);
    }

    #[tokio::test]
    async fn decode_events_counts_results() {
        let data_source = MockDataSource {
            sql_result: String::new(),
            rpc_urls: vec![],
            captured_prefixes: Mutex::new(vec![]),
            captured_events: Mutex::new(vec![]),
            captured_decimals: Mutex::new(vec![]),
            raw_statements: Vec::new(),
            captured_raw: Mutex::new(vec![]),
        };

        let decoded = decode_events(&data_source, vec![]).expect("decode events");
        assert_eq!(decoded.decoded.len(), 0);
        assert_eq!(decoded.decoded_count, 0);
    }

    #[tokio::test]
    async fn prepare_sql_generates_sql_with_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sync.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let data_source = MockDataSource {
            sql_result: "INSERT INTO sync(last_synced_block) VALUES(?end_block)".to_string(),
            rpc_urls: vec![Url::parse("http://example.com").unwrap()],
            captured_prefixes: Mutex::new(vec![]),
            captured_events: Mutex::new(vec![]),
            captured_decimals: Mutex::new(vec![]),
            raw_statements: vec![SqlStatement::new(RAW_SQL_STUB)],
            captured_raw: Mutex::new(vec![]),
        };

        let token_fetcher = MockFetcher { metadata: vec![] };
        let result = prepare_sql(
            &data_source,
            &token_fetcher,
            PrepareSqlParams {
                db_path: db_path_str.to_string(),
                metadata_rpc_urls: vec![],
                chain_id: 1,
                decoded_events: vec![],
                raw_events: vec![],
                target_block: 100,
            },
        )
        .await
        .expect("prepare sql");

        let sql_text = batch_to_string(&result);
        assert!(sql_text.contains("INSERT INTO sync"));
        let prefixes = data_source.captured_prefixes.lock().unwrap();
        assert_eq!(prefixes.len(), 1);
        assert!(prefixes[0].is_empty());
        let raw = data_source.captured_raw.lock().unwrap();
        assert_eq!(raw.len(), 1);
        assert!(raw[0].is_empty());
    }

    #[tokio::test]
    async fn prepare_sql_passes_token_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("prep.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let token_addr = Address::from([0xaa; 20]);
        let decoded = vec![DecodedEventData {
            event_type: EventType::DepositV2,
            block_number: "0x0".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0x0".into(),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                sender: Address::from([0x11; 20]),
                token: token_addr,
                vaultId: U256::from(0).into(),
                depositAmountUint256: U256::from(1),
            })),
        }];

        let token_info = TokenInfo {
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
        };
        let mock_fetcher = MockFetcher {
            metadata: vec![(token_addr, token_info)],
        };

        let data_source = MockDataSource {
            sql_result: String::new(),
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            captured_prefixes: Mutex::new(Vec::new()),
            captured_events: Mutex::new(Vec::new()),
            captured_decimals: Mutex::new(Vec::new()),
            raw_statements: vec![SqlStatement::new(RAW_SQL_STUB)],
            captured_raw: Mutex::new(Vec::new()),
        };

        let raw_events = vec![LogEntryResponse {
            address: "0x1".into(),
            topics: vec!["0x0".into()],
            data: "0x".into(),
            block_number: "0x0".into(),
            block_timestamp: Some("0x0".into()),
            transaction_hash: "0x0".into(),
            transaction_index: "0x0".into(),
            block_hash: "0x0".into(),
            log_index: "0x0".into(),
            removed: false,
        }];

        let batch = prepare_sql(
            &data_source,
            &mock_fetcher,
            PrepareSqlParams {
                db_path: db_path_str.to_string(),
                metadata_rpc_urls: data_source.rpc_urls().to_vec(),
                chain_id: 1,
                decoded_events: decoded,
                raw_events,
                target_block: 42,
            },
        )
        .await
        .unwrap();

        let update_stmt = batch
            .statements()
            .iter()
            .find(|stmt| {
                stmt.sql()
                    .contains("UPDATE sync_status SET last_synced_block")
            })
            .expect("update statement");
        assert!(matches!(
            update_stmt.params().first(),
            Some(SqlValue::I64(value)) if *value == 42
        ));

        let prefixes = data_source.captured_prefixes.lock().unwrap();
        assert_eq!(prefixes.len(), 1);
        assert!(prefixes[0].starts_with("INSERT INTO erc20_tokens"));

        let captured_events = data_source.captured_events.lock().unwrap();
        assert_eq!(captured_events.len(), 1);
        assert_eq!(captured_events[0].len(), 1);
        match &captured_events[0][0].decoded_data {
            DecodedEvent::DepositV2(deposit) => {
                assert_eq!(deposit.token, token_addr);
            }
            other => panic!("unexpected event type: {other:?}"),
        }

        let captured_decimals = data_source.captured_decimals.lock().unwrap();
        assert_eq!(captured_decimals.len(), 1);
        assert_eq!(captured_decimals[0].get(&token_addr), Some(&18));

        let captured_raw = data_source.captured_raw.lock().unwrap();
        assert_eq!(captured_raw.len(), 1);
        assert_eq!(captured_raw[0].len(), 1);
    }

    #[tokio::test]
    async fn prepare_sql_handles_empty_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("prep.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();

        let data_source = MockDataSource {
            sql_result: String::new(),
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            captured_prefixes: Mutex::new(Vec::new()),
            captured_events: Mutex::new(Vec::new()),
            captured_decimals: Mutex::new(Vec::new()),
            raw_statements: Vec::new(),
            captured_raw: Mutex::new(Vec::new()),
        };
        let mock_fetcher = MockFetcher { metadata: vec![] };

        let batch = prepare_sql(
            &data_source,
            &mock_fetcher,
            PrepareSqlParams {
                db_path: db_path_str.to_string(),
                metadata_rpc_urls: data_source.rpc_urls().to_vec(),
                chain_id: 1,
                decoded_events: vec![],
                raw_events: vec![],
                target_block: 75,
            },
        )
        .await
        .unwrap();

        let update_stmt = batch
            .statements()
            .iter()
            .find(|stmt| {
                stmt.sql()
                    .contains("UPDATE sync_status SET last_synced_block")
            })
            .expect("update statement");
        assert!(matches!(
            update_stmt.params().first(),
            Some(SqlValue::I64(value)) if *value == 75
        ));
        let prefixes = data_source.captured_prefixes.lock().unwrap();
        assert_eq!(prefixes.len(), 1);
        assert!(prefixes[0].is_empty());
    }
}
