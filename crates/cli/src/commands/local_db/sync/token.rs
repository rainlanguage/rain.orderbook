use alloy::primitives::Address;
use anyhow::Result;
use rain_orderbook_common::raindex_client::local_db::insert::generate_erc20_tokens_sql;
use rain_orderbook_common::raindex_client::local_db::tokens::collect_token_addresses;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use url::Url;

use super::{data_source::TokenMetadataFetcher, storage::fetch_existing_tokens};

pub(crate) struct TokenPrepResult {
    pub(crate) tokens_prefix_sql: String,
    pub(crate) decimals_by_addr: HashMap<String, u8>,
}

pub(crate) async fn prepare_token_metadata<T>(
    db_path: &str,
    rpc_urls: &[Url],
    chain_id: u32,
    decoded_events: &Value,
    token_fetcher: &T,
) -> Result<TokenPrepResult>
where
    T: TokenMetadataFetcher + Send + Sync,
{
    let address_set = collect_token_addresses(decoded_events);
    let mut all_token_addrs: Vec<Address> = address_set.into_iter().collect();
    all_token_addrs.sort();

    if all_token_addrs.is_empty() {
        return Ok(TokenPrepResult {
            tokens_prefix_sql: String::new(),
            decimals_by_addr: HashMap::new(),
        });
    }

    let addr_strings: Vec<String> = all_token_addrs
        .iter()
        .map(|a| format!("0x{:x}", a))
        .collect();
    let existing_rows = fetch_existing_tokens(db_path, chain_id, &addr_strings)?;

    let mut decimals_by_addr: HashMap<String, u8> = HashMap::new();
    let mut existing_lower: HashSet<String> = HashSet::new();
    for row in existing_rows.iter() {
        let key = row.address.to_ascii_lowercase();
        existing_lower.insert(key.clone());
        decimals_by_addr.insert(key, row.decimals);
    }

    let mut missing_addrs: Vec<Address> = Vec::new();
    for addr in all_token_addrs.iter() {
        let key = format!("0x{:x}", addr).to_ascii_lowercase();
        if !existing_lower.contains(&key) {
            missing_addrs.push(*addr);
        }
    }

    if missing_addrs.is_empty() {
        return Ok(TokenPrepResult {
            tokens_prefix_sql: String::new(),
            decimals_by_addr,
        });
    }

    println!("Fetching metadata for {} new token(s)", missing_addrs.len());
    let fetched = token_fetcher.fetch(rpc_urls, missing_addrs).await?;

    let tokens_prefix_sql = generate_erc20_tokens_sql(chain_id, &fetched);
    for (addr, info) in fetched.into_iter() {
        let key = format!("0x{:x}", addr);
        decimals_by_addr.insert(key, info.decimals);
    }

    Ok(TokenPrepResult {
        tokens_prefix_sql,
        decimals_by_addr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use rain_orderbook_common::erc20::TokenInfo;
    use serde_json::json;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::sqlite::sqlite_execute;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;

    struct NoopFetcher;

    #[async_trait]
    impl TokenMetadataFetcher for NoopFetcher {
        async fn fetch(&self, _: &[Url], _: Vec<Address>) -> Result<Vec<(Address, TokenInfo)>> {
            panic!("fetch should not be called")
        }
    }

    #[tokio::test]
    async fn prepare_tokens_uses_existing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tokens.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        sqlite_execute(
            &db_path_str,
            "INSERT INTO erc20_tokens (chain_id, address, name, symbol, decimals) VALUES (1, '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'A', 'A', 18);",
        )
        .unwrap();

        let decoded = json!([
            {
                "event_type": "DepositV2",
                "decoded_data": {
                    "sender": "0x1",
                    "token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "vault_id": "0x0",
                    "deposit_amount_uint256": "0x01"
                }
            }
        ]);

        let rpc_urls = vec![Url::parse("http://localhost:1").unwrap()];
        let prep = prepare_token_metadata(&db_path_str, &rpc_urls, 1, &decoded, &NoopFetcher)
            .await
            .unwrap();
        assert!(prep.tokens_prefix_sql.is_empty());
        assert_eq!(prep.decimals_by_addr.len(), 1);
        assert_eq!(
            prep.decimals_by_addr
                .get("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .copied(),
            Some(18)
        );
    }
}
