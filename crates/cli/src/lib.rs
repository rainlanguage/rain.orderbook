use anyhow::Result;
use clap::Subcommand;
use commands::local_db::LocalDbCommands;

mod commands;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(name = "local-db", subcommand)]
    LocalDb(LocalDbCommands),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::LocalDb(local_db) => local_db.execute().await,
        }
    }
}
