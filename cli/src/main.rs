pub(crate) mod cli;
pub(crate) mod meta;
pub(crate) mod solc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::main()
}