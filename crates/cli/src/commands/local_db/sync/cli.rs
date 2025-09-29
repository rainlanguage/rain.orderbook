use anyhow::Result;
use clap::Parser;

use super::{
    config::load_primary_orderbook_from_commit,
    data_source::DefaultTokenFetcher,
    runner::{SyncParams, SyncRunner},
    storage::build_local_db_from_network,
};

#[derive(Debug, Clone, Parser)]
#[command(about = "Incrementally sync a local SQLite database using on-chain events")]
pub struct SyncLocalDb {
    #[clap(long, help = "Path to SQLite DB that stores indexed data")]
    pub db_path: String,

    #[clap(long, help = "Chain ID for the orderbook deployment")]
    pub chain_id: u32,

    #[clap(
        long,
        help = "Git commit hash of the rain.orderbook repository used to resolve remote settings"
    )]
    pub repo_commit: String,

    #[clap(long, help = "Optional override for start block")]
    pub start_block: Option<u64>,

    #[clap(long, help = "Optional override for end block")]
    pub end_block: Option<u64>,
}

impl SyncLocalDb {
    pub async fn execute(self) -> Result<()> {
        println!("Starting local DB sync");

        let SyncLocalDb {
            db_path,
            chain_id,
            repo_commit,
            start_block,
            end_block,
        } = self;

        let primary_orderbook = load_primary_orderbook_from_commit(chain_id, &repo_commit).await?;
        let orderbook_address = format!("{:#x}", primary_orderbook.address);
        let deployment_block = primary_orderbook.deployment_block;

        if let Some(label) = &primary_orderbook.label {
            println!(
                "Using orderbook {} ({}) resolved from repo commit {}",
                orderbook_address, label, repo_commit
            );
        } else {
            println!(
                "Using orderbook {} resolved from repo commit {}",
                orderbook_address, repo_commit
            );
        }

        let (local_db, metadata_rpc_urls) =
            build_local_db_from_network(chain_id, primary_orderbook.network.as_ref())?;
        let token_fetcher = DefaultTokenFetcher;
        let runner = SyncRunner::new(&db_path, &local_db, metadata_rpc_urls, &token_fetcher);
        let params = SyncParams {
            chain_id,
            orderbook_address: &orderbook_address,
            deployment_block,
            start_block,
            end_block,
        };

        runner.run(&params).await
    }
}
