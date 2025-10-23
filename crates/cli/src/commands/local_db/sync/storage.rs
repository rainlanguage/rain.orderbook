use anyhow::{anyhow, Result};
use rain_orderbook_app_settings::network::NetworkCfg;
use rain_orderbook_common::local_db::{
    query::{
        create_tables::{CREATE_TABLES_SQL, REQUIRED_TABLES},
        fetch_erc20_tokens_by_addresses,
        fetch_last_synced_block::FETCH_LAST_SYNCED_BLOCK_SQL,
        fetch_store_addresses::FETCH_STORE_ADDRESSES_SQL,
    },
    LocalDb,
};
use serde::Deserialize;
use url::Url;

use crate::commands::local_db::executor::RusqliteExecutor;
use rain_orderbook_common::local_db::query::{LocalDbQueryExecutor, SqlStatement};

pub(crate) const DEFAULT_SCHEMA_SQL: &str = CREATE_TABLES_SQL;
pub(crate) const SYNC_STATUS_QUERY: &str = FETCH_LAST_SYNCED_BLOCK_SQL;

pub(crate) const STORE_ADDRESSES_QUERY: &str = FETCH_STORE_ADDRESSES_SQL;

pub(crate) async fn ensure_schema(db_path: &str) -> Result<bool> {
    let exec = RusqliteExecutor::new(db_path);

    const TABLE_QUERY: &str =
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';";
    #[derive(Debug, Deserialize)]
    struct TableNameRow {
        name: String,
    }

    let rows: Vec<TableNameRow> = exec
        .query_json(&SqlStatement::new(TABLE_QUERY))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    let existing: std::collections::HashSet<String> = rows
        .into_iter()
        .map(|row| row.name.to_ascii_lowercase())
        .collect();
    let has_tables = REQUIRED_TABLES
        .iter()
        .all(|t| existing.contains(&t.to_ascii_lowercase()));

    if has_tables {
        return Ok(false);
    }

    exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(true)
}

pub(crate) async fn fetch_last_synced(db_path: &str) -> Result<u64> {
    let exec = RusqliteExecutor::new(db_path);
    let rows: Vec<SyncStatusRow> = exec
        .query_json(&SqlStatement::new(SYNC_STATUS_QUERY))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(rows.first().map(|row| row.last_synced_block).unwrap_or(0))
}

pub(crate) async fn fetch_existing_store_addresses(db_path: &str) -> Result<Vec<String>> {
    let exec = RusqliteExecutor::new(db_path);
    let rows: Vec<StoreAddressRow> = exec
        .query_json(&SqlStatement::new(STORE_ADDRESSES_QUERY))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let trimmed = row.store_address.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_ascii_lowercase())
            }
        })
        .collect())
}

pub(crate) fn build_local_db_from_network(
    chain_id: u32,
    network: &NetworkCfg,
    api_token: &str,
) -> Result<(LocalDb, Vec<Url>)> {
    if network.chain_id != chain_id {
        return Err(anyhow!(
            "Chain ID mismatch: CLI provided {} but network '{}' is configured for {}",
            chain_id,
            network.key,
            network.chain_id
        ));
    }

    if network.rpcs.is_empty() {
        return Err(anyhow!(
            "No RPC URLs configured for network '{}' in settings YAML",
            network.key
        ));
    }

    let metadata_rpcs = network.rpcs.clone();
    let local_db = LocalDb::new_with_hyper_rpc(chain_id, api_token.to_string())?;
    Ok((local_db, metadata_rpcs))
}

pub(crate) async fn fetch_existing_tokens(
    db_path: &str,
    chain_id: u32,
    addresses: &[String],
) -> Result<Vec<Erc20TokenRow>> {
    // Build a parameterized statement. When address list is empty, there is
    // nothing to fetch.
    let Some(stmt) = fetch_erc20_tokens_by_addresses::build_fetch_stmt(chain_id, addresses)
        .map_err(|e| anyhow!(e.to_string()))?
    else {
        return Ok(vec![]);
    };

    let exec = RusqliteExecutor::new(db_path);
    exec.query_json(&stmt)
        .await
        .map_err(|e| anyhow!(e.to_string()))
}

#[derive(Debug, Deserialize)]
pub(crate) struct SyncStatusRow {
    pub(crate) last_synced_block: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Erc20TokenRow {
    pub(crate) address: String,
    pub(crate) decimals: u8,
}

#[derive(Debug, Deserialize)]
struct StoreAddressRow {
    store_address: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::local_db::sync::data_source::SyncDataSource;
    use tempfile::TempDir;

    #[test]
    fn build_local_db_from_network_uses_configured_rpcs() {
        let mut network = NetworkCfg::dummy();
        network.key = "arb-mainnet".to_string();
        network.chain_id = 42161;
        network.rpcs = vec![
            Url::parse("https://arb1.example-rpc.com").unwrap(),
            Url::parse("https://arb2.example-rpc.com").unwrap(),
        ];

        let api_token = "hyper-token";
        let (local_db, metadata_rpcs) =
            build_local_db_from_network(42161, &network, api_token).expect("network rpcs");

        assert_eq!(metadata_rpcs, network.rpcs);

        let event_urls = local_db.rpc_urls();
        assert_eq!(event_urls.len(), 1);
        assert_eq!(event_urls[0].host_str(), Some("arbitrum.rpc.hypersync.xyz"));
        assert!(event_urls[0].as_str().ends_with(&format!("/{api_token}")));
    }

    #[tokio::test]
    async fn ensure_schema_initializes_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("schema.db");
        let db_path_str = db_path.to_string_lossy();

        assert!(ensure_schema(&db_path_str).await.unwrap());
        assert!(!ensure_schema(&db_path_str).await.unwrap());
    }

    #[tokio::test]
    async fn fetch_last_synced_defaults_to_zero() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("status.db");
        let db_path_str = db_path.to_string_lossy();

        {
            let exec = RusqliteExecutor::new(&*db_path_str);
            exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
                .await
                .unwrap();
        }
        let value = fetch_last_synced(&db_path_str).await.unwrap();
        assert_eq!(value, 0);
    }

    #[tokio::test]
    async fn fetch_existing_store_addresses_returns_lowercase() {
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
                '0xABCDEFabcdefABCDEFabcdefABCDEFabcdefABCD',
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

        let stores = fetch_existing_store_addresses(&db_path_str).await.unwrap();
        assert_eq!(stores, vec!["0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"]);
    }
}
