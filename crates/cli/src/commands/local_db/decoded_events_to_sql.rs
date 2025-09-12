use alloy::primitives::{Address, U256};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use rain_math_float::Float;
use rain_orderbook_common::raindex_client::local_db::{
    tokens::collect_token_addresses, LocalDb, LocalDbError,
};
use serde::Deserialize;
use std::fs::{write, File};
use std::io::BufReader;
use std::str::FromStr;

#[derive(Args)]
pub struct DecodedEventsToSql {
    #[arg(long)]
    pub input_file: String,

    #[arg(long)]
    pub output_file: Option<String>,

    #[arg(long)]
    pub end_block: u64,

    #[arg(long, help = "Chain ID for erc20_tokens upserts")]
    pub chain_id: u32,

    #[arg(
        long,
        help = "Path to tokens.json providing metadata (decimals) for deposits when tokens are present"
    )]
    pub tokens_file: Option<String>,
}

impl DecodedEventsToSql {
    pub async fn execute(self) -> Result<()> {
        println!("Generating SQL statements...");

        let file = File::open(&self.input_file)
            .with_context(|| format!("Failed to open input file: {:?}", self.input_file))?;
        let reader = BufReader::new(file);

        let mut data: serde_json::Value =
            serde_json::from_reader(reader).context("Failed to parse JSON")?;
        let address_set = collect_token_addresses(&data);
        if !address_set.is_empty() {
            let mut decimals_by_addr = std::collections::HashMap::<String, u8>::new();

            // Require tokens.json when tokens are present
            let tokens_path = self.tokens_file.as_ref().ok_or_else(|| {
                anyhow!("--tokens-file is required to compute deposit_amounts for deposits")
            })?;
            let content = std::fs::read_to_string(tokens_path)?;
            let tokens_in: Vec<TokensFileEntry> = serde_json::from_str(&content)?;
            for t in tokens_in.iter() {
                let addr = Address::from_str(&t.address).map_err(|e| {
                    anyhow!(
                        "Invalid token address '{}' in tokens.json: {}",
                        t.address,
                        e
                    )
                })?;
                decimals_by_addr.insert(format!("0x{:x}", addr), t.decimals);
            }

            data = patch_deposit_amounts_with_decimals(data, &decimals_by_addr)
                .map_err(|e| anyhow!("{}", e))?;
        }

        let sql_statements = LocalDb::default()
            .decoded_events_to_sql(data, self.end_block)
            .map_err(|e| anyhow::anyhow!("Failed to generate SQL: {}", e))?;

        let output_path = self.output_file.unwrap_or_else(|| "events.sql".to_string());

        write(&output_path, sql_statements)
            .with_context(|| format!("Failed to write output file: {:?}", output_path))?;

        println!("SQL statements written to {:?}", output_path);
        Ok(())
    }
}

#[derive(Deserialize)]
struct TokensFileEntry {
    address: String,
    decimals: u8,
}

