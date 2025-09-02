use anyhow::{Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::LocalDb;
use std::fs::{write, File};
use std::io::BufReader;

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

        let data: serde_json::Value =
            serde_json::from_reader(reader).context("Failed to parse JSON")?;

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
        let output_file = temp_dir.path().join("custom_output.sql");

        let test_data = json!([
            {"type": "test_event", "data": {"value": 123}}
        ]);

        fs::write(&input_file, serde_json::to_string(&test_data)?)?;

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

        let test_data = json!([
            {"type": "test_event", "data": {"value": 456}}
        ]);

        fs::write(&input_file, serde_json::to_string(&test_data)?)?;

        let cmd = DecodedEventsToSql {
            input_file: input_file.to_string_lossy().to_string(),
            output_file: None,
            end_block: 2000,
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
