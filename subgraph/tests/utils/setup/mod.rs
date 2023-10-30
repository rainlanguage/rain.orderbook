use crate::utils::deploy::{deploy1820, get_meta_address};
use anyhow::Result;
use ethers::providers::{Http, Provider};
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::sync::OnceCell;

static PROVIDER: Lazy<OnceCell<Provider<Http>>> = Lazy::new(|| OnceCell::new());

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("An error occurred when creating provider instance: {0}")]
    ProviderInstanceError(#[from] Box<dyn std::error::Error>),
}

// PROVIDER CODE INIT
pub async fn init_provider() -> Result<Provider<Http>, SetupError> {
    let provider_url = "http://localhost:8545";

    let provider: Provider<Http> =
        Provider::<Http>::try_from(provider_url).expect("could not instantiate Provider");

    // Always checking if the Registry1820 is deployed. Deploy it otherwise
    let _ = deploy1820(&provider).await;

    get_meta_address(&provider)
        .await
        .expect("cannot deploy AuthoringMetaGetter at initialization");

    Ok(provider)
}

async fn provider_node() -> Result<Provider<Http>, SetupError> {
    match init_provider().await {
        Ok(data) => Ok(data),
        Err(err) => Err(SetupError::ProviderInstanceError(Box::new(err))),
    }
}

pub async fn get_provider() -> Result<&'static Provider<Http>> {
    let provider_lazy = PROVIDER
        .get_or_try_init(|| async { provider_node().await })
        .await
        .map_err(|err| SetupError::ProviderInstanceError(Box::new(err)));

    match provider_lazy {
        Ok(provider) => Ok(provider),
        Err(e) => return Err(anyhow::Error::msg(e.to_string())),
    }
}
