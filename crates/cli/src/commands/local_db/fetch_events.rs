use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_client::sqlite_web::SqliteWeb;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
#[command(about = "Fetch events from blockchain and save to JSON file")]
pub struct FetchEvents {
    #[clap(long)]
    pub api_token: String,
    #[clap(long)]
    pub chain_id: u32,
    #[clap(long)]
    pub start_block: u64,
    #[clap(long)]
    pub end_block: Option<u64>,
    #[clap(long)]
    pub orderbook_address: String,
    #[clap(long)]
    pub output_file: Option<String>,
}

impl FetchEvents {
    pub async fn execute(self) -> Result<()> {
        println!("Starting event fetch...");

        let local_db = SqliteWeb::new(self.chain_id, self.api_token)?;

        let end_block = if let Some(end_block) = self.end_block {
            end_block
        } else {
            local_db
                .hyper_rpc_client()
                .get_latest_block_number()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get latest block number: {}", e))?
        };

        let all_events = local_db
            .fetch_events(&self.orderbook_address, self.start_block, end_block)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch events: {}", e))?;

        let output_filename = self
            .output_file
            .unwrap_or_else(|| format!("src/commands/local_db/events_{}.json", end_block));
        let mut file = File::create(&output_filename)?;
        file.write_all(serde_json::to_string_pretty(&all_events)?.as_bytes())?;

        println!("Events and results saved to: {}", output_filename);
        Ok(())
    }
}
