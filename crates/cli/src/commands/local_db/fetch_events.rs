use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_client::local_db::fetch::{fetch_events, HyperRpcClient};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
pub struct FetchEvents {
    #[clap(short, long)]
    pub output_file: Option<String>,
}

impl FetchEvents {
    pub async fn execute(self) -> Result<()> {
        println!("Starting event fetch...");

        let total_start = std::time::Instant::now();

        let client = HyperRpcClient {};
        let start_block = 19033330u64;
        let end_block = client
            .get_latest_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get latest block number: {}", e))?;

        // let start_block = 19036351u64;
        // let end_block = 34658179u64;

        let all_events = fetch_events(
            "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
            start_block,
            end_block,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch events: {}", e))?;

        let total_time = total_start.elapsed();

        // Extract events from the result
        let events_array = match all_events.as_array() {
            Some(events) => events,
            None => return Err(anyhow::anyhow!("Expected events array from fetch_events")),
        };

        let total_events = events_array.len();

        // Calculate some basic statistics for CLI output
        let total_size_bytes = serde_json::to_string(&all_events)?.len();

        // Create metadata for output file
        let output_data = serde_json::json!({
            "metadata": {
                "test_config": {
                    "start_block": start_block,
                    "contract_address": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                    "note": "Using common fetch logic from rain_orderbook_common"
                },
                "results": {
                    "total_events_found": total_events,
                    "total_time_seconds": total_time.as_secs_f64(),
                    "total_size_bytes": total_size_bytes,
                    "total_size_mb": total_size_bytes as f64 / 1_048_576.0,
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "events": all_events
        });

        println!("\n=== SUMMARY ===");
        println!("Total events found: {}", total_events);
        println!("Total time: {:.2}s", total_time.as_secs_f64());
        println!(
            "Total size: {} bytes ({:.2} MB)",
            total_size_bytes,
            total_size_bytes as f64 / 1_048_576.0
        );

        // Save results to file
        let output_filename = self
            .output_file
            .unwrap_or_else(|| format!("events_{}.json", end_block));
        let mut file = File::create(&output_filename)?;
        file.write_all(serde_json::to_string_pretty(&output_data)?.as_bytes())?;
        println!("Events and results saved to: {}", output_filename);

        Ok(())
    }
}
