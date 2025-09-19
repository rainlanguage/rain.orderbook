use super::sqlite::{sqlite_execute, sqlite_has_required_tables, sqlite_query_json};
use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use rain_orderbook_common::erc20::TokenInfo;
use rain_orderbook_common::raindex_client::local_db::helpers::patch_deposit_amounts_with_decimals;
use rain_orderbook_common::raindex_client::local_db::insert::generate_erc20_tokens_sql;
use rain_orderbook_common::raindex_client::local_db::query::create_tables::REQUIRED_TABLES;
use rain_orderbook_common::raindex_client::local_db::token_fetch::fetch_erc20_metadata_concurrent;
use rain_orderbook_common::raindex_client::local_db::tokens::collect_token_addresses;
use rain_orderbook_common::raindex_client::local_db::LocalDb;
use serde::Deserialize;
use serde_json::Value;
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

#[async_trait]
pub trait SyncDataSource {
    async fn latest_block(&self) -> Result<u64>;
    async fn fetch_events(
        &self,
        orderbook_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Value>;
    fn decode_events(&self, events: Value) -> Result<Value>;
    fn events_to_sql(
        &self,
        decoded_events: Value,
        end_block: u64,
        prefix_sql: &str,
    ) -> Result<String>;
    fn rpc_urls(&self) -> &[Url];
}

#[async_trait]
pub trait TokenMetadataFetcher {
    async fn fetch(&self, rpcs: &[Url], missing: Vec<Address>)
        -> Result<Vec<(Address, TokenInfo)>>;
}

pub struct DefaultTokenFetcher;

#[async_trait]
impl TokenMetadataFetcher for DefaultTokenFetcher {
    async fn fetch(
        &self,
        rpcs: &[Url],
        missing: Vec<Address>,
    ) -> Result<Vec<(Address, TokenInfo)>> {
        if missing.is_empty() {
            return Ok(vec![]);
        }

        let fetched = fetch_erc20_metadata_concurrent(rpcs.to_vec(), missing)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(fetched)
    }
}

#[async_trait]
impl SyncDataSource for LocalDb {
    async fn latest_block(&self) -> Result<u64> {
        self.rpc_client()
            .get_latest_block_number(self.rpc_urls())
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn fetch_events(
        &self,
        orderbook_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Value> {
        self.fetch_events(orderbook_address, start_block, end_block)
            .await
            .map_err(|e| anyhow!(e))
    }

    fn decode_events(&self, events: Value) -> Result<Value> {
        self.decode_events(events).map_err(|e| anyhow!(e))
    }

    fn events_to_sql(
        &self,
        decoded_events: Value,
        end_block: u64,
        prefix_sql: &str,
    ) -> Result<String> {
        self.decoded_events_to_sql_with_prefix(decoded_events, end_block, prefix_sql)
            .map_err(|e| anyhow!("Failed to generate SQL: {}", e))
    }

    fn rpc_urls(&self) -> &[Url] {
        self.rpc_urls()
    }
}

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

        let SyncLocalDb {
            db_path,
            chain_id,
            orderbook_address,
            deployment_block,
            start_block,
            end_block,
            api_token,
            rpc,
        } = self;

        let local_db = build_local_db(chain_id, api_token, rpc)?;
        let token_fetcher = DefaultTokenFetcher;
        let runner = SyncRunner::new(&db_path, &local_db, &token_fetcher);
        let params = SyncParams {
            chain_id,
            orderbook_address: &orderbook_address,
            deployment_block,
            start_block,
            end_block,
        };

        runner.run(&params).await
    }
}

pub struct SyncRunner<'a, D, T> {
    db_path: &'a str,
    data_source: &'a D,
    token_fetcher: &'a T,
}

