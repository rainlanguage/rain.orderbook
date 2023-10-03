use ethers::utils::AnvilInstance;

use super::touch_deployer::deploy_touch_deployer;

pub async fn deploy_orderbook(anvil: &AnvilInstance) -> anyhow::Result<()> {
    let _touch_deployer = deploy_touch_deployer(anvil).await?;
    // let mut json = String::new();
    // let mut file = File::open("tests/utils/deploy/deploy_orderbook/OrderBook.json")?;
    // file.read_to_string(&mut json)?;

    // let json: Value = serde_json::from_str(&json)?;

    Ok(())
}
