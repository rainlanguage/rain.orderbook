use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use clap::Parser;
use futures::stream::StreamExt;
use rain_orderbook_common::erc20::{TokenInfo, ERC20};
use rain_orderbook_common::raindex_client::local_db::{
    insert::generate_erc20_tokens_sql, tokens::collect_token_addresses,
};
use rain_orderbook_common::raindex_client::local_db::{LocalDb, LocalDbError};
use serde_json::Value;
use std::fs;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(about = "Generate SQL upserts for ERC20 tokens from decoded events")]
pub struct TokenSql {
    #[clap(long, help = "Chain ID for DB rows")]
    pub chain_id: u32,
    #[clap(long, help = "Direct RPC URL(s); repeat to add multiple", action = clap::ArgAction::Append, value_name = "URL")]
    pub rpc: Vec<String>,
    #[clap(long, help = "Path to decoded events JSON")]
    pub input_file: String,
    #[clap(
        long,
        help = "Path to write generated token SQL",
        default_value = "tokens.sql"
    )]
    pub output_file: String,
    #[clap(
        long,
        default_value_t = 3,
        help = "Max token fetch retry attempts per address"
    )]
    pub max_retries: u32,
    #[clap(long, default_value_t = 500, help = "Delay between retries in ms")]
    pub retry_delay_ms: u64,
    #[clap(long, default_value_t = 16, help = "Concurrent token metadata fetches")]
    pub concurrency: usize,
    #[clap(
        long,
        help = "Allow partial success; write successes even if some tokens fail"
    )]
    pub allow_partial: bool,
    #[clap(
        long,
        help = "Where to write failures JSON when --allow-partial is set"
    )]
    pub failures_file: Option<String>,
}

impl TokenSql {
    pub async fn execute(self) -> Result<()> {
        // Read and parse decoded events
        let input_content = fs::read_to_string(&self.input_file)?;
        let decoded: Value = serde_json::from_str(&input_content)?;

        // Build LocalDb for RPC URL selection
        // Require at least one standard RPC URL; HyperRPC is not supported for token-sql.
        if self.rpc.is_empty() {
            return Err(anyhow!(
                "--rpc is required (one or more URLs) for token-sql"
            ));
        }
        let mut urls: Vec<Url> = Vec::new();
        for rpc_url in &self.rpc {
            let url = Url::parse(rpc_url)
                .map_err(|e| anyhow!("Invalid --rpc URL '{}': {}", rpc_url, e))?;
            urls.push(url);
        }
        let local_db = LocalDb::new_with_regular_rpcs(urls);

        // Collect unique token addresses from decoded JSON
        let address_set = collect_token_addresses(&decoded);
        let mut addrs: Vec<Address> = address_set.into_iter().collect();
        addrs.sort();

        // If nothing to fetch, write empty SQL and exit
        if addrs.is_empty() {
            fs::write(&self.output_file, "")?;
            return Ok(());
        }

        println!(
            "token-sql: found {} unique token address(es); using {} RPC(s)",
            addrs.len(),
            local_db.rpc_urls().len()
        );

        // Fetch token metadata concurrently with retries
        let rpcs = local_db.rpc_urls().to_vec();
        let fetched = fetch_erc20_metadata_concurrent(
            rpcs,
            addrs,
            self.max_retries,
            self.retry_delay_ms,
            self.concurrency,
            self.allow_partial,
            self.failures_file.clone(),
        )
        .await?;

        // Build SQL and write to file
        let sql = generate_erc20_tokens_sql(self.chain_id, &fetched);
        fs::write(&self.output_file, sql)?;

        println!("token-sql: wrote {}", &self.output_file);
        Ok(())
    }
}

