use super::meta_getter::get_authoring_meta;
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

mod setup;
pub use setup::get_expression_deployer;

pub async fn touch_deployer(
    wallet: Option<Wallet<SigningKey>>,
) -> Result<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = Some(wallet.unwrap_or(get_wallet(0)));

    let rainterpreter = rainterpreter_deploy(wallet.clone()).await?;

    let store = rainterpreter_store_deploy(wallet.clone()).await?;

    let expression_deployer =
        rainterpreter_expression_deployer_deploy(rainterpreter.address(), store.address(), None)
            .await
            .expect("failed at expression_deployer_deploy");

    Ok(expression_deployer)
}

pub async fn rainterpreter_deploy(
    wallet: Option<Wallet<SigningKey>>,
) -> Result<Rainterpreter<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let interpreter = Rainterpreter::deploy(deployer, ())?.send().await?;

    Ok(interpreter)
}

pub async fn rainterpreter_store_deploy(
    wallet: Option<Wallet<SigningKey>>,
) -> Result<RainterpreterStore<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

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
    wallet: Option<Wallet<SigningKey>>,
) -> Result<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let wallet = wallet.unwrap_or(get_wallet(0));
    let provider = get_provider().await.expect("cannot get provider");
    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let meta_bytes = get_authoring_meta().await.to_vec();
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

    let contract = deploy_transaction
        .deploy_tokens(args)
        .expect("failed deploy tokens")
        .send()
        .await
        .expect("failed at deployment");

    let deployer = RainterpreterExpressionDeployer::new(contract.address(), client);

    return Ok(deployer);
}
