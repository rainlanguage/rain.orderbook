use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::erc20::TokenInfo;
use rain_orderbook_common::raindex_client::local_db::insert::generate_erc20_tokens_sql;
use serde::Deserialize;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone, Parser)]
#[command(about = "Turn tokens.json into erc20_tokens upsert SQL")]
pub struct TokensToSql {
    #[clap(long, help = "Chain ID for DB rows")]
    pub chain_id: u32,
    #[clap(long, help = "Path to tokens JSON")]
    pub input_file: String,
    #[clap(
        long,
        help = "Path to write generated token SQL",
        default_value = "tokens.sql"
    )]
    pub output_file: String,
}

#[derive(Debug, Deserialize)]
struct TokenJsonIn {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
}

impl TokensToSql {
    pub async fn execute(self) -> Result<()> {
        let content = fs::read_to_string(&self.input_file)?;
        let tokens_in: Vec<TokenJsonIn> = serde_json::from_str(&content)?;

        let mut tokens: Vec<(Address, TokenInfo)> = Vec::new();
        for t in tokens_in.into_iter() {
            let addr = Address::from_str(&t.address)
                .map_err(|e| anyhow!("Invalid token address '{}': {}", t.address, e))?;
            tokens.push((
                addr,
                TokenInfo {
                    decimals: t.decimals,
                    name: t.name,
                    symbol: t.symbol,
                },
            ));
        }

        let sql = generate_erc20_tokens_sql(self.chain_id, &tokens);
        fs::write(&self.output_file, sql)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn tokens_to_sql_writes_insert() {
        let temp = TempDir::new().unwrap();
        let input_path = temp.path().join("tokens.json");
        let output_path = temp.path().join("tokens.sql");

        let json = r#"[
            {"address":"0x0101010101010101010101010101010101010101","name":"Foo","symbol":"FOO","decimals":18},
            {"address":"0x0202020202020202020202020202020202020202","name":"Bar","symbol":"BAR","decimals":6}
        ]"#;
        fs::write(&input_path, json).unwrap();

        let cmd = TokensToSql {
            chain_id: 1,
            input_file: input_path.to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
        };
        cmd.execute().await.unwrap();

        let sql = fs::read_to_string(&output_path).unwrap();
        assert!(sql.contains("INSERT INTO erc20_tokens"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("FOO"));
        assert!(sql.contains("BAR"));
    }
}