fn patch_deposit_amounts_with_decimals(
    decoded_events: serde_json::Value,
    decimals_by_addr: &std::collections::HashMap<String, u8>,
) -> Result<serde_json::Value, LocalDbError> {
    let events = decoded_events.as_array().ok_or_else(|| {
        LocalDbError::CustomError("Decoded events should be an array".to_string())
    })?;

    let mut patched = Vec::with_capacity(events.len());
    for ev in events.iter() {
        let mut ev_clone = ev.clone();
        let event_type = ev.get("event_type").and_then(|v| v.as_str()).unwrap_or("");
        if event_type == "DepositV2" {
            let obj = ev_clone.as_object_mut().ok_or_else(|| {
                LocalDbError::CustomError("Event should be an object".to_string())
            })?;
            let dd = obj
                .get_mut("decoded_data")
                .and_then(|v| v.as_object_mut())
                .ok_or_else(|| {
                    LocalDbError::CustomError("Missing decoded_data in DepositV2".to_string())
                })?;

            let token = dd.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
                LocalDbError::CustomError("Missing token in DepositV2".to_string())
            })?;
            let token_key = token.to_ascii_lowercase();
            let decimals = decimals_by_addr.get(&token_key).ok_or_else(|| {
                LocalDbError::CustomError(format!(
                    "Missing decimals for token {} required to compute deposit_amount",
                    token
                ))
            })?;

            let amt_hex = dd
                .get("deposit_amount_uint256")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    LocalDbError::CustomError(
                        "Missing deposit_amount_uint256 in DepositV2".to_string(),
                    )
                })?;
            let digits = amt_hex.strip_prefix("0x").unwrap_or(amt_hex);
            let amount = U256::from_str_radix(digits, 16).map_err(|e| {
                LocalDbError::CustomError(format!(
                    "Invalid deposit_amount_uint256 '{}': {}",
                    amt_hex, e
                ))
            })?;

            let amount_float = Float::from_fixed_decimal(amount, *decimals).map_err(|e| {
                LocalDbError::CustomError(format!(
                    "Float conversion failed for deposit_amount (token {}, decimals {}): {}",
                    token, decimals, e
                ))
            })?;

            dd.insert(
                "deposit_amount".to_string(),
                serde_json::Value::String(amount_float.as_hex()),
            );
        }

        patched.push(ev_clone);
    }

    Ok(serde_json::Value::Array(patched))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address as AlloyAddress;
    use rain_orderbook_test_fixtures::LocalEvm;
    use serde_json::json;
    use std::fs;
    use std::str::FromStr;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_with_custom_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("custom_output.sql");

        let test_data = json!([
            {"type": "test_event", "data": {"value": 123}}
        ]);

        fs::write(&input_file, serde_json::to_string(&test_data)?)?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
            end_block: 1000,
            chain_id: 1,
            tokens_file: None,
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file)?;
        assert!(!output_content.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_default_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let expected_output = temp_dir.path().join("events.sql");

        let test_data = json!([
            {"type": "test_event", "data": {"value": 456}}
        ]);

        fs::write(&input_file, serde_json::to_string(&test_data)?)?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
            end_block: 2000,
            chain_id: 1,
            tokens_file: None,
        };

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = cmd.execute().await;

        std::env::set_current_dir(original_dir)?;

        result?;

        assert!(expected_output.exists());
        let output_content = fs::read_to_string(&expected_output)?;
        assert!(!output_content.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_invalid_input_file() {
        let cmd = DecodedEventsToSql {
            input_file: "nonexistent_file.json".to_string(),
            output_file: None,
            end_block: 1000,
            chain_id: 1,
            tokens_file: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_invalid_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("invalid.json");

        fs::write(&input_file, "invalid json content")?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
            end_block: 1000,
            chain_id: 1,
            tokens_file: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_execute_computes_deposit_amount_without_token_upsert() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("decoded.json");
        let output_file = temp_dir.path().join("events.sql");

        // Start local EVM with a token
        let local_evm = LocalEvm::new_with_tokens(1).await;
        let token = local_evm.tokens[0].clone();
        let token_addr: AlloyAddress = *token.address();

        // Build a decoded DepositV2 event requiring decimals
        let decoded = json!([
            {
                "event_type": "DepositV2",
                "block_number": "0x3e8",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0x111",
                "log_index": "0x0",
                "decoded_data": {
                    "sender": "0x0000000000000000000000000000000000000001",
                    "token": format!("0x{:x}", token_addr),
                    "vault_id": "0x1",
                    "deposit_amount_uint256": "0x0de0b6b3a7640000" // 1e18
                }
            }
        ]);
        std::fs::write(&input_file, serde_json::to_string(&decoded)?)?;

        // Build tokens.json with decimals for the token used in decoded events
        let tokens_path = temp_dir.path().join("tokens.json");
        let tokens_json = format!(
            "[{{\"address\":\"0x{:x}\",\"name\":\"\",\"symbol\":\"\",\"decimals\":18}}]",
            token_addr
        );
        std::fs::write(&tokens_path, tokens_json)?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
            end_block: 1000,
            chain_id: 1,
            tokens_file: Some(tokens_path.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        let sql = std::fs::read_to_string(&output_file)?;
        assert!(sql.contains("INSERT INTO deposits"));
        // Expect deposit_amount to be Float for 1e18 with 18 decimals => 1
        let expected =
            Float::from_fixed_decimal(U256::from_str("1000000000000000000").unwrap(), 18)
                .unwrap()
                .as_hex();
        assert!(sql.contains(&expected));

        Ok(())
    }
}
