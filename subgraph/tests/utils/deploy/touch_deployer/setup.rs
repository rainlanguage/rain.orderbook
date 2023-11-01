use crate::generated::RainterpreterExpressionDeployer;
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

use super::touch_deployer;

static EXPRESSION_DEPLOYER: Lazy<
    OnceCell<RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
> = Lazy::new(|| OnceCell::new());

#[derive(Error, Debug)]
pub enum ExpressionDeployerSetupError {
    #[error("An error occurred when deploying ExpressionDeployer at initialization provider instance: {0}")]
    InitDeployDeployerError(#[from] Box<dyn std::error::Error>),
}

// PROVIDER CODE INIT
/// Deploy (initialize) an expression deployer contract to be used across the setup
pub async fn init_deployer() -> Result<
    RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ExpressionDeployerSetupError,
> {
    // By providing `None` as wallet, the function will use the default - The wallet at index 0.
    let contract = touch_deployer(None)
        .await
        .expect("cannot deploy expression deployer at setup initialization");

    Ok(contract)
}

async fn try_deployer_deploy() -> Result<
    RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ExpressionDeployerSetupError,
> {
    match init_deployer().await {
        Ok(data) => Ok(data),
        Err(err) => Err(ExpressionDeployerSetupError::InitDeployDeployerError(
            Box::new(err),
        )),
    }
}

/// Obtain the RainterpreterExpressionDeployer deployed for the test
pub async fn get_expression_deployer() -> Result<
    &'static RainterpreterExpressionDeployer<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
> {
    let orderbook_lazy = EXPRESSION_DEPLOYER
        .get_or_try_init(|| async { try_deployer_deploy().await })
        .await
        .map_err(|err| ExpressionDeployerSetupError::InitDeployDeployerError(Box::new(err)));

    match orderbook_lazy {
        Ok(contract) => Ok(contract),
        Err(e) => return Err(anyhow::Error::msg(e.to_string())),
    }
}
