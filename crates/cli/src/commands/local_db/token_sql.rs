use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::token_fetch::fetch_erc20_metadata_concurrent;
use rain_orderbook_common::raindex_client::local_db::LocalDb;
use rain_orderbook_common::raindex_client::local_db::{
    insert::generate_erc20_tokens_sql, tokens::collect_token_addresses,
};
use serde_json::Value;
use std::fs;
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
        let fetched = fetch_erc20_metadata_concurrent(rpcs, addrs).await?;

        // Build SQL and write to file
        let sql = generate_erc20_tokens_sql(self.chain_id, &fetched);
        fs::write(&self.output_file, sql)?;

        println!("token-sql: wrote {}", &self.output_file);
        Ok(())
    }
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
