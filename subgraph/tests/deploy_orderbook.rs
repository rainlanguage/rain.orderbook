use utils::{
    deploy::{deploy1820::deploy1820, deploy_orderbook::deploy_orderbook},
    utils::deploy_anvil_and_docker,
};

mod utils;
#[tokio::main]
#[test]
async fn orderbook_entity_test() -> anyhow::Result<()> {
    let anvil = deploy_anvil_and_docker()?;
    deploy1820(&anvil).await?;
    deploy_orderbook(&anvil).await?;

    Ok(())
}
