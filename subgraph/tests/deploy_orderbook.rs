use utils::deploy::{deploy1820::deploy1820, deploy_orderbook::deploy_orderbook};

mod utils;
#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    deploy1820().await?;
    deploy_orderbook().await?;
    Ok(())
} 