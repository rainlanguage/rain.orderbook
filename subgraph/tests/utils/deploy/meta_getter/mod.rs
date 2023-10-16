use crate::{
    generated::AuthoringMetaGetter,
    utils::{get_provider, get_wallet},
};
use anyhow::Result;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Http, Middleware};
use ethers::types::{Bytes, H160};
use ethers::{providers::Provider, signers::Signer};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::OnceCell;

static META_GETTER: Lazy<OnceCell<H160>> = Lazy::new(|| OnceCell::new());

#[derive(thiserror::Error, Debug)]
pub enum MetaGetterError {
    #[error("An error when deploying MetaGetter")]
    DeployError(#[from] Box<dyn std::error::Error>),
}

async fn meta_getter_init(provider: &Provider<Http>) -> Result<H160, MetaGetterError> {
    // let provider = get_provider().await.expect("cannot get provider");
    let meta_address = authoring_meta_getter_deploy(provider)
        .await
        .expect("cannot deploy in init");

    Ok(meta_address)
}

async fn meta_getter(provider: &Provider<Http>) -> Result<H160, MetaGetterError> {
    // If an error occurs, wrap it using MetaGetterError::DeployError
    match meta_getter_init(provider).await {
        Ok(data) => Ok(data),
        Err(err) => Err(MetaGetterError::DeployError(Box::new(err))),
    }
}

///
pub async fn get_meta_address(provider: &Provider<Http>) -> Result<&'static H160, MetaGetterError> {
    META_GETTER
        .get_or_try_init(|| async { meta_getter(provider).await })
        .await
        .map_err(|e| MetaGetterError::DeployError(Box::new(e)))
}

pub async fn authoring_meta_getter_deploy(provider: &Provider<Http>) -> Result<H160> {
    let wallet = get_wallet(0);

    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let contract = AuthoringMetaGetter::deploy(deployer, ())
        .expect("cannot create the factory AuthoringMetaGetter instance")
        .send()
        .await
        .expect("cannot deploy AuthoringMetaGetter");

    Ok(contract.address())
}

/// Get the AuthoringMeta bytes to deploy ExpressionDeployers.
/// This function only will work after the META_GETTER is being initialized by calling `get_meta_address()`
pub async fn get_authoring_meta() -> Bytes {
    let provider = get_provider().await.expect("cannot get provider");
    let wallet = get_wallet(0);

    let meta_address: H160 = *META_GETTER
        .get()
        .expect("AuthoringMetaGetter has not being initialized");

    let chain_id = provider.get_chainid().await.expect("cannot get chain id");

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    return AuthoringMetaGetter::new(meta_address, deployer)
        .get_authoring_meta()
        .await
        .expect("not able to get meta bytes");
}
