pub mod cli;
pub mod dump;
pub mod executor;
pub mod pipeline;
pub mod sync;

use anyhow::Result;
use clap::Subcommand;
use cli::RunPipeline;
use dump::DbDump;
use sync::SyncLocalDb;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDbCommands {
    #[command(name = "dump")]
    Dump(DbDump),
    #[command(name = "sync")]
    Sync(SyncLocalDb),
    #[command(name = "run-pipeline")]
    RunPipeline(RunPipeline),
}

impl LocalDbCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDbCommands::Dump(dump) => dump.execute().await,
            LocalDbCommands::Sync(cmd) => cmd.execute().await,
            LocalDbCommands::RunPipeline(cmd) => cmd.execute().await,
        }
    }
}
