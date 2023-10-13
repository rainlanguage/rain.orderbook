use super::deploy::{registry1820::deploy1820, meta_getter::get_meta_address};
use ethers::providers::{Http, Provider};
use once_cell::sync::Lazy;
use reqwest;
use thiserror::Error;
use tokio::{
    sync::OnceCell,
    time::{timeout, Duration},
};

static SUBGRAPH: Lazy<OnceCell<bool>> = Lazy::new(|| OnceCell::new());
static PROVIDER: Lazy<OnceCell<Provider<Http>>> = Lazy::new(|| OnceCell::new());

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("An error occurred during initialization: {0}")]
    InitializationError(#[from] Box<dyn std::error::Error>),
    #[error("An error occurred when creating provider instance")]
    ProviderInstanceError(),
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
        Err(_) => Err(SetupError::ProviderInstanceError()),
    }
}

pub async fn get_provider() -> Result<&'static Provider<Http>, SetupError> {
    PROVIDER
        .get_or_try_init(|| async { provider_node().await })
        .await
        .map_err(|_| SetupError::ProviderInstanceError())
}

// SUBGRAPH CODE INIT
async fn subgraph_node_init() -> Result<bool, SetupError> {
    let is_running = check_subgraph_node().await;

    Ok(is_running)
}

async fn subgraph_node() -> Result<bool, SetupError> {
    // If an error occurs, wrap it using MyError::InitializationError
    match subgraph_node_init().await {
        Ok(data) => Ok(data),
        Err(err) => Err(SetupError::InitializationError(Box::new(err))),
    }
}

pub async fn is_sugraph_node_init() -> Result<&'static bool, SetupError> {
    SUBGRAPH
        .get_or_try_init(|| async { subgraph_node().await })
        .await
        .map_err(|e| SetupError::InitializationError(Box::new(e)))
}

/// Check if the subgraph node is live to be able to deploy subgraphs
pub async fn check_subgraph_node() -> bool {
    let client = reqwest::Client::new();

    let url = "http://localhost:8030";

    let mut retries = 0;
    // Max retries allowed
    let max_retries = 6;
    // Retry interval
    let retry_interval = Duration::from_secs(5);

    loop {
        retries += 1;
        // Send an HTTP GET request with a timeout
        let response = timeout(Duration::from_secs(5), client.get(url).send())
            .await
            .expect("No reqyest sent to the url");

        match response {
            Ok(res) if (res.status().is_success()) => {
                return true;
            }
            _ => {
                if retries >= max_retries {
                    if retries >= max_retries {
                        println!("Max retries reached. Exiting.");
                        // return Err(reqwest::Error::from("Max retries reached"));
                        return false;
                    }
                }
                println!(
                    "Retry attempt {} failed. Retrying in {} seconds...",
                    retries,
                    retry_interval.as_secs()
                );
                tokio::time::sleep(retry_interval).await;
            }
        }
    }
}
