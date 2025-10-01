use anyhow::{anyhow, Result};
use rain_orderbook_common::raindex_client::local_db::helpers::patch_deposit_amounts_with_decimals;
use serde_json::Value;
use url::Url;

use super::super::data_source::{SyncDataSource, TokenMetadataFetcher};
use super::super::token::{prepare_token_metadata, TokenPrepResult};

pub(super) struct FetchResult {
    pub(super) events: Value,
    pub(super) raw_count: usize,
}

pub(super) struct DecodedEvents {
    pub(super) decoded: Value,
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
    let raw_count = events.as_array().map(|a| a.len()).unwrap_or(0);
    Ok(FetchResult { events, raw_count })
}

pub(super) fn decode_events<D>(data_source: &D, events: Value) -> Result<DecodedEvents>
where
    D: SyncDataSource + Send + Sync,
{
    let decoded = data_source.decode_events(events)?;
    let decoded_count = decoded.as_array().map(|a| a.len()).unwrap_or(0);
    Ok(DecodedEvents {
        decoded,
        decoded_count,
    })
}

pub(super) async fn prepare_sql<D, T>(
    data_source: &D,
    token_fetcher: &T,
    db_path: &str,
    metadata_rpc_urls: &[Url],
    chain_id: u32,
    decoded_events: Value,
    target_block: u64,
) -> Result<String>
where
    D: SyncDataSource + Send + Sync,
    T: TokenMetadataFetcher + Send + Sync,
{
    let metadata_rpc_slice = if metadata_rpc_urls.is_empty() {
        data_source.rpc_urls()
    } else {
        metadata_rpc_urls
    };

    let token_prep = prepare_token_metadata(
        db_path,
        metadata_rpc_slice,
        chain_id,
        &decoded_events,
        token_fetcher,
    )
    .await?;

    let patched_events = patch_events(decoded_events, &token_prep)?;

    data_source.events_to_sql(patched_events, target_block, &token_prep.tokens_prefix_sql)
}

fn patch_events(decoded_events: Value, token_prep: &TokenPrepResult) -> Result<Value> {
    patch_deposit_amounts_with_decimals(decoded_events, &token_prep.decimals_by_addr)
        .map_err(|e| anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use async_trait::async_trait;
    use rain_math_float::Float;
    use rain_orderbook_common::erc20::TokenInfo;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::sqlite::sqlite_execute;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;
    use crate::commands::local_db::sync::token::TokenPrepResult;

    struct MockDataSource {
        latest_block: u64,
        sql_result: String,
        rpc_urls: Vec<Url>,
        captured_prefixes: Mutex<Vec<String>>,
        captured_events: Mutex<Vec<Value>>,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(self.latest_block)
        }

        async fn fetch_events(&self, _: &str, _: u64, _: u64) -> Result<Value> {
            Ok(json!([]))
        }

        fn decode_events(&self, events: Value) -> Result<Value> {
            Ok(events)
        }

        fn events_to_sql(
            &self,
            decoded_events: Value,
            _end_block: u64,
            prefix_sql: &str,
        ) -> Result<String> {
            self.captured_prefixes
                .lock()
                .unwrap()
                .push(prefix_sql.to_string());
            self.captured_events
                .lock()
                .unwrap()
                .push(decoded_events.clone());
            Ok(self.sql_result.clone())
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
    async fn prepare_sql_patches_deposits() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("prep.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();

        let token_addr = Address::from_slice(&[0xaa; 20]);
        let events = json!([
            {
                "event_type": "DepositV2",
                "decoded_data": {
                    "sender": "0x1",
                    "token": format!("0x{:x}", token_addr),
                    "vault_id": "0x0",
                    "deposit_amount_uint256": "0x01"
                }
            }
        ]);

        let token_info = TokenInfo {
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
        };
        let mock_fetcher = MockFetcher {
            metadata: vec![(token_addr, token_info.clone())],
        };

        let data_source = MockDataSource {
            latest_block: 100,
            sql_result: String::from("SQL"),
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            captured_prefixes: Mutex::new(Vec::new()),
            captured_events: Mutex::new(Vec::new()),
        };

        let sql = prepare_sql(
            &data_source,
            &mock_fetcher,
            &db_path_str,
            data_source.rpc_urls(),
            1,
            events,
            50,
        )
        .await
        .unwrap();

        assert_eq!(sql, "SQL");

        let patched = data_source.captured_events.lock().unwrap();
        let amount = &patched[0][0]["decoded_data"]["deposit_amount"];
        let expected = Float::from_fixed_decimal(U256::from(1u64), 18)
            .unwrap()
            .as_hex();
        assert_eq!(amount, &json!(expected));
    }

    #[test]
    fn patch_events_propagates_metadata() {
        let decoded = json!([
            {
                "decoded_data": {
                    "deposit_amount_uint256": "0x01"
                }
            }
        ]);
        let prep = TokenPrepResult {
            tokens_prefix_sql: String::new(),
            decimals_by_addr: HashMap::new(),
        };
        let patched = patch_events(decoded.clone(), &prep).unwrap();
        assert_eq!(patched, decoded);
    }
}
