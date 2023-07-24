use serde_json::Value;
use std::fs::File;
use std::io::Read;
use web3::ethabi::Contract as C;
use web3::{contract::Contract, transports::Http};

use super::touch_deployer::deploy_touch_deployer;

pub async fn deploy_orderbook() -> anyhow::Result<()> {
    let touch_deployer = deploy_touch_deployer().await?;
    // let mut json = String::new();
    // let mut file = File::open("tests/utils/deploy/deploy_orderbook/OrderBook.json")?;
    // file.read_to_string(&mut json)?;

    // let json: Value = serde_json::from_str(&json)?;

    Ok(())
}
