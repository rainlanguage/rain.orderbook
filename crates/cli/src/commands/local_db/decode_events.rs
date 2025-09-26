use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::{
    hyper_rpc::LogEntryResponse, raindex_client::sqlite_web::decode::decode_events,
};
use std::fs::File;
use std::io::{BufReader, Write};

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

        let file = File::open(&self.input_file)?;
        let reader = BufReader::new(file);
        let events: Vec<LogEntryResponse> = serde_json::from_reader(reader)?;

        println!("Processing {} events...", events.len());

        let decoded_result = decode_events(&events)
            .map_err(|e| anyhow::anyhow!("Failed to decode events: {}", e))?;

        let output_filename = self
            .output_file
            .unwrap_or_else(|| "decoded_events.json".to_string());

        let mut file = File::create(&output_filename)?;
        file.write_all(serde_json::to_string_pretty(&decoded_result)?.as_bytes())?;

        println!("Decoded events saved to: {}", output_filename);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_common::hyper_rpc::LogEntryResponse;
    use std::fs;
    use tempfile::TempDir;

    fn sample_event(index: u32) -> LogEntryResponse {
        let hex_index = format!("0x{:x}", index + 1);
        LogEntryResponse {
            address: "0x0000000000000000000000000000000000000000".to_string(),
            topics: vec![format!("0x{:064x}", index + 1)],
            data: format!("0x{:064x}", index + 42),
            block_number: hex_index.clone(),
            block_timestamp: Some(hex_index.clone()),
            transaction_hash: format!("0x{:064x}", index + 100),
            transaction_index: "0x0".to_string(),
            block_hash: format!("0x{:064x}", index + 200),
            log_index: "0x0".to_string(),
            removed: false,
        }
    }

    #[tokio::test]
    async fn test_execute_with_custom_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("custom_output.json");

        let test_events = vec![sample_event(0)];

        fs::write(&input_file, serde_json::to_string(&test_events)?)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file)?;
        let parsed_output: serde_json::Value = serde_json::from_str(&output_content)?;

        assert!(parsed_output.is_object() || parsed_output.is_array());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_default_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let expected_output = temp_dir.path().join("decoded_events.json");

        let test_events = vec![sample_event(1)];

        fs::write(&input_file, serde_json::to_string(&test_events)?)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
        };

        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&temp_dir)?;

        let result = cmd.execute().await;

        std::env::set_current_dir(original_dir)?;

        result?;

        assert!(expected_output.exists());
        let output_content = fs::read_to_string(&expected_output)?;
        let parsed_output: serde_json::Value = serde_json::from_str(&output_content)?;

        assert!(parsed_output.is_object() || parsed_output.is_array());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_empty_events() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("empty_input.json");
        let output_file = temp_dir.path().join("empty_output.json");

        let empty_events: Vec<LogEntryResponse> = vec![];
        fs::write(&input_file, serde_json::to_string(&empty_events)?)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file)?;
        let parsed_output: serde_json::Value = serde_json::from_str(&output_content)?;

        assert!(parsed_output.is_object() || parsed_output.is_array());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_multiple_events() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("multi_input.json");
        let output_file = temp_dir.path().join("multi_output.json");

        let test_events = vec![sample_event(1), sample_event(2), sample_event(3)];

        fs::write(&input_file, serde_json::to_string(&test_events)?)?;

        let cmd = DecodeEvents {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: Some(output_file.to_string_lossy().to_string()),
        };

        cmd.execute().await?;

        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file)?;
        let parsed_output: serde_json::Value = serde_json::from_str(&output_content)?;

        assert!(parsed_output.is_object() || parsed_output.is_array());

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

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No such file") || error_msg.contains("not found"));
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
