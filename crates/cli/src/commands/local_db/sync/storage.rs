use anyhow::{anyhow, Result};
use rain_orderbook_app_settings::network::NetworkCfg;
use rain_orderbook_common::raindex_client::local_db::{
    query::{
        create_tables::{CREATE_TABLES_SQL, REQUIRED_TABLES},
        fetch_erc20_tokens_by_addresses::FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL,
        fetch_last_synced_block::FETCH_LAST_SYNCED_BLOCK_SQL,
        fetch_store_addresses::FETCH_STORE_ADDRESSES_SQL,
    },
    LocalDb,
};
use serde::Deserialize;
use url::Url;

use super::super::sqlite::{sqlite_execute, sqlite_has_required_tables, sqlite_query_json};

pub(crate) const DEFAULT_SCHEMA_SQL: &str = CREATE_TABLES_SQL;
pub(crate) const SYNC_STATUS_QUERY: &str = FETCH_LAST_SYNCED_BLOCK_SQL;
pub(crate) const ERC20_QUERY_TEMPLATE: &str = FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL;
pub(crate) const STORE_ADDRESSES_QUERY: &str = FETCH_STORE_ADDRESSES_SQL;

pub(crate) fn ensure_schema(db_path: &str) -> Result<bool> {
    if sqlite_has_required_tables(db_path, REQUIRED_TABLES)? {
        return Ok(false);
    }

    sqlite_execute(db_path, DEFAULT_SCHEMA_SQL)?;
    Ok(true)
}

pub(crate) fn fetch_last_synced(db_path: &str) -> Result<u64> {
    let rows: Vec<SyncStatusRow> = sqlite_query_json(db_path, SYNC_STATUS_QUERY)?;
    Ok(rows.first().map(|row| row.last_synced_block).unwrap_or(0))
}

pub(crate) fn fetch_existing_store_addresses(db_path: &str) -> Result<Vec<String>> {
    let rows: Vec<StoreAddressRow> = sqlite_query_json(db_path, STORE_ADDRESSES_QUERY)?;
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

pub(crate) fn fetch_existing_tokens(
    db_path: &str,
    chain_id: u32,
    addresses: &[String],
) -> Result<Vec<Erc20TokenRow>> {
    if addresses.is_empty() {
        return Ok(vec![]);
    }

    let in_clause = addresses
        .iter()
        .map(|addr| format!("'{}'", addr.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = ERC20_QUERY_TEMPLATE
        .replace("?chain_id", &chain_id.to_string())
        .replace("?addresses_in", &in_clause);

    sqlite_query_json(db_path, &sql)
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

    #[test]
    fn ensure_schema_initializes_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("schema.db");
        let db_path_str = db_path.to_string_lossy();

        assert!(ensure_schema(&db_path_str).unwrap());
        assert!(!ensure_schema(&db_path_str).unwrap());
    }

    #[test]
    fn fetch_last_synced_defaults_to_zero() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("status.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        let value = fetch_last_synced(&db_path_str).unwrap();
        assert_eq!(value, 0);
    }

    #[test]
    fn fetch_existing_store_addresses_returns_lowercase() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("stores.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        sqlite_execute(
            &db_path_str,
            "INSERT INTO interpreter_store_sets (store_address, transaction_hash, log_index, block_number, block_timestamp, namespace, key, value) VALUES ('0xABCDEFabcdefABCDEFabcdefABCDEFabcdefABCD', '0x1', 0, 1, 0, '0x0', '0x0', '0x0');",
        )
        .unwrap();

        let stores = fetch_existing_store_addresses(&db_path_str).unwrap();
        assert_eq!(stores, vec!["0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"]);
    }
}
