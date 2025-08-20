use anyhow::{Context, Result};
use clap::Args;
use rain_orderbook_common::raindex_client::local_db::insert::decoded_events_to_sql;
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct EventsToSql {
    #[arg(
        short,
        long,
        default_value = "src/commands/local_db/decoded_events.json",
        help = "Path to the decoded events JSON file"
    )]
    pub input: PathBuf,

    #[arg(
        short,
        long,
        default_value = "src/commands/local_db/events.sql",
        help = "Path to output SQL file"
    )]
    pub output: PathBuf,
}

impl EventsToSql {
    pub async fn execute(self) -> Result<()> {
        println!("Reading decoded events from: {:?}", self.input);

        let content = fs::read_to_string(&self.input)
            .with_context(|| format!("Failed to read input file: {:?}", self.input))?;

        let data: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse JSON")?;

        println!("Generating SQL statements...");

        // Call the common insert function
        let sql_statements = decoded_events_to_sql(data)
            .map_err(|e| anyhow::anyhow!("Failed to generate SQL: {}", e))?;

        fs::write(&self.output, sql_statements)
            .with_context(|| format!("Failed to write output file: {:?}", self.output))?;

        println!("SQL statements written to {:?}", self.output);

        Ok(())
    }
}
