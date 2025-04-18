use crate::{
    add_order::{AddOrderArgs, AddOrderArgsError},
    frontmatter::parse_frontmatter,
    remove_order::{RemoveOrderArgs, RemoveOrderArgsError},
    transaction::TransactionArgs,
};
use alloy::primitives::Bytes;
use js_sys::Uint8Array;
use rain_orderbook_app_settings::ParseConfigError;
use rain_orderbook_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct RemoveOrderCalldata(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(RemoveOrderCalldata);

/// Represents all possible errors of this module
#[derive(Debug, Error)]
pub enum Error {
    #[error("undefined deployment")]
    UndefinedDeployment,
    #[error(transparent)]
    ParseConfigError(#[from] ParseConfigError),
    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
    #[error(transparent)]
    RemoveOrderArgsError(#[from] RemoveOrderArgsError),
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

/// Get addOrder() calldata from a given dotrain text and deployment key from its frontmatter
#[wasm_bindgen(js_name = "getAddOrderCalldata")]
pub async fn get_add_order_calldata(dotrain: &str, deployment: &str) -> Result<Uint8Array, Error> {
    let config = parse_frontmatter(dotrain.to_string())?;
    let deployment_ref = config.get_deployment(deployment)?;
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

/// Get removeOrder() calldata for a given order
#[wasm_bindgen(js_name = "getRemoveOrderCalldata")]
pub async fn get_remove_order_calldata(order: SgOrder) -> Result<RemoveOrderCalldata, Error> {
    let remove_order_args = RemoveOrderArgs { order };
    let calldata = remove_order_args.get_rm_order_calldata().await?;
    Ok(RemoveOrderCalldata(Bytes::copy_from_slice(&calldata)))
}
