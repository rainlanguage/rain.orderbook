mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::main().await
}
