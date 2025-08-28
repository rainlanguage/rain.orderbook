use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_client::sqlite_web::decode::decode_events;
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

        let decoded_result = decode_events(events_value)
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
