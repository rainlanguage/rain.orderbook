use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::hyper_rpc::HyperRpcError;
use rain_orderbook_common::raindex_client::sqlite_web::{SqliteWeb, SqliteWebError};
use std::fs::File;
use std::io::Write;

#[async_trait::async_trait]
pub trait EventClient {
    async fn get_latest_block_number(&self) -> Result<u64, HyperRpcError>;
    async fn fetch_events(
        &self,
        address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<serde_json::Value, SqliteWebError>;
}

#[async_trait::async_trait]
impl EventClient for SqliteWeb {
    async fn get_latest_block_number(&self) -> Result<u64, HyperRpcError> {
        self.hyper_rpc_client().get_latest_block_number().await
    }

    async fn fetch_events(
        &self,
        address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<serde_json::Value, SqliteWebError> {
        self.fetch_events(address, start_block, end_block).await
    }
}

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
    pub async fn execute_with_client<C: EventClient>(self, client: C) -> Result<()> {
        println!("Starting event fetch...");

        let end_block = if let Some(end_block) = self.end_block {
            end_block
        } else {
            client
                .get_latest_block_number()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get latest block number: {}", e))?
        };

        let all_events = client
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

    pub async fn execute(self) -> Result<()> {
        let local_db = SqliteWeb::new(self.chain_id, self.api_token.clone())?;
        self.execute_with_client(local_db).await
    }
}
