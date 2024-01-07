use clap::Parser;
use anyhow::Result;

mod orderbook;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    orderbook: orderbook::Orderbook,
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let cli = Cli::parse();
    orderbook::dispatch(cli.orderbook).await
}