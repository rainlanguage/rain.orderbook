use anyhow::Result;
use std::mem;
use url::Url;

use super::super::sqlite::sqlite_execute;
use super::{
    data_source::{SyncDataSource, TokenMetadataFetcher},
    storage::ensure_schema,
    store::{collect_all_store_addresses, fetch_and_merge_store_events},
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
        let mut decoded = decode_events(self.data_source, fetch.events)?;
        println!("Decoded {} events", decoded.decoded_count);

        println!("Collecting interpreter store addresses");
        let store_addresses = collect_all_store_addresses(self.db_path, &decoded.decoded)?;
        if !store_addresses.is_empty() {
            println!(
                "Fetching interpreter store Set events for {} store(s)",
                store_addresses.len()
            );
            let decoded_value = mem::take(&mut decoded.decoded);
            let merge = fetch_and_merge_store_events(
                self.data_source,
                decoded_value,
                &store_addresses,
                window.start_block,
                window.target_block,
            )
            .await?;
            println!(
                "Fetched {} interpreter store Set events",
                merge.stats.fetched_raw_count
            );
            if merge.stats.decoded_count > 0 {
                println!(
                    "Decoded {} interpreter store events",
                    merge.stats.decoded_count
                );
            }
            decoded.decoded = merge.events;
            decoded.decoded_count = merge.stats.total_decoded_count;
        }

        println!("Preparing token metadata");
        let sql = prepare_sql(
            self.data_source,
            self.token_fetcher,
            self.db_path,
            self.metadata_rpcs(),
            params.chain_id,
            decoded.decoded,
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
    use rain_math_float::Float;
    use rain_orderbook_common::erc20::TokenInfo;
    use serde_json::{json, Value};
    use std::sync::Mutex;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::sqlite::{sqlite_execute, sqlite_query_json};
    use crate::commands::local_db::sync::storage::{
        Erc20TokenRow, SyncStatusRow, DEFAULT_SCHEMA_SQL, SYNC_STATUS_QUERY,
    };

    struct MockDataSource {
        latest_block: u64,
        events: Value,
        decoded: Value,
        store_events: Value,
        decoded_store: Value,
        sql_result: String,
        rpc_urls: Vec<Url>,
        fetch_calls: Mutex<Vec<(String, u64, u64)>>,
        store_fetch_calls: Mutex<Vec<(Vec<String>, u64, u64)>>,
        prefixes: Mutex<Vec<String>>,
        patched_events: Mutex<Vec<Value>>,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(self.latest_block)
        }

        async fn fetch_events(
            &self,
            orderbook_address: &str,
            start_block: u64,
            end_block: u64,
        ) -> Result<Value> {
            self.fetch_calls.lock().unwrap().push((
                orderbook_address.to_string(),
                start_block,
                end_block,
            ));
            Ok(self.events.clone())
        }

        async fn fetch_store_set_events(
            &self,
            store_addresses: &[String],
            start_block: u64,
            end_block: u64,
        ) -> Result<Value> {
            self.store_fetch_calls.lock().unwrap().push((
                store_addresses.to_vec(),
                start_block,
                end_block,
            ));
            Ok(self.store_events.clone())
        }

        fn decode_events(&self, events: Value) -> Result<Value> {
            if events == self.events {
                Ok(self.decoded.clone())
            } else if events == self.store_events {
                Ok(self.decoded_store.clone())
            } else {
                Ok(events)
            }
        }

        fn events_to_sql(
            &self,
            decoded_events: Value,
            end_block: u64,
            prefix_sql: &str,
        ) -> Result<String> {
            self.prefixes.lock().unwrap().push(prefix_sql.to_string());
            self.patched_events
                .lock()
                .unwrap()
                .push(decoded_events.clone());

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

    struct MockTokenFetcher {
        metadata: Vec<(Address, TokenInfo)>,
        calls: Mutex<Vec<Vec<Address>>>,
    }

    #[async_trait]
    impl TokenMetadataFetcher for MockTokenFetcher {
        async fn fetch(
            &self,
            _: &[Url],
            missing: Vec<Address>,
        ) -> Result<Vec<(Address, TokenInfo)>> {
            self.calls.lock().unwrap().push(missing.clone());
            Ok(self.metadata.clone())
        }
    }

    struct PanicFetcher;

    #[async_trait]
    impl TokenMetadataFetcher for PanicFetcher {
        async fn fetch(&self, _: &[Url], _: Vec<Address>) -> Result<Vec<(Address, TokenInfo)>> {
            panic!("metadata fetch should not be called")
        }
    }

    #[tokio::test]
    async fn sync_runner_uses_cached_tokens_without_fetch() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("cached.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        sqlite_execute(
            &db_path_str,
            "INSERT INTO erc20_tokens (chain_id, address, name, symbol, decimals) VALUES (1, '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'A', 'A', 18);",
        )
        .unwrap();

        let events = json!([{"blockNumber": "0x1", "data": "0x", "topics": []}]);
        let decoded = json!([
            {
                "event_type": "DepositV2",
                "block_number": "0x1",
                "block_timestamp": "0x0",
                "transaction_hash": "0x01",
                "log_index": "0x0",
                "decoded_data": {
                    "sender": "0x1",
                    "token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "vault_id": "0x0",
                    "deposit_amount_uint256": "0x01"
                }
            }
        ]);

        let data_source = MockDataSource {
            latest_block: 150,
            events,
            decoded,
            store_events: Value::Array(vec![]),
            decoded_store: Value::Array(vec![]),
            sql_result: "BEGIN TRANSACTION;\nUPDATE sync_status SET last_synced_block = ?end_block, updated_at = CURRENT_TIMESTAMP WHERE id = 1;\nCOMMIT;\n"
            .to_string(),
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            fetch_calls: Mutex::new(Vec::new()),
            store_fetch_calls: Mutex::new(Vec::new()),
            prefixes: Mutex::new(Vec::new()),
            patched_events: Mutex::new(Vec::new()),
        };

        let runner = SyncRunner::new(
            &db_path_str,
            &data_source,
            data_source.rpc_urls.clone(),
            &PanicFetcher,
        );
        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xfeed",
            deployment_block: 120,
            start_block: None,
            end_block: None,
        };

        runner.run(&params).await.unwrap();

        let fetch_calls = data_source.fetch_calls.lock().unwrap();
        assert_eq!(fetch_calls.len(), 1);
        assert_eq!(fetch_calls[0], ("0xfeed".to_string(), 120, 150));

        assert!(data_source.store_fetch_calls.lock().unwrap().is_empty());

        // Token prefix SQL should be empty and deposit patched using cached decimals
        assert!(data_source.prefixes.lock().unwrap()[0].is_empty());
        let patched = data_source.patched_events.lock().unwrap()[0].clone();
        let amount = &patched[0]["decoded_data"]["deposit_amount"];
        let expected = Float::from_fixed_decimal(U256::from(1u64), 18)
            .unwrap()
            .as_hex();
        assert_eq!(amount, &json!(expected));

        let token_calls = sqlite_query_json::<Vec<Erc20TokenRow>>(
            &db_path_str,
            "SELECT address, decimals FROM erc20_tokens;",
        )
        .unwrap();
        assert_eq!(token_calls.len(), 1);
        assert_eq!(
            token_calls[0].address,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );

        let sync_rows: Vec<SyncStatusRow> =
            sqlite_query_json(&db_path_str, SYNC_STATUS_QUERY).unwrap();
        assert_eq!(sync_rows[0].last_synced_block, 150);
    }

    #[tokio::test]
    async fn sync_runner_executes_with_mocks() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sync.db");
        let db_path_str = db_path.to_string_lossy();

        let token_addr = Address::from_slice(&[0xaa; 20]);
        let events = json!([{"blockNumber": "0x1", "data": "0x", "topics": []}]);
        let decoded = json!([
            {
                "event_type": "DepositV2",
                "block_number": "0x1",
                "block_timestamp": "0x0",
                "transaction_hash": "0x01",
                "log_index": "0x0",
                "decoded_data": {
                    "sender": "0x1",
                    "token": format!("0x{:x}", token_addr),
                    "vault_id": "0x0",
                    "deposit_amount_uint256": "0x01"
                }
            }
        ]);

        let expected_sql = "BEGIN TRANSACTION;\nUPDATE sync_status SET last_synced_block = ?end_block, updated_at = CURRENT_TIMESTAMP WHERE id = 1;\nCOMMIT;\n"
        .to_string();

        let data_source = MockDataSource {
            latest_block: 120,
            events,
            decoded,
            store_events: Value::Array(vec![]),
            decoded_store: Value::Array(vec![]),
            sql_result: expected_sql,
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            fetch_calls: Mutex::new(Vec::new()),
            store_fetch_calls: Mutex::new(Vec::new()),
            prefixes: Mutex::new(Vec::new()),
            patched_events: Mutex::new(Vec::new()),
        };

        let token_info = TokenInfo {
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
        };
        let token_fetcher = MockTokenFetcher {
            metadata: vec![(token_addr, token_info.clone())],
            calls: Mutex::new(Vec::new()),
        };

        let runner = SyncRunner::new(
            &db_path_str,
            &data_source,
            data_source.rpc_urls.clone(),
            &token_fetcher,
        );
        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xfeed",
            deployment_block: 100,
            start_block: None,
            end_block: None,
        };

        runner.run(&params).await.unwrap();

        let fetch_calls = data_source.fetch_calls.lock().unwrap();
        assert_eq!(fetch_calls.len(), 1);
        assert_eq!(fetch_calls[0], ("0xfeed".to_string(), 100, 120));

        assert!(data_source.store_fetch_calls.lock().unwrap().is_empty());

        let prefixes = data_source.prefixes.lock().unwrap();
        assert_eq!(prefixes.len(), 1);
        assert!(prefixes[0].contains("INSERT INTO erc20_tokens"));

        let patched_events = data_source.patched_events.lock().unwrap();
        let patched = patched_events[0].clone();
        let depo = &patched[0]["decoded_data"]["deposit_amount"];
        let expected = Float::from_fixed_decimal(U256::from(1u64), 18)
            .unwrap()
            .as_hex();
        assert_eq!(depo, &json!(expected));

        let token_calls = token_fetcher.calls.lock().unwrap();
        assert_eq!(token_calls.len(), 1);
        assert_eq!(token_calls[0], vec![token_addr]);

        let tokens_in_db: Vec<Erc20TokenRow> =
            sqlite_query_json(&db_path_str, "SELECT address, decimals FROM erc20_tokens;").unwrap();
        assert_eq!(tokens_in_db.len(), 1);
        assert_eq!(tokens_in_db[0].address, format!("0x{:x}", token_addr));

        let sync_rows: Vec<SyncStatusRow> =
            sqlite_query_json(&db_path_str, SYNC_STATUS_QUERY).unwrap();
        assert_eq!(sync_rows[0].last_synced_block, 120);
    }

    #[tokio::test]
    async fn sync_runner_fetches_store_set_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("stores.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        sqlite_execute(
            &db_path_str,
            "INSERT INTO interpreter_store_sets (store_address, transaction_hash, log_index, block_number, block_timestamp, namespace, key, value) VALUES ('0x2222222222222222222222222222222222222222', '0x1', 0, 1, 0, '0x0', '0x0', '0x0');",
        )
        .unwrap();

        let events = json!([{
            "blockNumber": "0x1",
            "blockTimestamp": "0x0",
            "transactionHash": "0xabc",
            "logIndex": "0x0",
            "topics": ["0xaddorder"],
            "data": "0x",
            "address": "0xorderbook"
        }]);
        let decoded = json!([{
            "event_type": "AddOrderV3",
            "decoded_data": {
                "order": {
                    "evaluable": {"store": "0x1111111111111111111111111111111111111111"},
                    "valid_inputs": [],
                    "valid_outputs": []
                }
            }
        }]);

        let store_events = json!([{
            "blockNumber": "0x2",
            "blockTimestamp": "0x0",
            "transactionHash": "0xdef",
            "logIndex": "0x0",
            "topics": ["0xset"],
            "data": "0x",
            "address": "0x2222222222222222222222222222222222222222"
        }]);
        let decoded_store = json!([{
            "event_type": "Set",
            "decoded_data": {
                "namespace": "0x01",
                "key": "0x02",
                "value": "0x03"
            }
        }]);

        let expected_sql = "BEGIN TRANSACTION;\nUPDATE sync_status SET last_synced_block = ?end_block, updated_at = CURRENT_TIMESTAMP WHERE id = 1;\nCOMMIT;\n"
            .to_string();

        let data_source = MockDataSource {
            latest_block: 5,
            events,
            decoded,
            store_events,
            decoded_store,
            sql_result: expected_sql,
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            fetch_calls: Mutex::new(Vec::new()),
            store_fetch_calls: Mutex::new(Vec::new()),
            prefixes: Mutex::new(Vec::new()),
            patched_events: Mutex::new(Vec::new()),
        };

        let token_fetcher = MockTokenFetcher {
            metadata: Vec::new(),
            calls: Mutex::new(Vec::new()),
        };

        let runner = SyncRunner::new(
            &db_path_str,
            &data_source,
            data_source.rpc_urls.clone(),
            &token_fetcher,
        );
        let params = SyncParams {
            chain_id: 1,
            orderbook_address: "0xfeed",
            deployment_block: 1,
            start_block: None,
            end_block: Some(5),
        };

        runner.run(&params).await.unwrap();

        let store_calls = data_source.store_fetch_calls.lock().unwrap();
        assert_eq!(store_calls.len(), 1);
        let (stores, start, end) = &store_calls[0];
        assert_eq!(*start, 1);
        assert_eq!(*end, 5);
        assert_eq!(stores.len(), 2);
        assert!(stores
            .iter()
            .any(|s| s == "0x1111111111111111111111111111111111111111"));
        assert!(stores
            .iter()
            .any(|s| s == "0x2222222222222222222222222222222222222222"));

        let patched_events = data_source.patched_events.lock().unwrap();
        let patched = patched_events[0].as_array().unwrap();
        assert_eq!(patched.len(), 2);
        assert_eq!(patched[1]["event_type"], json!("Set"));
    }
}
