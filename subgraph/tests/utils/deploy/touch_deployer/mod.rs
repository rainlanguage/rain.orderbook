use crate::{
    generated::{
        Rainterpreter, RainterpreterExpressionDeployer, RainterpreterStore,
        RAINTERPRETEREXPRESSIONDEPLOYER_ABI, RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE,
    },
    utils::{get_provider, get_wallet},
};
use anyhow::Result;
use ethers::{
    abi::Token,
    contract::ContractFactory,
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
    types::H160,
};
use std::sync::Arc;

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
    let wallet = get_wallet(0)?;
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let interpreter = Rainterpreter::deploy(deployer, ())?.send().await?;

    Ok(interpreter)
}

pub async fn rainterpreter_store_deploy(
) -> Result<RainterpreterStore<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = get_wallet(0)?;
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let store = RainterpreterStore::deploy(deployer, ())?.send().await?;

    Ok(store)
}

pub async fn rainterpreter_expression_deployer_deploy(
    rainiterpreter_address: H160,
    store_address: H160,
) -> Result<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = get_wallet(0)?;
    let provider = get_provider().await?;
    let chain_id = provider.get_chainid().await?;

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let meta_bytes =
        std::fs::read("./tests/generated/RainterpreterExpressionDeployerNP.rain.meta")?;

    let args = vec![Token::Tuple(vec![
        Token::Address(rainiterpreter_address),
        Token::Address(store_address),
        Token::Bytes(meta_bytes),
    ])];

    let deploy_transaction = ContractFactory::new(
        RAINTERPRETEREXPRESSIONDEPLOYER_ABI.clone(),
        RAINTERPRETEREXPRESSIONDEPLOYER_BYTECODE.clone(),
        client.clone(),
    );

    let contract = deploy_transaction.deploy_tokens(args)?.send().await?;

    let deployer = RainterpreterExpressionDeployer::new(contract.address(), client);

    return Ok(deployer);
}
