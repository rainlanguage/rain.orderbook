use crate::{generated::AuthoringMetaGetter, utils::setup::get_wallets_handler};
use ethers::types::{Bytes, H160};
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

static META_GETTER: Lazy<OnceCell<H160>> = Lazy::new(|| OnceCell::new());

async fn meta_getter() -> anyhow::Result<H160> {
    match authoring_meta_getter_deploy().await {
        Ok(address) => Ok(address),
        Err(error) => Err(error),
    }
}

async fn get_meta_address() -> anyhow::Result<&'static H160> {
    META_GETTER
        .get_or_try_init(|| async { meta_getter().await })
        .await
        .map_err(|e| e)
}

async fn authoring_meta_getter_deploy() -> anyhow::Result<H160> {
    let deployer = get_wallets_handler().get_client(0).await?;
    let contract = AuthoringMetaGetter::deploy(deployer, ())?.send().await?;
    Ok(contract.address())
}

/// Get the AuthoringMeta bytes to deploy ExpressionDeployers.
pub async fn get_authoring_meta() -> anyhow::Result<Bytes> {
    let meta_address = get_meta_address().await?;
    let deployer = get_wallets_handler().get_client(0).await?;

    Ok(AuthoringMetaGetter::new(*meta_address, deployer)
        .get_authoring_meta()
        .await?)
}
