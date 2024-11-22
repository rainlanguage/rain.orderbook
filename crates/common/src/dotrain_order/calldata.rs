use super::*;
use crate::{
    add_order::{AddOrderArgs, AddOrderArgsError},
    deposit::{DepositArgs, DepositError},
    transaction::{TransactionArgs, WritableTransactionExecuteError},
};
use alloy::{
    hex::FromHexError,
    primitives::{Bytes, U256},
};
use rain_orderbook_app_settings::{deployment::Deployment, orderbook::Orderbook};
use std::{collections::HashMap, str::FromStr, sync::Arc};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::impl_wasm_traits;
#[cfg(target_family = "wasm")]
use serde_wasm_bindgen::{from_value, to_value};
#[cfg(target_family = "wasm")]
use tsify::Tsify;
#[cfg(target_family = "wasm")]
use wasm_bindgen::convert::{
    js_value_vector_from_abi, js_value_vector_into_abi, FromWasmAbi, IntoWasmAbi,
    LongRefFromWasmAbi, RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR};
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, UnwrapThrowExt};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(
    target_family = "wasm",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct ApprovalCalldata {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    token: Address,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    calldata: Bytes,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ApprovalCalldata);

impl DotrainOrder {
    fn get_deployment(
        &self,
        deployment_name: &str,
    ) -> Result<Arc<Deployment>, DotrainOrderCalldataError> {
        self.config.deployments.get(deployment_name).cloned().ok_or(
            DotrainOrderCalldataError::DeploymentNotFound(deployment_name.to_string()),
        )
    }

    fn get_orderbook(
        &self,
        deployment_name: &str,
    ) -> Result<Arc<Orderbook>, DotrainOrderCalldataError> {
        self.get_deployment(deployment_name)?
            .order
            .orderbook
            .clone()
            .ok_or(DotrainOrderCalldataError::OrderbookNotFound)
    }

    pub async fn generate_approval_calldatas(
        &self,
        deployment_name: &str,
        owner: &str,
        token_deposits: &HashMap<Address, U256>,
    ) -> Result<Vec<ApprovalCalldata>, DotrainOrderCalldataError> {
        let deployment = self.get_deployment(deployment_name)?;
        let orderbook = self.get_orderbook(deployment_name)?;

        let mut calldatas = Vec::new();

        for output in &deployment.order.outputs {
            if let Some(deposit_amount) = token_deposits.get(&output.token.address) {
                let deposit_amount = deposit_amount.to_owned();
                let deposit_args = DepositArgs {
                    token: output.token.address,
                    amount: deposit_amount,
                    vault_id: U256::default(),
                };
                let transaction_args = TransactionArgs {
                    orderbook_address: orderbook.address,
                    rpc_url: orderbook.network.rpc.to_string(),
                    ..Default::default()
                };

                let allowance = deposit_args
                    .read_allowance(Address::from_str(owner)?, transaction_args.clone())
                    .await?;

                if allowance < deposit_amount {
                    let approve_call = deposit_args
                        .get_approve_calldata(transaction_args, allowance)
                        .await?;
                    calldatas.push(ApprovalCalldata {
                        token: output.token.address,
                        calldata: Bytes::copy_from_slice(&approve_call),
                    });
                }
            }
        }

        Ok(calldatas)
    }

    pub async fn generate_deposit_calldatas(
        &mut self,
        deployment_name: &str,
        token_deposits: &HashMap<(U256, Address), U256>,
    ) -> Result<Vec<Bytes>, DotrainOrderCalldataError> {
        let deployment = self.get_deployment(deployment_name)?;
        let mut calldatas = Vec::new();

        for output in &deployment.order.outputs {
            let vault_id = output
                .vault_id
                .ok_or(DotrainOrderCalldataError::VaultIdNotFound)?;

            let token_deposit = token_deposits
                .get(&(vault_id, output.token.address))
                .ok_or(DotrainOrderCalldataError::TokenNotFound(
                    output.token.address.to_string(),
                ))?;
            let calldata = DepositArgs {
                token: output.token.address,
                amount: token_deposit.to_owned(),
                vault_id,
            }
            .get_deposit_calldata()
            .await?;

            calldatas.push(Bytes::copy_from_slice(&calldata));
        }

        Ok(calldatas)
    }

    pub async fn generate_add_order_calldata(
        &mut self,
        deployment_name: &str,
    ) -> Result<Bytes, DotrainOrderCalldataError> {
        let deployment = self.get_deployment(deployment_name)?;
        let orderbook = self.get_orderbook(deployment_name)?;

        let calldata = AddOrderArgs::new_from_deployment(
            self.dotrain().to_string(),
            deployment.as_ref().to_owned(),
        )
        .await?
        .get_add_order_calldata(TransactionArgs {
            orderbook_address: orderbook.address,
            rpc_url: orderbook.network.rpc.to_string(),
            ..Default::default()
        })
        .await?;

        Ok(Bytes::copy_from_slice(&calldata))
    }
}

#[derive(Debug, Error)]
pub enum DotrainOrderCalldataError {
    #[error("Deployment not found {0}")]
    DeploymentNotFound(String),

    #[error("Orderbook not found")]
    OrderbookNotFound,

    #[error("Token not found {0}")]
    TokenNotFound(String),

    #[error("Vault id not found")]
    VaultIdNotFound,

    #[error(transparent)]
    DepositError(#[from] DepositError),

    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
}
