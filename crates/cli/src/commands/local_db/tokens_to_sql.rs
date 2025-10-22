use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use rain_orderbook_common::erc20::TokenInfo;
use rain_orderbook_common::local_db::{
    insert::generate_erc20_token_statements,
    query::{SqlStatement, SqlStatementBatch, SqlValue},
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use tokio::fs;

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
        let TokensToSql {
            chain_id,
            input_file,
            output_file,
        } = self;

        let content = fs::read_to_string(&input_file)
            .await
            .with_context(|| format!("failed to read tokens JSON from {}", &input_file))?;
        let tokens_in: Vec<TokenJsonIn> = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse tokens JSON from {}", &input_file))?;

        let mut tokens_map: BTreeMap<Address, TokenInfo> = BTreeMap::new();
        for t in tokens_in.into_iter() {
            let addr = Address::from_str(&t.address)
                .map_err(|e| anyhow!("Invalid token address '{}': {}", t.address, e))?;
            tokens_map.insert(
                addr,
                TokenInfo {
                    decimals: t.decimals,
                    name: t.name,
                    symbol: t.symbol,
                },
            );
        }
        let tokens: Vec<(Address, TokenInfo)> = tokens_map.into_iter().collect();

        let batch = generate_erc20_token_statements(chain_id, &tokens);
        fs::write(&output_file, batch_to_string(&batch))
            .await
            .with_context(|| format!("failed to write token SQL to {}", &output_file))?;
        Ok(())
    }
}

fn batch_to_string(batch: &SqlStatementBatch) -> String {
    let mut out = String::new();
    for stmt in batch.statements() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        let sql = materialize_statement(stmt);
        out.push_str(&sql);
        if !sql.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn materialize_statement(statement: &SqlStatement) -> String {
    let mut sql = statement.sql().to_string();
    for (idx, value) in statement.params().iter().enumerate().rev() {
        let placeholder = format!("?{}", idx + 1);
        sql = sql.replacen(&placeholder, &render_sql_value(value), 1);
    }
    sql
}

fn render_sql_value(value: &SqlValue) -> String {
    match value {
        SqlValue::Text(text) => format!("'{}'", text.replace('\'', "''")),
        SqlValue::I64(num) => num.to_string(),
        SqlValue::U64(num) => num.to_string(),
        SqlValue::Null => "NULL".to_string(),
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
        std::fs::write(&input_path, json).unwrap();

        let cmd = TokensToSql {
            chain_id: 1,
            input_file: input_path.to_string_lossy().to_string(),
            output_file: output_path.to_string_lossy().to_string(),
        };
        cmd.execute().await.unwrap();

        let sql = std::fs::read_to_string(&output_path).unwrap();
        assert!(sql.contains("INSERT INTO erc20_tokens"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("FOO"));
        assert!(sql.contains("BAR"));
    }
}
