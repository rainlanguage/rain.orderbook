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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ApprovalCalldata {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub token: Address,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub calldata: Bytes,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(ApprovalCalldata);

impl DotrainOrder {
    fn get_deployment(
        &self,
        deployment_name: &str,
    ) -> Result<Deployment, DotrainOrderCalldataError> {
        Ok(self.dotrain_yaml.get_deployment(deployment_name)?)
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
        token_deposits: &HashMap<String, U256>,
    ) -> Result<Vec<ApprovalCalldata>, DotrainOrderCalldataError> {
        let deployment = self.get_deployment(deployment_name)?;
        let orderbook = self.get_orderbook(deployment_name)?;

        let mut calldatas = Vec::new();

        for (i, output) in deployment.order.outputs.iter().enumerate() {
            let output_token = output
                .token
                .as_ref()
                .ok_or_else(|| DotrainOrderCalldataError::OutputTokenNotFound(i.to_string()))?;

            if let Some(deposit_amount) = token_deposits.get(&output_token.key) {
                let deposit_amount = deposit_amount.to_owned();
                let deposit_args = DepositArgs {
                    token: output_token.address,
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
                    let approve_call = deposit_args.get_approve_calldata(transaction_args).await?;
                    calldatas.push(ApprovalCalldata {
                        token: output_token.address,
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

        for (i, output) in deployment.order.outputs.iter().enumerate() {
            let output_token = output
                .token
                .as_ref()
                .ok_or_else(|| DotrainOrderCalldataError::OutputTokenNotFound(i.to_string()))?;
            let vault_id = output
                .vault_id
                .ok_or(DotrainOrderCalldataError::VaultIdNotFound(i.to_string()))?;

            let token_deposit = token_deposits
                .get(&(vault_id, output_token.address))
                .ok_or(DotrainOrderCalldataError::TokenNotFound(
                    output_token.address.to_string(),
                ))?;

            if *token_deposit == U256::ZERO {
                continue;
            }

            let calldata = DepositArgs {
                token: output_token.address,
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

        let calldata = AddOrderArgs::new_from_deployment(self.dotrain().to_string(), deployment)
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

    #[error("Token not found for output index: {0}")]
    OutputTokenNotFound(String),

    #[error("Vault id not found for output index: {0}")]
    VaultIdNotFound(String),

    #[error("Token not found {0}")]
    TokenNotFound(String),

    #[error(transparent)]
    DepositError(#[from] DepositError),

    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    FromHexError(#[from] FromHexError),

    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),

    #[error(transparent)]
    YamlError(#[from] YamlError),
}