async fn fetch_erc20_metadata_concurrent(
    rpcs: Vec<Url>,
    addresses: Vec<Address>,
    max_retries: u32,
    retry_delay_ms: u64,
    concurrency: usize,
    allow_partial: bool,
    failures_file: Option<String>,
) -> Result<Vec<(Address, TokenInfo)>, LocalDbError> {
    async fn fetch_with_retries(
        rpcs: Vec<Url>,
        addr: Address,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Result<(Address, TokenInfo), LocalDbError> {
        let erc20 = ERC20::new(rpcs.clone(), addr);
        let mut attempt: u32 = 0;
        loop {
            match erc20.token_info(None).await {
                Ok(info) => return Ok((addr, info)),
                Err(e) => {
                    attempt += 1;
                    if attempt >= max_retries {
                        return Err(LocalDbError::CustomError(format!(
                            "Failed to fetch token info for 0x{:x} after {} attempts: {}",
                            addr, max_retries, e
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }

    let results: Vec<Result<(Address, TokenInfo), LocalDbError>> =
        futures::stream::iter(addresses.into_iter().map(|addr| {
            let rpcs = rpcs.clone();
            async move { fetch_with_retries(rpcs, addr, max_retries, retry_delay_ms).await }
        }))
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut successes: Vec<(Address, TokenInfo)> = Vec::new();
    let mut failures: Vec<(Address, String)> = Vec::new();
    for r in results {
        match r {
            Ok(pair) => successes.push(pair),
            Err(e) => {
                if allow_partial {
                    // Parse out address from message if possible; else generic
                    let msg = e.to_string();
                    let addr_str = msg
                        .find("0x")
                        .and_then(|idx| msg.get(idx..idx + 42))
                        .unwrap_or("")
                        .to_string();
                    let addr = Address::from_str(&addr_str).unwrap_or(Address::ZERO);
                    failures.push((addr, msg));
                } else {
                    return Err(e);
                }
            }
        }
    }

    if allow_partial {
        if let Some(path) = failures_file {
            // Serialize failures as JSON { address: string, error: string }[]
            let as_json: Vec<serde_json::Value> = failures
                .into_iter()
                .map(|(addr, err)| {
                    let addr_str = if addr == Address::ZERO {
                        "unknown".to_string()
                    } else {
                        format!("0x{:x}", addr)
                    };
                    serde_json::json!({"address": addr_str, "error": err})
                })
                .collect();
            let _ = fs::write(
                path,
                serde_json::to_string_pretty(&as_json).unwrap_or_else(|_| "[]".to_string()),
            );
        }
    }

    Ok(successes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn token_sql_writes_empty_when_no_tokens() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("decoded.json");
        let output_path = temp_dir.path().join("tokens.sql");

        fs::write(&input_path, "[]").unwrap();

        let cmd = TokenSql {
            chain_id: 8453,
            rpc: vec!["http://localhost:8545".to_string()], // not used since no tokens
            input_file: input_path.to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
            max_retries: 3,
            retry_delay_ms: 10,
            concurrency: 4,
            allow_partial: false,
            failures_file: None,
        };

        let res = cmd.execute().await;
        assert!(res.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn token_sql_generates_sql_from_local_evm() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("decoded.json");
        let output_path = temp_dir.path().join("tokens.sql");

        // Spin up local EVM with a token
        let local_evm = LocalEvm::new_with_tokens(1).await;
        let token = local_evm.tokens[0].clone();
        let token_addr: Address = *token.address();

        // Build minimal decoded events JSON including the token address in a DepositV2 event
        let decoded = json!([{
            "event_type": "DepositV2",
            "decoded_data": {"token": format!("0x{:x}", token_addr)}
        }]);
        fs::write(&input_path, serde_json::to_string(&decoded).unwrap()).unwrap();

        let cmd = TokenSql {
            chain_id: 8453,
            rpc: vec![local_evm.url()],
            input_file: input_path.to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
            max_retries: 3,
            retry_delay_ms: 10,
            concurrency: 4,
            allow_partial: false,
            failures_file: None,
        };

        let res = cmd.execute().await;
        assert!(res.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("INSERT INTO erc20_tokens"));
        assert!(content
            .to_ascii_lowercase()
            .contains(&format!("0x{:x}", token_addr)));
        assert!(content.contains("Token1"));
        assert!(content.contains("TOKEN1"));
    }
}
