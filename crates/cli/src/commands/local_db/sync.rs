use super::sqlite::{sqlite_execute, sqlite_has_required_tables, sqlite_query_json};
use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::{
    helpers::patch_deposit_amounts_with_decimals, insert::generate_erc20_tokens_sql,
    query::create_tables::REQUIRED_TABLES, token_fetch::fetch_erc20_metadata_concurrent,
    tokens::collect_token_addresses, LocalDb,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use url::Url;

const DEFAULT_SCHEMA_SQL: &str =
    include_str!("../../../../common/src/raindex_client/local_db/query/create_tables/query.sql");
const SYNC_STATUS_QUERY: &str = include_str!(
    "../../../../common/src/raindex_client/local_db/query/fetch_last_synced_block/query.sql"
);
const ERC20_QUERY_TEMPLATE: &str = include_str!(
    "../../../../common/src/raindex_client/local_db/query/fetch_erc20_tokens_by_addresses/query.sql"
);

#[derive(Debug, Clone, Parser)]
#[command(about = "Incrementally sync a local SQLite database using on-chain events")]
pub struct SyncLocalDb {
    #[clap(long, help = "Path to SQLite DB that stores indexed data")]
    pub db_path: String,

    #[clap(long, help = "Chain ID for the orderbook deployment")]
    pub chain_id: u32,

    #[clap(
        long,
        help = "Orderbook contract address to index",
        value_name = "0xADDRESS"
    )]
    pub orderbook_address: String,

    #[clap(long, help = "Deployment block used when DB is empty")]
    pub deployment_block: u64,

    #[clap(long, help = "Optional override for start block")]
    pub start_block: Option<u64>,

    #[clap(long, help = "Optional override for end block")]
    pub end_block: Option<u64>,

    #[clap(long, help = "Hyperlane API token (used if no --rpc values provided)")]
    pub api_token: Option<String>,

    #[clap(
        long,
        action = clap::ArgAction::Append,
        value_name = "URL",
        help = "Direct RPC URL(s); repeat to provide multiple"
    )]
    pub rpc: Vec<String>,
}

impl SyncLocalDb {
    pub async fn execute(self) -> Result<()> {
        println!("Starting local DB sync");
        let schema_applied = ensure_schema(&self.db_path)?;
        if schema_applied {
            println!("Database schema initialized at {}", self.db_path);
        }

        let last_synced_block = fetch_last_synced(&self.db_path)?;
        println!("Current last_synced_block: {}", last_synced_block);

        let mut start_block = self
            .start_block
            .unwrap_or_else(|| default_start_block(last_synced_block, self.deployment_block));

        if last_synced_block > 0 && start_block <= last_synced_block {
            println!(
                "Provided start block {} is <= last_synced_block {}; bumping to {}",
                start_block,
                last_synced_block,
                last_synced_block + 1
            );
            start_block = last_synced_block + 1;
        }

        if last_synced_block == 0 && start_block < self.deployment_block {
            println!(
                "Start block {} is before deployment block {}; using deployment block",
                start_block, self.deployment_block
            );
            start_block = self.deployment_block;
        }

        let local_db = build_local_db(&self)?;
        let latest_block = local_db
            .rpc_client()
            .get_latest_block_number(local_db.rpc_urls())
            .await
            .map_err(anyhow::Error::from)?;
        println!("Network latest block: {}", latest_block);

        let mut target_block = self.end_block.unwrap_or(latest_block);
        if target_block > latest_block {
            println!(
                "Requested end block {} is beyond chain head {}; clamping",
                target_block, latest_block
            );
            target_block = latest_block;
        }

        if start_block > target_block {
            println!(
                "Nothing to do (start block {} exceeds target block {})",
                start_block, target_block
            );
            return Ok(());
        }

        println!(
            "Fetching events for {} from block {} to {}",
            self.orderbook_address, start_block, target_block
        );
        let events = local_db
            .fetch_events(&self.orderbook_address, start_block, target_block)
            .await
            .map_err(anyhow::Error::from)?;
        let raw_event_count = events.as_array().map(|a| a.len()).unwrap_or(0);
        println!("Fetched {} raw events", raw_event_count);

        println!("Decoding events");
        let decoded_events = local_db
            .decode_events(events)
            .map_err(anyhow::Error::from)?;
        let decoded_count = decoded_events.as_array().map(|a| a.len()).unwrap_or(0);
        println!("Decoded {} events", decoded_count);

        println!("Preparing token metadata");
        let token_prep =
            prepare_token_metadata(&self.db_path, &local_db, self.chain_id, &decoded_events)
                .await?;

        let patched_events =
            patch_deposit_amounts_with_decimals(decoded_events, &token_prep.decimals_by_addr)
                .map_err(anyhow::Error::from)?;

        println!("Generating SQL for {} events", decoded_count);
        let sql = local_db
            .decoded_events_to_sql_with_prefix(
                patched_events,
                target_block,
                &token_prep.tokens_prefix_sql,
            )
            .map_err(|e| anyhow!("Failed to generate SQL: {}", e))?;

        println!("Applying SQL to {}", self.db_path);
        sqlite_execute(&self.db_path, &sql)?;

        println!("Sync complete. last_synced_block is now {}", target_block);
        Ok(())
    }
}

