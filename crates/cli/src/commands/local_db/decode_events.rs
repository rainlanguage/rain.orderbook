use anyhow::{Context, Result};
use clap::Parser;
use rain_orderbook_common::{
    raindex_client::local_db::decode::decode_events, rpc_client::LogEntryResponse,
};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Parser)]
#[command(about = "Decode events from a JSON file and save the results")]
pub struct DecodeEvents {
    #[clap(long)]
    pub input_file: String,
    #[clap(long)]
    pub output_file: Option<String>,
}

impl DecodeEvents {
    pub async fn execute(self) -> Result<()> {
        println!("Reading events from: {}", self.input_file);

        let file = File::open(&self.input_file)
            .with_context(|| format!("Failed to open {}", self.input_file))?;
        let reader = BufReader::new(file);
        let events: Vec<LogEntryResponse> = serde_json::from_reader(reader)
            .with_context(|| format!("Failed to parse {} as log entries", self.input_file))?;

        println!("Processing {} events...", events.len());

        let decoded_result = decode_events(&events)
            .map_err(|e| anyhow::anyhow!("Failed to decode events: {}", e))?;

        let output_filename = self.output_file.map(PathBuf::from).unwrap_or_else(|| {
            let input_path = Path::new(&self.input_file);
            input_path
                .parent()
                .map(|dir| dir.join("decoded_events.json"))
                .unwrap_or_else(|| PathBuf::from("decoded_events.json"))
        });

        let mut file = File::create(&output_filename)
            .with_context(|| format!("Failed to create {}", output_filename.display()))?;
        serde_json::to_writer_pretty(&mut file, &decoded_result).with_context(|| {
            format!(
                "Failed to write decoded events to {}",
                output_filename.display()
            )
        })?;
        writeln!(file)?;

        println!("Decoded events saved to: {}", output_filename.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{Address, Bytes, FixedBytes, U256},
        sol_types::SolEvent,
    };
    use rain_orderbook_bindings::IOrderBookV5::{AddOrderV3, EvaluableV4, OrderV4, IOV2};
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn sample_order_v4() -> OrderV4 {
        OrderV4 {
            owner: Address::from([1u8; 20]),
            nonce: U256::from(1).into(),
            evaluable: EvaluableV4 {
                interpreter: Address::from([2u8; 20]),
                store: Address::from([3u8; 20]),
                bytecode: Bytes::from(vec![0x01, 0x02, 0x03, 0x04]),
            },
            validInputs: vec![
                IOV2 {
                    token: Address::from([4u8; 20]),
                    vaultId: U256::from(100).into(),
                },
                IOV2 {
                    token: Address::from([5u8; 20]),
                    vaultId: U256::from(200).into(),
                },
            ],
            validOutputs: vec![IOV2 {
                token: Address::from([6u8; 20]),
                vaultId: U256::from(300).into(),
            }],
        }
    }

    fn add_order_event(sender_byte: u8, order_nonce: u64, log_index: u64) -> LogEntryResponse {
        let sender = Address::from([sender_byte; 20]);
        let order_hash = FixedBytes::<32>::from([sender_byte + 1; 32]);
        let mut order = sample_order_v4();
        order.nonce = U256::from(order_nonce).into();

        let event = AddOrderV3 {
            sender,
            orderHash: order_hash,
            order,
        };

        let data = format!("0x{}", hex::encode(event.encode_data()));
        LogEntryResponse {
            address: "0x0000000000000000000000000000000000000000".to_string(),
            topics: vec![format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            data,
            block_number: format!("0x{:x}", log_index + 1),
            block_timestamp: Some(format!("0x{:x}", log_index + 2)),
            transaction_hash: format!("0x{}", hex::encode([sender_byte; 32])),
            transaction_index: "0x0".to_string(),
            block_hash: format!("0x{}", hex::encode([sender_byte + 2; 32])),
            log_index: format!("0x{:x}", log_index),
            removed: false,
        }
    }

    fn write_events(path: &Path, events: &[LogEntryResponse]) -> Result<()> {
        fs::write(path, serde_json::to_string(events)?)?;
        Ok(())
    }

    fn decoded_output(path: &Path) -> serde_json::Value {
        let output_content = fs::read_to_string(path).expect("output to exist");
        serde_json::from_str(&output_content).expect("valid json")
    }

    #[tokio::test]
    async fn test_execute_with_custom_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("custom_output.json");

        let test_events = vec![add_order_event(7, 10, 0)];

        write_events(&input_file, &test_events)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let parsed_output = decoded_output(&output_file);
        assert_eq!(parsed_output.as_array().map(|arr| arr.len()), Some(1));
        let event = &parsed_output[0];
        assert_eq!(event["event_type"], "AddOrderV3");
        assert_eq!(
            event["decoded_data"]["sender"],
            serde_json::Value::String(format!("0x{}", hex::encode([7u8; 20])))
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_default_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let expected_output = temp_dir.path().join("decoded_events.json");

        let test_events = vec![add_order_event(9, 11, 1)];

        write_events(&input_file, &test_events)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
        };

        cmd.execute().await?;

        assert!(expected_output.exists());
        let parsed_output = decoded_output(&expected_output);
        assert_eq!(parsed_output.as_array().map(|arr| arr.len()), Some(1));
        assert_eq!(parsed_output[0]["event_type"], "AddOrderV3");

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_empty_events() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("empty_input.json");
        let output_file = temp_dir.path().join("empty_output.json");

        let empty_events: Vec<LogEntryResponse> = vec![];
        write_events(&input_file, &empty_events)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let parsed_output = decoded_output(&output_file);
        assert_eq!(parsed_output.as_array().map(Vec::len), Some(0));

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_multiple_events() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("multi_input.json");
        let output_file = temp_dir.path().join("multi_output.json");

        let test_events = vec![
            add_order_event(10, 12, 1),
            add_order_event(11, 13, 2),
            add_order_event(12, 14, 3),
        ];

        write_events(&input_file, &test_events)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let parsed_output = decoded_output(&output_file);
        assert_eq!(parsed_output.as_array().map(|arr| arr.len()), Some(3));
        assert!(parsed_output
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_type"] == "AddOrderV3"));

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_nonexistent_input_file() {
        let cmd = DecodeEvents {
            input_file: "/path/that/does/not/exist.json".to_string(),
            output_file: Some("output.json".to_string()),
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("Failed to open"),
            "unexpected error message: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_execute_with_invalid_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("invalid.json");
        let output_file = temp_dir.path().join("output.json");

        fs::write(&input_file, "{ invalid json content")?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_json_not_array() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("not_array.json");
        let output_file = temp_dir.path().join("output.json");

        fs::write(&input_file, r#"{"events": "not an array"}"#)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.is_empty());

        Ok(())
    }
}
