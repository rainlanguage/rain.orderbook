use crate::{
    generated::OrderBook,
    subgraph::{deploy, Config},
    utils::get_block_number,
};
use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
};
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::sync::OnceCell;

use super::deploy_orderbook;

static ORDERBOOK: Lazy<OnceCell<OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>> =
    Lazy::new(|| OnceCell::new());

#[derive(Error, Debug)]
pub enum OrderBookSetupError {
    #[error("An error occurred when deploying OB at initialization provider instance: {0}")]
    InitDeployOBError(#[from] Box<dyn std::error::Error>),
    #[error("An error occurred when deploying the OB subgraph")]
    SgDeployError(),
}

// PROVIDER CODE INIT
/// Deploy (initialize) an orderbook contract to be used across the setup
pub async fn init_orderbook(
) -> Result<OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, OrderBookSetupError> {
    // By providing `None` as wallet, the function will use the default - The wallet at index 0.
    let orderbook = deploy_orderbook(None)
        .await
        .expect("cannot deploy OB at setup initialization");

    let sg_config = Config {
        contract_address: &format!("{:?}", orderbook.address()),
        block_number: get_block_number()
            .await
            .expect("cannot get block number")
            .as_u64(),
    };

    let is_sg_deployed = deploy(sg_config).expect("cannot deploy OB SG at setup initialization");

    if is_sg_deployed {
        Ok(orderbook)
    } else {
        Err(OrderBookSetupError::SgDeployError())
    }
    // Ok(orderbook)
}

async fn try_ob_deploy(
) -> Result<OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, OrderBookSetupError> {
    match init_orderbook().await {
        Ok(data) => Ok(data),
        Err(err) => Err(OrderBookSetupError::InitDeployOBError(Box::new(err))),
    }
}

/// Obtain the OB deployed for the test
pub async fn get_orderbook(
) -> Result<&'static OrderBook<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let orderbook_lazy = ORDERBOOK
        .get_or_try_init(|| async { try_ob_deploy().await })
        .await
        .map_err(|err| OrderBookSetupError::InitDeployOBError(Box::new(err)));

    match orderbook_lazy {
        Ok(contract) => Ok(contract),
        Err(e) => return Err(anyhow::Error::msg(e.to_string())),
    }
}
