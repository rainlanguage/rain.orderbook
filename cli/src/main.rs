mod cli;
pub(crate) mod subgraph;
pub(crate) mod tokens;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::main().await
}