use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::decode::decode_events;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
pub struct DecodeEvents {
    #[clap(
        short,
        long,
        default_value = "src/commands/local_db/fetch_events_results.json"
    )]
    pub input_file: String,

    #[clap(
        short,
        long,
        default_value = "src/commands/local_db/decoded_events.json"
    )]
    pub output_file: String,
}

impl DecodeEvents {
    pub async fn execute(self) -> Result<()> {
        println!("Reading events from: {}", self.input_file);

        // Read the input file
        let file_content = std::fs::read_to_string(&self.input_file)?;
        let json_data: serde_json::Value = serde_json::from_str(&file_content)?;

        // Extract events array from the input file
        let events = json_data
            .get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow::anyhow!("No events found in input file"))?;

        println!("Processing {} events...", events.len());

        // Convert the events array to the format expected by the common decode function
        let events_value = serde_json::Value::Array(events.clone());

        // Call the common decode function
        let decoded_result = decode_events(events_value)
            .map_err(|e| anyhow::anyhow!("Failed to decode events: {}", e))?;

        // Add CLI-specific metadata to the result
        let mut output_data = decoded_result;
        if let Some(metadata) = output_data.get_mut("metadata") {
            if let Some(metadata_obj) = metadata.as_object_mut() {
                metadata_obj.insert(
                    "source_file".to_string(),
                    serde_json::Value::String(self.input_file.clone()),
                );
                metadata_obj.insert(
                    "timestamp".to_string(),
                    serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
                );
            }
        }

        // Write to output file
        let mut file = File::create(&self.output_file)?;
        file.write_all(serde_json::to_string_pretty(&output_data)?.as_bytes())?;

        // Extract statistics for CLI output
        let total_processed = output_data
            .get("metadata")
            .and_then(|m| m.get("total_events_processed"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let total_decoded = output_data
            .get("metadata")
            .and_then(|m| m.get("total_events_decoded"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let decode_stats = output_data
            .get("metadata")
            .and_then(|m| m.get("decode_statistics"))
            .and_then(|v| v.as_object());

        println!("\n=== DECODE SUMMARY ===");
        println!("Total events processed: {}", total_processed);
        println!("Total events decoded: {}", total_decoded);

        if let Some(stats) = decode_stats {
            println!("Decode statistics:");
            for (event_type, count) in stats {
                println!("  {}: {}", event_type, count);
            }
        }

        println!("Decoded events saved to: {}", self.output_file);

        Ok(())
    }
}
