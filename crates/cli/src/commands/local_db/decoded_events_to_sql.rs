use anyhow::{Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::{
    decode::{DecodedEvent, DecodedEventData},
    insert::decoded_events_to_sql,
};
use std::fs::{write, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct DecodedEventsToSql {
    #[arg(long)]
    pub input_file: String,

    #[arg(long)]
    pub output_file: Option<String>,

    #[arg(long)]
    pub end_block: u64,
}

impl DecodedEventsToSql {
    pub async fn execute(self) -> Result<()> {
        println!("Generating SQL statements...");

        let file = File::open(&self.input_file)
            .with_context(|| format!("Failed to open input file: {:?}", self.input_file))?;
        let reader = BufReader::new(file);

        let decoded_events: Vec<DecodedEventData<DecodedEvent>> =
            serde_json::from_reader(reader).context("Failed to parse decoded events JSON")?;

        let sql_statements = decoded_events_to_sql(&decoded_events, self.end_block)
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
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        Ok(())
    }
}
