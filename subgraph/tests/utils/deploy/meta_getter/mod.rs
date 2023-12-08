use crate::{generated::AuthoringMetaGetter, utils::setup::get_wallets_handler};
use ethers::types::Bytes;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

static META_GETTER: Lazy<OnceCell<Bytes>> = Lazy::new(|| OnceCell::new());

async fn meta_getter() -> anyhow::Result<Bytes> {
    let deployer = get_wallets_handler().get_client(0).await?;
    let contract = AuthoringMetaGetter::deploy(deployer, ())?.send().await?;
    Ok(contract.get_authoring_meta().await?)
}

pub async fn get_authoring_meta() -> anyhow::Result<&'static Bytes> {
    META_GETTER
        .get_or_try_init(|| async { meta_getter().await })
        .await
        .map_err(|e| e)
}


