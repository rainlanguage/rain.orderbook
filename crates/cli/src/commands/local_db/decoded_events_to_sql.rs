use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::{
    decode::{DecodedEvent, DecodedEventData},
    helpers::ensure_deposit_decimals_available,
    insert::{decoded_events_to_sql_with_options, SqlGenerationOptions},
};
use std::collections::HashMap;
use std::fs::{write, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
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

        let decoded_events: Vec<DecodedEventData<DecodedEvent>> =
            serde_json::from_reader(reader).context("Failed to parse decoded events JSON")?;

        let needs_decimals = decoded_events
            .iter()
            .any(|event| matches!(event.decoded_data, DecodedEvent::DepositV2(_)));

        let mut deposit_decimals = HashMap::new();
        if needs_decimals {
            let tokens_path = self.tokens_file.as_ref().ok_or_else(|| {
                anyhow!("--tokens-file is required to compute deposit_amounts for deposits")
            })?;
            deposit_decimals = load_tokens_file(tokens_path)?;
            ensure_deposit_decimals_available(&decoded_events, &deposit_decimals)?;
        }

        let options = SqlGenerationOptions {
            deposit_decimals: if needs_decimals {
                Some(&deposit_decimals)
            } else {
                None
            },
            ..Default::default()
        };

        let sql_statements =
            decoded_events_to_sql_with_options(&decoded_events, self.end_block, &options)
                .map_err(|e| anyhow::anyhow!("Failed to generate SQL: {}", e))?;

        let output_path = self.output_file.map(PathBuf::from).unwrap_or_else(|| {
            let input_path = Path::new(&self.input_file);
            input_path
                .parent()
                .map(|dir| dir.join("events.sql"))
                .unwrap_or_else(|| PathBuf::from("events.sql"))
        });

        write(&output_path, sql_statements)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

        println!("SQL statements written to {}", output_path.display());
        Ok(())
    }
}

fn load_tokens_file(path: &str) -> Result<HashMap<Address, u8>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read tokens file: {path}"))?;
    let entries: Vec<TokensFileEntry> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tokens file JSON: {path}"))?;

    let mut map = HashMap::new();
    for entry in entries {
        let addr = Address::from_str(&entry.address).map_err(|err| {
            anyhow!(
                "Invalid token address '{}' in tokens.json: {}",
                entry.address,
                err
            )
        })?;
        map.insert(addr, entry.decimals);
    }

    Ok(map)
}

#[derive(serde::Deserialize)]
struct TokensFileEntry {
    address: String,
    decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_common::raindex_client::local_db::decode::{EventType, UnknownEventDecoded};
    use std::fs;
    use tempfile::TempDir;

    fn sample_unknown_event() -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "0x1".to_string(),
            block_timestamp: "0x2".to_string(),
            transaction_hash: "0xabc".to_string(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x0".to_string(),
                note: "test".to_string(),
            }),
        }
    }

    fn sample_deposit_event() -> DecodedEventData<DecodedEvent> {
        use alloy::primitives::{Address, U256};
        use rain_orderbook_bindings::IOrderBookV5::DepositV2;
        use rain_orderbook_common::raindex_client::local_db::decode::EventType;

        let deposit = DepositV2 {
            sender: Address::from([0x11u8; 20]),
            token: Address::from([0x22u8; 20]),
            vaultId: U256::from(1u64).into(),
            depositAmountUint256: U256::from(1000u64),
        };

        DecodedEventData {
            event_type: EventType::DepositV2,
            block_number: "0x10".to_string(),
            block_timestamp: "0x20".to_string(),
            transaction_hash: "0xdead".to_string(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::DepositV2(Box::new(deposit)),
        }
    }

    #[tokio::test]
    async fn test_execute_with_custom_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("custom_output.sql");

        let events = vec![sample_unknown_event()];
        fs::write(&input_file, serde_json::to_string(&events)?)?;

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

        let events = vec![sample_unknown_event()];
        fs::write(&input_file, serde_json::to_string(&events)?)?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
            end_block: 2000,
            chain_id: 1,
            tokens_file: None,
        };

        cmd.execute().await?;

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

    #[tokio::test]
    async fn test_execute_requires_tokens_file_for_deposits() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("deposits.json");

        let events = vec![sample_deposit_event()];
        fs::write(&input_file, serde_json::to_string(&events).unwrap()).unwrap();

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
            end_block: 1000,
            chain_id: 1,
            tokens_file: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_tokens_file_for_deposits() -> Result<()> {
        use alloy::primitives::Address;

        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("deposits.json");
        let events = vec![sample_deposit_event()];
        fs::write(&input_file, serde_json::to_string(&events)?)?;

        let token_addr = Address::from([0x22u8; 20]);
        let tokens_path = temp_dir.path().join("tokens.json");
        let tokens_json = format!("[{{\"address\":\"0x{:x}\",\"decimals\":18}}]", token_addr);
        fs::write(&tokens_path, tokens_json)?;

        let output_file = temp_dir.path().join("events.sql");

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
            end_block: 1000,
            chain_id: 1,
            tokens_file: Some(tokens_path.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file)?;
        assert!(output_content.contains("INSERT INTO deposits"));

        Ok(())
    }
}
