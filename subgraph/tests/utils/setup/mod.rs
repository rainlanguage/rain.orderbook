use crate::utils::deploy::deploy1820;
use anyhow::Result;
use ethers::providers::{Http, Provider};
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

static PROVIDER: Lazy<OnceCell<Provider<Http>>> = Lazy::new(|| OnceCell::new());

async fn provider_node() -> Result<Provider<Http>> {
    let provider_url = "http://localhost:8545";

    let provider: Provider<Http> = Provider::<Http>::try_from(provider_url)?;

    // Always checking if the Registry1820 is deployed. Deploy it otherwise
    let _ = deploy1820(&provider).await;

    Ok(provider)
}

pub async fn get_provider() -> Result<&'static Provider<Http>> {
    let provider_lazy = PROVIDER
        .get_or_try_init(|| async { provider_node().await })
        .await
        .map_err(|err| err);

    match provider_lazy {
        Ok(provider) => Ok(provider),
        Err(e) => return Err(anyhow::Error::msg(e.to_string())),
    }
}
