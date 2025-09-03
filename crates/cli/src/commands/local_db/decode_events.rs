use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::LocalDb;
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
        let events: Vec<serde_json::Value> = serde_json::from_reader(reader)?;

        println!("Processing {} events...", events.len());

        let events_value = serde_json::Value::Array(events);

        let decoded_result = LocalDb::default()
            .decode_events(events_value)
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
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_with_custom_output_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input_file = temp_dir.path().join("input.json");
        let output_file = temp_dir.path().join("custom_output.json");

        let test_events = vec![json!({"type": "test_event", "data": {"value": 123}})];

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

        let test_events = vec![json!({"type": "test_event", "data": {"value": 456}})];

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

        let empty_events: Vec<serde_json::Value> = vec![];
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

        let test_events = vec![
            json!({"type": "event1", "data": "test1"}),
            json!({"type": "event2", "data": "test2"}),
            json!({"type": "event3", "data": {"nested": true}}),
        ];

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
