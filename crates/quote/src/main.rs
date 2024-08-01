#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rain_orderbook_quote::cli::main().await
}

#[cfg(target_family = "wasm")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    rain_orderbook_quote::cli::main().await
}
