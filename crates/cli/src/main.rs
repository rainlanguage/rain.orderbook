use anyhow::Result;
use clap::Parser;
use rain_orderbook_cli::telegram_notifications::check_balance;
use rain_orderbook_cli::Orderbook;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    orderbook: Orderbook,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Config tracing subscriber output
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()?
        .add_directive("ethers_signer=off".parse()?)
        .add_directive("coins_ledger=off".parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_target(false)
        .without_time()
        .compact()
        .init();

    let balance = 8; // Example balance
    check_balance(balance); // Call the function

    let cli = Cli::parse();
    cli.orderbook.execute().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
