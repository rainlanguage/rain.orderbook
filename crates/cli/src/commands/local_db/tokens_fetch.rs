use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::token_fetch::fetch_erc20_metadata_concurrent;
use rain_orderbook_common::raindex_client::local_db::tokens::collect_token_addresses;
use serde::Serialize;
use std::fs;
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(about = "Fetch ERC20 metadata from decoded events and write tokens.json")]
pub struct TokensFetch {
    #[clap(long, help = "Direct RPC URL(s); repeat to add multiple", action = clap::ArgAction::Append, value_name = "URL")]
    pub rpc: Vec<String>,
    #[clap(long, help = "Path to decoded events JSON")]
    pub input_file: String,
    #[clap(
        long,
        help = "Path to write tokens JSON",
        default_value = "tokens.json"
    )]
    pub output_file: String,
}

#[derive(Serialize)]
struct TokenJson {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
}

impl TokensFetch {
    pub async fn execute(self) -> Result<()> {
        if self.rpc.is_empty() {
            return Err(anyhow!(
                "--rpc is required (one or more URLs) for tokens-fetch"
            ));
        }

        // Read decoded events
        let decoded_str = fs::read_to_string(&self.input_file)?;
        let decoded: serde_json::Value = serde_json::from_str(&decoded_str)?;

        // Collect token addresses
        let mut addrs: Vec<Address> = collect_token_addresses(&decoded).into_iter().collect();
        addrs.sort();
        if addrs.is_empty() {
            fs::write(&self.output_file, "[]")?;
            return Ok(());
        }

        // Parse RPC URLs
        let rpcs: Vec<Url> = self
            .rpc
            .iter()
            .map(|u| Url::parse(u))
            .collect::<Result<_, _>>()
            .map_err(|e| anyhow!("Invalid --rpc URL: {}", e))?;

        // Fetch metadata
        let fetched = fetch_erc20_metadata_concurrent(rpcs, addrs).await?;

        // Serialize to tokens.json
        let tokens: Vec<TokenJson> = fetched
            .into_iter()
            .map(|(addr, info)| TokenJson {
                address: format!("0x{:x}", addr),
                name: info.name,
                symbol: info.symbol,
                decimals: info.decimals,
            })
            .collect();
        let json = serde_json::to_string_pretty(&tokens)?;
        fs::write(&self.output_file, json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tokens_fetch_produces_json() {
        let temp = TempDir::new().unwrap();
        let input_path = temp.path().join("decoded.json");
        let output_path = temp.path().join("tokens.json");

        // Local EVM and a decoded event that references its token address
        let local_evm = LocalEvm::new_with_tokens(1).await;
        let token = local_evm.tokens[0].clone();
        let decoded = json!([
            {"event_type":"DepositV2","decoded_data": {"token": format!("0x{:x}", token.address())}}
        ]);
        fs::write(&input_path, serde_json::to_string(&decoded).unwrap()).unwrap();

        let cmd = TokensFetch {
            rpc: vec![local_evm.url()],
            input_file: input_path.to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
        };

        cmd.execute().await.unwrap();
        let out = fs::read_to_string(&output_path).unwrap();
        assert!(out.contains("symbol"));
        assert!(out.contains("decimals"));
        assert!(out
            .to_ascii_lowercase()
            .contains(&format!("0x{:x}", token.address())));
    }
}
