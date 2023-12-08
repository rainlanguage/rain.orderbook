use super::deploy::deploy_orderbook;
use crate::{generated::Orderbook, utils::setup::get_rpc_provider};
use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
};
use once_cell::sync::Lazy;
use rain_cli_subgraph::subgraph;
use tokio::sync::OnceCell;

static ORDERBOOK: Lazy<OnceCell<Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>> =
    Lazy::new(|| OnceCell::new());

async fn init_orderbook() -> Result<Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>
{
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let provider = get_rpc_provider().await?;
    let block = provider.get_block_number().await?;

    let orderbook = deploy_orderbook().await?;

    let build_args = subgraph::build::BuildArgs {
        address: Some(format!("{:?}", orderbook.address())),
        block_number: Some(block.as_u64()),
        network: Some("localhost".to_string()),
        template_path: None,
        output_path: None,
    };
    let resp_build = subgraph::build::build(build_args);
    if resp_build.is_err() {
        return Err(anyhow::anyhow!(resp_build.err().unwrap()));
    }

    let deploy_args = subgraph::deploy::DeployArgs {
        subgraph_name: "test/test".to_string(),
        endpoint: Some("http://localhost:8020/".to_string()),
        token_access: None,
    };

    let resp_deploy = subgraph::deploy::deploy(deploy_args);
    if resp_deploy.is_err() {
        return Err(anyhow::anyhow!(resp_deploy.err().unwrap()));
    }

    Ok(orderbook)
}

pub async fn get_orderbook(
) -> Result<&'static Orderbook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let orderbook_lazy = ORDERBOOK
        .get_or_try_init(|| async { init_orderbook().await })
        .await
        .map_err(|err| err);

    match orderbook_lazy {
        Ok(contract) => Ok(contract),
        Err(e) => return Err(anyhow::Error::msg(e.to_string())),
    }
}
