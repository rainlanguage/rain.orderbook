use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::{
    decode::{DecodedEvent, DecodedEventData},
    insert::decoded_events_to_sql,
};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
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
        let mut deposit_tokens: BTreeSet<Address> = BTreeSet::new();
        for event in &decoded_events {
            if let DecodedEvent::DepositV2(deposit) = &event.decoded_data {
                deposit_tokens.insert(deposit.token);
            }
        }

        let mut decimals_by_token: HashMap<Address, u8> = HashMap::new();

        if !deposit_tokens.is_empty() {
            let tokens_path = self.tokens_file.as_ref().ok_or_else(|| {
                anyhow!("--tokens-file is required to compute deposit amounts for deposit events")
            })?;

            let content =
                std::fs::read_to_string(tokens_path).context("Failed to read tokens file")?;
            let tokens_in: Vec<TokensFileEntry> =
                serde_json::from_str(&content).context("Failed to parse tokens file")?;

            for entry in tokens_in {
                let addr = Address::from_str(&entry.address).map_err(|e| {
                    anyhow!(
                        "Invalid token address '{}' in tokens file: {}",
                        entry.address,
                        e
                    )
                })?;
                decimals_by_token.insert(addr, entry.decimals);
            }

            for token in &deposit_tokens {
                if !decimals_by_token.contains_key(token) {
                    return Err(anyhow!(
                        "Missing decimals for token 0x{:x} in tokens file",
                        token
                    ));
                }
            }
        }

        let sql_statements =
            decoded_events_to_sql(&decoded_events, self.end_block, &decimals_by_token, None)
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

#[derive(Deserialize)]
struct TokensFileEntry {
    address: String,
    decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address as AlloyAddress, B256, U256};
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;
    use rain_orderbook_common::raindex_client::local_db::decode::{EventType, UnknownEventDecoded};
    use std::fs;
    use std::str::FromStr;
    use tempfile::TempDir;

    fn sample_unknown_event() -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: EventType::Unknown,
            block_number: 0x1,
            block_timestamp: 0x2,
            transaction_hash: B256::from([0xab; 32]),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x0".to_string(),
                note: "test".to_string(),
            }),
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_execute_computes_deposit_amount_without_token_upsert() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("decoded.json");
        let output_file = temp_dir.path().join("events.sql");

        let token_addr =
            AlloyAddress::from_str("0x00000000000000000000000000000000000000aa").unwrap();

        // Build a decoded DepositV2 event requiring decimals
        let decoded = vec![DecodedEventData {
            event_type: EventType::DepositV2,
            block_number: 0x3e8,
            block_timestamp: 0x64b8c123,
            transaction_hash: B256::from([0x11; 32]),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                sender: AlloyAddress::from_str("0x0000000000000000000000000000000000000001")
                    .unwrap(),
                token: token_addr,
                vaultId: U256::from(1).into(),
                depositAmountUint256: U256::from_str("1000000000000000000").unwrap(),
            })),
        }];
        std::fs::write(&input_file, serde_json::to_string(&decoded)?)?;

        // Build tokens.json with decimals for the token used in decoded events
        let tokens_path = temp_dir.path().join("tokens.json");
        let tokens_json = format!(
            "[{{\"address\":\"0x{:x}\",\"name\":\"Token\",\"symbol\":\"TKN\",\"decimals\":18}}]",
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
