use alloy::primitives::Address;
use anyhow::{Context, Result};
use itertools::Itertools;
use rain_orderbook_common::local_db::address_collectors::collect_token_addresses;
use rain_orderbook_common::local_db::OrderbookIdentifier;
use rain_orderbook_common::local_db::{
    decode::{DecodedEvent, DecodedEventData},
    insert::generate_erc20_token_statements,
    query::SqlStatementBatch,
};
use rain_orderbook_common::rpc_client::RpcClient;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::{data_source::TokenMetadataFetcher, storage::fetch_existing_tokens};

pub(crate) struct TokenPrepResult {
    pub(crate) tokens_prefix_sql: SqlStatementBatch,
    pub(crate) decimals_by_addr: HashMap<Address, u8>,
}

pub(crate) async fn prepare_token_metadata<T>(
    db_path: &str,
    rpc_client: &RpcClient,
    ob_id: &OrderbookIdentifier,
    decoded_events: &[DecodedEventData<DecodedEvent>],
    token_fetcher: &T,
) -> Result<TokenPrepResult>
where
    T: TokenMetadataFetcher + Send + Sync,
{
    let address_set = collect_token_addresses(decoded_events);
    let all_token_addrs: Vec<Address> = address_set.into_iter().sorted().collect();

    if all_token_addrs.is_empty() {
        return Ok(TokenPrepResult {
            tokens_prefix_sql: SqlStatementBatch::new(),
            decimals_by_addr: HashMap::new(),
        });
    }

    let existing_rows = fetch_existing_tokens(db_path, ob_id, &all_token_addrs).await?;

    let mut decimals_by_addr: HashMap<Address, u8> = HashMap::new();
    let mut existing_lower: HashSet<String> = HashSet::new();
    for row in existing_rows.iter() {
        let key = row.token_address.to_ascii_lowercase();
        existing_lower.insert(key.clone());
        let address = Address::from_str(&row.token_address)
            .with_context(|| format!("Invalid address stored in DB: {}", row.token_address))?;
        decimals_by_addr.insert(address, row.decimals);
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
            tokens_prefix_sql: SqlStatementBatch::new(),
            decimals_by_addr,
        });
    }

    println!("Fetching metadata for {} new token(s)", missing_addrs.len());
    let fetched = token_fetcher.fetch(rpc_client, missing_addrs).await?;

    let tokens_prefix_sql = generate_erc20_token_statements(ob_id, &fetched);
    for (addr, info) in fetched.into_iter() {
        decimals_by_addr.insert(addr, info.decimals);
    }

    Ok(TokenPrepResult {
        tokens_prefix_sql,
        decimals_by_addr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use async_trait::async_trait;
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;
    use rain_orderbook_common::erc20::TokenInfo;
    use rain_orderbook_common::local_db::decode::{DecodedEvent, DecodedEventData, EventType};
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::executor::RusqliteExecutor;
    use crate::commands::local_db::sync::storage::DEFAULT_SCHEMA_SQL;
    use rain_orderbook_common::local_db::query::{LocalDbQueryExecutor, SqlStatement};

    struct NoopFetcher;

    #[async_trait]
    impl TokenMetadataFetcher for NoopFetcher {
        async fn fetch(&self, _: &RpcClient, _: Vec<Address>) -> Result<Vec<(Address, TokenInfo)>> {
            panic!("fetch should not be called")
        }
    }

    #[tokio::test]
    async fn prepare_tokens_uses_existing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tokens.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(DEFAULT_SCHEMA_SQL))
            .await
            .unwrap();
        exec.query_text(&SqlStatement::new(
                "INSERT INTO erc20_tokens (chain_id, orderbook_address, token_address, name, symbol, decimals) VALUES (1, '0x0101010101010101010101010101010101010101', '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'A', 'A', 18);",
            ))
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

        let rpc_client =
            RpcClient::new_with_urls(vec![Url::parse("https://test.com").unwrap()]).unwrap();
        let orderbook_address = Address::from([0x01; 20]);
        let prep = prepare_token_metadata(
            &db_path_str,
            &rpc_client,
            &OrderbookIdentifier::new(1, orderbook_address),
            &decoded,
            &NoopFetcher,
        )
        .await
        .unwrap();
        assert!(prep.tokens_prefix_sql.is_empty());
        assert_eq!(prep.decimals_by_addr.len(), 1);
        assert_eq!(prep.decimals_by_addr.get(&token_addr).copied(), Some(18));
    }
}
