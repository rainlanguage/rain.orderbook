mod cli;
pub(crate) mod subgraph;
pub(crate) mod tokens;
pub(crate) mod gasoracle;
pub(crate) mod orderbook;
pub(crate) mod rpc;
pub(crate) mod transaction;





#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::main().await
}