fn default_start_block(last_synced_block: u64, deployment_block: u64) -> u64 {
    if last_synced_block == 0 {
        deployment_block
    } else {
        last_synced_block + 1
    }
}

fn ensure_schema(db_path: &str) -> Result<bool> {
    if sqlite_has_required_tables(db_path, REQUIRED_TABLES)? {
        return Ok(false);
    }

    sqlite_execute(db_path, DEFAULT_SCHEMA_SQL)?;
    Ok(true)
}

fn fetch_last_synced(db_path: &str) -> Result<u64> {
    let rows: Vec<SyncStatusRow> = sqlite_query_json(db_path, SYNC_STATUS_QUERY)?;
    Ok(rows.first().map(|row| row.last_synced_block).unwrap_or(0))
}

fn build_local_db(cmd: &SyncLocalDb) -> Result<LocalDb> {
    if !cmd.rpc.is_empty() {
        let urls: Vec<Url> = cmd
            .rpc
            .iter()
            .map(|raw| Url::parse(raw).with_context(|| format!("Invalid RPC URL: {}", raw)))
            .collect::<Result<_, _>>()?;
        Ok(LocalDb::new_with_regular_rpcs(urls))
    } else {
        let api_token = cmd
            .api_token
            .as_ref()
            .ok_or_else(|| anyhow!("Provide either --rpc or --api-token"))?;
        LocalDb::new_with_hyper_rpc(cmd.chain_id, api_token.clone()).map_err(anyhow::Error::from)
    }
}

struct TokenPrepResult {
    tokens_prefix_sql: String,
    decimals_by_addr: HashMap<String, u8>,
}

async fn prepare_token_metadata(
    db_path: &str,
    local_db: &LocalDb,
    chain_id: u32,
    decoded_events: &serde_json::Value,
) -> Result<TokenPrepResult> {
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
        let key = format!("0x{:x}", addr);
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
    let fetched = fetch_erc20_metadata_concurrent(local_db.rpc_urls().to_vec(), missing_addrs)
        .await
        .map_err(anyhow::Error::from)?;

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

fn fetch_existing_tokens(
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
struct SyncStatusRow {
    last_synced_block: u64,
}

#[derive(Debug, Deserialize)]
struct Erc20TokenRow {
    address: String,
    decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_common::raindex_client::local_db::LocalDb;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn default_start_behavior() {
        assert_eq!(default_start_block(0, 123), 123);
        assert_eq!(default_start_block(10, 5), 11);
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

        let prep = prepare_token_metadata(&db_path_str, &LocalDb::default(), 1, &decoded)
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
