use crate::{
    generated::AuthoringMetaGetter,
    utils::{get_provider, get_wallet},
};
use ethers::{
    prelude::SignerMiddleware,
    providers::Middleware,
    signers::Signer,
    types::{Bytes, H160},
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::OnceCell;

static META_GETTER: Lazy<OnceCell<H160>> = Lazy::new(|| OnceCell::new());

async fn meta_getter() -> anyhow::Result<H160> {
    match authoring_meta_getter_deploy().await {
        Ok(address) => Ok(address),
        Err(error) => Err(error),
    }
}

///
pub async fn get_meta_address() -> anyhow::Result<&'static H160> {
    META_GETTER
        .get_or_try_init(|| async { meta_getter().await })
        .await
        .map_err(|e| e)
}

pub async fn authoring_meta_getter_deploy() -> anyhow::Result<H160> {
    let provider = get_provider().await?;
    let wallet = get_wallet(0)?;

    let chain_id = provider.get_chainid().await?;

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let contract = AuthoringMetaGetter::deploy(deployer, ())?.send().await?;

    Ok(contract.address())
}

/// Get the AuthoringMeta bytes to deploy ExpressionDeployers.
pub async fn get_authoring_meta() -> anyhow::Result<Bytes> {
    let provider = get_provider().await?;
    let wallet = get_wallet(0)?;

    let meta_address = get_meta_address().await?;

    let chain_id = provider.get_chainid().await?;

    let deployer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(chain_id.as_u64()),
    ));

    let meta_bytes = AuthoringMetaGetter::new(*meta_address, deployer)
        .get_authoring_meta()
        .await?;

    Ok(meta_bytes)
}
