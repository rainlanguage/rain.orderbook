use anyhow::Result;
use url::Url;

use super::super::sqlite::sqlite_execute;
use super::{
    data_source::{SyncDataSource, TokenMetadataFetcher},
    storage::ensure_schema,
};

use self::{
    apply::{decode_events, fetch_events, prepare_sql},
    window::compute_sync_window,
};

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
        let schema_applied = ensure_schema(self.db_path)?;
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
        let fetch = fetch_events(
            self.data_source,
            params.orderbook_address,
            window.start_block,
            window.target_block,
        )
        .await?;
        println!("Fetched {} raw events", fetch.raw_count);

        println!("Decoding events");
        let decoded = decode_events(self.data_source, fetch.events)?;
        println!("Decoded {} events", decoded.decoded_count);

        println!("Preparing token metadata");
        let sql = prepare_sql(
            self.data_source,
            self.token_fetcher,
            self.db_path,
            self.metadata_rpcs(),
            params.chain_id,
            &decoded.decoded,
            window.target_block,
        )
        .await?;

        println!("Generating SQL for {} events", decoded.decoded_count);
        println!("Applying SQL to {}", self.db_path);
        sqlite_execute(self.db_path, &sql)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use async_trait::async_trait;
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;
    use rain_orderbook_common::erc20::TokenInfo;
    use rain_orderbook_common::raindex_client::local_db::decode::{
        DecodedEvent, DecodedEventData, EventType,
    };
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::TempDir;

    use crate::commands::local_db::sqlite::{sqlite_execute, sqlite_query_json};
    use crate::commands::local_db::sync::storage::{
        SyncStatusRow, DEFAULT_SCHEMA_SQL, SYNC_STATUS_QUERY,
    };

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
        decoded_events: Vec<DecodedEventData<DecodedEvent>>,
        sql_result: String,
        fetch_calls: Mutex<Vec<(String, u64, u64)>>,
        sql_calls: Mutex<Vec<(usize, u64)>>,
        prefixes: Mutex<Vec<String>>,
        decimals: Mutex<Vec<HashMap<Address, u8>>>,
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

        fn decode_events(
            &self,
            _events: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
            Ok(self.decoded_events.clone())
        }

        fn events_to_sql(
            &self,
            decoded_events: &[DecodedEventData<DecodedEvent>],
            end_block: u64,
            decimals_by_token: &HashMap<Address, u8>,
            prefix_sql: &str,
        ) -> Result<String> {
            self.sql_calls
                .lock()
                .unwrap()
                .push((decoded_events.len(), end_block));
            self.prefixes.lock().unwrap().push(prefix_sql.to_string());
            self.decimals
                .lock()
                .unwrap()
                .push(decimals_by_token.clone());

            let mut out = String::new();
            if !prefix_sql.is_empty() {
                out.push_str(prefix_sql);
                if !prefix_sql.ends_with('\n') {
                    out.push('\n');
                }
            }
            out.push_str(
                &self
                    .sql_result
                    .replace("?end_block", &end_block.to_string()),
            );
            Ok(out)
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

    #[tokio::test]
    async fn run_executes_full_flow() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sync.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();

        let token_addr = Address::from([0xaa; 20]);
        let decoded_events = vec![sample_decoded_event(token_addr)];

        let data_source = TestDataSource {
            latest_block: 200,
            rpc_urls: vec![Url::parse("http://event.rpc").unwrap()],
            fetch_logs: vec![sample_log()],
            decoded_events: decoded_events.clone(),
            sql_result: "UPDATE sync_status SET last_synced_block = ?end_block".into(),
            fetch_calls: Mutex::new(vec![]),
            sql_calls: Mutex::new(vec![]),
            prefixes: Mutex::new(vec![]),
            decimals: Mutex::new(vec![]),
        };

        let token_info = TokenInfo {
            name: "Token".into(),
            symbol: "TKN".into(),
            decimals: 18,
        };
        let fetcher = TestFetcher {
            metadata: vec![(token_addr, token_info)],
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
            end_block: Some(190),
        };

        runner.run(&params).await.unwrap();

        let fetch_calls = data_source.fetch_calls.lock().unwrap();
        assert_eq!(fetch_calls.len(), 1);
        assert_eq!(fetch_calls[0], ("0xorder".into(), 150, 190));

        let sql_calls = data_source.sql_calls.lock().unwrap();
        assert_eq!(sql_calls.len(), 1);
        assert_eq!(sql_calls[0], (decoded_events.len(), 190));

        let prefixes = data_source.prefixes.lock().unwrap();
        assert_eq!(prefixes.len(), 1);

        let decimals = data_source.decimals.lock().unwrap();
        assert_eq!(decimals.len(), 1);
        assert_eq!(decimals[0].get(&token_addr), Some(&18));

        let fetcher_calls = fetcher.calls.lock().unwrap();
        assert_eq!(fetcher_calls.len(), 1);
        assert_eq!(fetcher_calls[0], vec![token_addr]);

        let sync_rows: Vec<SyncStatusRow> =
            sqlite_query_json(&db_path_str, SYNC_STATUS_QUERY).unwrap();
        assert_eq!(sync_rows[0].last_synced_block, 190);
    }

    #[test]
    fn metadata_rpcs_falls_back_to_data_source() {
        let data_source = TestDataSource {
            latest_block: 0,
            rpc_urls: vec![Url::parse("http://event.rpc").unwrap()],
            fetch_logs: vec![],
            decoded_events: vec![],
            sql_result: String::new(),
            fetch_calls: Mutex::new(vec![]),
            sql_calls: Mutex::new(vec![]),
            prefixes: Mutex::new(vec![]),
            decimals: Mutex::new(vec![]),
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
}
