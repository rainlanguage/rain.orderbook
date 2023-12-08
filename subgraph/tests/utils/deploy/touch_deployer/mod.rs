use crate::generated::{
    Rainterpreter, RainterpreterExpressionDeployer, RainterpreterStore,
    RAINTERPRETEREXPRESSIONDEPLOYER_ABI, RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE,
};
use crate::utils::deploy::get_authoring_meta;
use crate::utils::setup::get_wallets_handler;
use anyhow::Result;
use ethers::{
    abi::Token,
    contract::ContractFactory,
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::H160,
};

pub async fn touch_deployer(
) -> Result<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let rainterpreter = rainterpreter_deploy().await?;
    let store = rainterpreter_store_deploy().await?;
    let expression_deployer =
        rainterpreter_expression_deployer_deploy(rainterpreter.address(), store.address()).await?;

    Ok(expression_deployer)
}

pub async fn rainterpreter_deploy(
) -> Result<Rainterpreter<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let deployer = get_wallets_handler().get_client(0).await?;
    Ok(Rainterpreter::deploy(deployer, ())?.send().await?)
}

pub async fn rainterpreter_store_deploy(
) -> Result<RainterpreterStore<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let deployer = get_wallets_handler().get_client(0).await?;
    Ok(RainterpreterStore::deploy(deployer, ())?.send().await?)
}

pub async fn rainterpreter_expression_deployer_deploy(
    rainiterpreter_address: H160,
    store_address: H160,
) -> Result<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let meta_bytes = get_authoring_meta().await?.to_vec();
    let args = vec![Token::Tuple(vec![
        Token::Address(rainiterpreter_address),
        Token::Address(store_address),
        Token::Bytes(meta_bytes),
    ])];

    let deployer = get_wallets_handler().get_client(0).await?;

    let deploy_transaction = ContractFactory::new(
        RAINTERPRETEREXPRESSIONDEPLOYER_ABI.clone(),
        RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE.clone(),
        deployer.clone(),
    );
    let contract = deploy_transaction.deploy_tokens(args)?.send().await?;

    Ok(RainterpreterExpressionDeployer::new(
        contract.address(),
        deployer,
    ))
}
