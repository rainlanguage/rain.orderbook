use crate::{
    add_order::{AddOrderArgs, AddOrderArgsError},
    frontmatter::parse_frontmatter,
    transaction::TransactionArgs,
};
use js_sys::Uint8Array;
use rain_orderbook_app_settings::{Config, ParseConfigSourceError};
use std::ops::Deref;
use thiserror::Error;
use wasm_bindgen::prelude::*;

/// Represents all possible errors of this module
#[derive(Debug, Error)]
pub enum Error {
    #[error("undefined deployment")]
    UndefinedDeployment,
    #[error(transparent)]
    ParseConfigSourceError(#[from] ParseConfigSourceError),
    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

/// Get addOrder() calldata from a given dotrain text and deployment key from its frontmatter
#[wasm_bindgen(js_name = "getAddOrderCalldata")]
pub async fn get_add_order_calldata(dotrain: &str, deployment: &str) -> Result<Uint8Array, Error> {
    let config: Config = parse_frontmatter(dotrain.to_string()).await?.try_into()?;
    let deployment_ref = config
        .deployments
        .get(deployment)
        .ok_or(Error::UndefinedDeployment)?;
    let add_order_args =
        AddOrderArgs::new_from_deployment(dotrain.to_string(), deployment_ref.deref().clone())
            .await?;

    let tx_args = TransactionArgs {
        rpc_url: deployment_ref.scenario.deployer.network.rpc.to_string(),
        ..Default::default()
    };
    Ok(add_order_args
        .get_add_order_calldata(tx_args)
        .await?
        .as_slice()
        .into())
}
