use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Middleware};
use ethers::types::{Bytes, Eip1559TransactionRequest, H160, U256};
use ethers::{
    prelude::abigen,
    providers::Provider,
    signers::{LocalWallet, Signer},
    utils::AnvilInstance,
};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
abigen!(
    RainterpreterExpressionDeployer,
    "tests/utils/deploy/touch_deployer/RainterpreterExpressionDeployer.json",
    derives(serde::Deserialize, serde::Serialize);

    Rainterpreter,
    "tests/utils/deploy/touch_deployer/Rainterpreter.json";

    RainterpreterStore,
    "tests/utils/deploy/touch_deployer/RainterpreterStore.json"
);
pub async fn deploy_touch_deployer(anvil: &AnvilInstance) -> anyhow::Result<H160> {
    let provider =
        Provider::<Http>::try_from(&anvil.endpoint())?.interval(Duration::from_millis(10u64));

    rainterpreter_deploy(&provider, &anvil).await?;
    rainterpreter_store_deploy(&provider, &anvil).await?;
    let expression_deployer = rainterpreter_expression_deployer_deploy(&provider, anvil).await?;
    Ok(expression_deployer)
}

pub async fn rainterpreter_deploy(
    provider: &Provider<Http>,
    anvil: &AnvilInstance,
) -> anyhow::Result<H160> {
    let deployer: LocalWallet = anvil.keys()[0].clone().into();

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        deployer.with_chain_id(anvil.chain_id()),
    ));
    let store = Rainterpreter::deploy(deployer, ())?.send().await?;
    Ok(store.address())
}

pub async fn rainterpreter_store_deploy(
    provider: &Provider<Http>,
    anvil: &AnvilInstance,
) -> anyhow::Result<H160> {
    let deployer: LocalWallet = anvil.keys()[0].clone().into();

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        deployer.with_chain_id(anvil.chain_id()),
    ));
    let store = RainterpreterStore::deploy(deployer, ())?.send().await?;
    Ok(store.address())
}

pub async fn rainterpreter_expression_deployer_deploy(
    provider: &Provider<Http>,
    anvil: &AnvilInstance,
) -> anyhow::Result<H160> {
    let deployer: LocalWallet = anvil.keys()[0].clone().into();

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        deployer.with_chain_id(anvil.chain_id()),
    ));
    let mut data = String::new();
    let mut file = File::open("tests/utils/deploy/touch_deployer/data.json")?;
    file.read_to_string(&mut data)?;

    let data: Value = serde_json::from_str(&data)?;
    let data = data["data"].as_str().unwrap();

    let mut tx = Eip1559TransactionRequest::new();
    tx.to = Some(H160::zero().into());
    tx.value = Some(U256::zero());
    tx.max_fee_per_gas = Some(U256::from(50_000_000_000u128));
    tx.max_priority_fee_per_gas = Some(U256::from(20_000_000_000u128));
    tx.data = Some(Bytes::from_str(data)?);
    tx.chain_id = Some(provider.get_chainid().await.unwrap().as_u64().into());
    tx.gas = Some(U256::from(21000));

    let tx_receipt = deployer.send_transaction(tx, None).await?.await?.unwrap();

    let contract =
        RainterpreterExpressionDeployer::new(tx_receipt.contract_address.unwrap(), deployer);

    println!("{:?}", contract.interpreter().await?);

    Ok(contract.address())
}
