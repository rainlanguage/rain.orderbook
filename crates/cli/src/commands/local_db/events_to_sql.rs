use anyhow::{Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::insert::decoded_events_to_sql;
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct EventsToSql {
    #[arg(short, long, help = "Path to the decoded events JSON file")]
    pub input: PathBuf,

    #[arg(short, long, help = "Path to output SQL file")]
    pub output: Option<PathBuf>,
}

impl EventsToSql {
    pub async fn execute(self) -> Result<()> {
        println!("Reading decoded events from: {:?}", self.input);

        let content = fs::read_to_string(&self.input)
            .with_context(|| format!("Failed to read input file: {:?}", self.input))?;

        let data: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse JSON")?;

        println!("Generating SQL statements...");

        // Extract block number from input filename for both SQL function and output filename
        let input_filename = self
            .input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("decoded_events");

        let (block_num, block_num_u64) = if let Some(block_str) = input_filename
            .strip_prefix("decoded_events_")
            .and_then(|s| s.strip_suffix(".json"))
        {
            let block_u64 = block_str.parse::<u64>().unwrap_or(0);
            (Some(block_str), block_u64)
        } else {
            (None, 0)
        };

        // Call the common insert function
        let sql_statements = decoded_events_to_sql(data, block_num_u64)
            .map_err(|e| anyhow::anyhow!("Failed to generate SQL: {}", e))?;

        // Determine output filename
        let output_path = self.output.unwrap_or_else(|| {
            if let Some(block_num) = block_num {
                PathBuf::from(format!("events_{}.sql", block_num))
            } else {
                PathBuf::from("events.sql")
            }
        });

        fs::write(&output_path, sql_statements)
            .with_context(|| format!("Failed to write output file: {:?}", output_path))?;

        println!("SQL statements written to {:?}", output_path);

        Ok(())
    }
}
