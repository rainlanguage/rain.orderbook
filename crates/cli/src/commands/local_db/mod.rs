pub mod cli;
pub mod executor;
pub mod functions;
pub mod pipeline;

use anyhow::Result;
use clap::Subcommand;
use cli::RunPipeline;
use import_dump::ImportDump;

mod import_dump;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDbCommands {
    #[command(name = "sync")]
    Sync(RunPipeline),

    #[command(name = "import-dump")]
    ImportDump(ImportDump),
}

impl LocalDbCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDbCommands::Sync(cmd) => cmd.execute().await,
            LocalDbCommands::ImportDump(cmd) => cmd.execute().await,
        }
    }
}