impl<'a, D, T> SyncRunner<'a, D, T>
where
    D: SyncDataSource + Send + Sync,
    T: TokenMetadataFetcher + Send + Sync,
{
    pub fn new(db_path: &'a str, data_source: &'a D, token_fetcher: &'a T) -> Self {
        Self {
            db_path,
            data_source,
            token_fetcher,
        }
    }

    pub async fn run(&self, params: &SyncParams<'_>) -> Result<()> {
        let schema_applied = ensure_schema(self.db_path)?;
        if schema_applied {
            println!("Database schema initialized at {}", self.db_path);
        }

        let last_synced_block = fetch_last_synced(self.db_path)?;
        println!("Current last_synced_block: {}", last_synced_block);

        let mut start_block = params
            .start_block
            .unwrap_or_else(|| default_start_block(last_synced_block, params.deployment_block));

        if last_synced_block > 0 && start_block <= last_synced_block {
            println!(
                "Provided start block {} is <= last_synced_block {}; bumping to {}",
                start_block,
                last_synced_block,
                last_synced_block + 1
            );
            start_block = last_synced_block + 1;
        }

        if last_synced_block == 0 && start_block < params.deployment_block {
            println!(
                "Start block {} is before deployment block {}; using deployment block",
                start_block, params.deployment_block
            );
            start_block = params.deployment_block;
        }

        let latest_block = self.data_source.latest_block().await?;
        println!("Network latest block: {}", latest_block);

        let mut target_block = params.end_block.unwrap_or(latest_block);
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
            params.orderbook_address, start_block, target_block
        );
        let events = self
            .data_source
            .fetch_events(params.orderbook_address, start_block, target_block)
            .await?;
        let raw_event_count = events.as_array().map(|a| a.len()).unwrap_or(0);
        println!("Fetched {} raw events", raw_event_count);

        println!("Decoding events");
        let decoded_events = self.data_source.decode_events(events)?;
        let decoded_count = decoded_events.as_array().map(|a| a.len()).unwrap_or(0);
        println!("Decoded {} events", decoded_count);

        println!("Preparing token metadata");
        let token_prep = prepare_token_metadata(
            self.db_path,
            self.data_source.rpc_urls(),
            params.chain_id,
            &decoded_events,
            self.token_fetcher,
        )
        .await?;

        let patched_events =
            patch_deposit_amounts_with_decimals(decoded_events, &token_prep.decimals_by_addr)
                .map_err(|e| anyhow!(e))?;

        println!("Generating SQL for {} events", decoded_count);
        let sql = self.data_source.events_to_sql(
            patched_events,
            target_block,
            &token_prep.tokens_prefix_sql,
        )?;

        println!("Applying SQL to {}", self.db_path);
        sqlite_execute(self.db_path, &sql)?;

        println!("Sync complete. last_synced_block is now {}", target_block);
        Ok(())
    }
}

pub struct SyncParams<'a> {
    pub chain_id: u32,
    pub orderbook_address: &'a str,
    pub deployment_block: u64,
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
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

fn build_local_db(
    chain_id: u32,
    api_token: Option<String>,
    rpc_urls: Vec<String>,
) -> Result<LocalDb> {
    if !rpc_urls.is_empty() {
        let urls: Vec<Url> = rpc_urls
            .iter()
            .map(|raw| Url::parse(raw).with_context(|| format!("Invalid RPC URL: {}", raw)))
            .collect::<Result<_, _>>()?;
        return Ok(LocalDb::new_with_regular_rpcs(urls));
    }

    let api_token = api_token.ok_or_else(|| anyhow!("Provide either --rpc or --api-token"))?;
    LocalDb::new_with_hyper_rpc(chain_id, api_token).map_err(|e| anyhow!(e))
}

struct TokenPrepResult {
    tokens_prefix_sql: String,
    decimals_by_addr: HashMap<String, u8>,
}
async fn prepare_token_metadata<T>(
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
    use alloy::primitives::U256;
    use rain_math_float::Float;
    use serde_json::json;
    use std::sync::Mutex;
    use tempfile::TempDir;

    #[test]
    fn default_start_behavior() {
        assert_eq!(default_start_block(0, 123), 123);
        assert_eq!(default_start_block(10, 5), 11);
    }

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

    struct MockDataSource {
        latest_block: u64,
        events: Value,
        decoded: Value,
        sql_result: String,
        rpc_urls: Vec<Url>,
        fetch_calls: Mutex<Vec<(String, u64, u64)>>,
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

        fn decode_events(&self, events: Value) -> Result<Value> {
            assert_eq!(events, self.events);
            Ok(self.decoded.clone())
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
            sql_result: "BEGIN TRANSACTION;
UPDATE sync_status SET last_synced_block = ?end_block, updated_at = CURRENT_TIMESTAMP WHERE id = 1;
COMMIT;
"
            .to_string(),
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            fetch_calls: Mutex::new(Vec::new()),
            prefixes: Mutex::new(Vec::new()),
            patched_events: Mutex::new(Vec::new()),
        };

        let runner = SyncRunner::new(&db_path_str, &data_source, &PanicFetcher);
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

        let expected_sql = "BEGIN TRANSACTION;
UPDATE sync_status SET last_synced_block = ?end_block, updated_at = CURRENT_TIMESTAMP WHERE id = 1;
COMMIT;
"
        .to_string();

        let data_source = MockDataSource {
            latest_block: 120,
            events,
            decoded,
            sql_result: expected_sql,
            rpc_urls: vec![Url::parse("http://localhost:1").unwrap()],
            fetch_calls: Mutex::new(Vec::new()),
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

        let runner = SyncRunner::new(&db_path_str, &data_source, &token_fetcher);
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
}
