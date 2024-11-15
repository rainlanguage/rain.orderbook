use super::*;
use alloy::{
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use alloy_ethers_typecast::multicall::IMulticall3::{aggregate3Call, Call3};
use rain_orderbook_app_settings::{order::OrderIO, orderbook::Orderbook};
use rain_orderbook_common::{
    add_order::AddOrderArgs, deposit::DepositArgs, transaction::TransactionArgs,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}
impl_wasm_traits!(TokenAllowance);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ApprovalCalldata {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    calldata: Bytes,
}
impl_wasm_traits!(ApprovalCalldata);

#[wasm_bindgen]
impl DotrainOrderGui {
    fn get_orderbook(&self) -> Result<Arc<Orderbook>, GuiError> {
        self.deployment
            .deployment
            .as_ref()
            .order
            .as_ref()
            .orderbook
            .as_ref()
            .ok_or(GuiError::OrderbookNotFound)
            .cloned()
    }

    fn get_all_vaults(&self) -> Vec<OrderIO> {
        let mut all_vaults: Vec<_> = self.deployment.deployment.order.inputs.clone();
        let filtered_outputs: Vec<OrderIO> = self
            .deployment
            .deployment
            .order
            .outputs
            .iter()
            .filter(|output| !all_vaults.contains(*output))
            .cloned()
            .collect();
        all_vaults.extend(filtered_outputs);
        all_vaults
    }

    fn get_deposits_as_map(&self) -> HashMap<Address, String> {
        self.get_deposits()
            .into_iter()
            .map(|d| (d.address, d.amount))
            .collect()
    }

    fn get_deposit_args(
        &self,
        deposits_map: &HashMap<Address, String>,
        order_io: &OrderIO,
    ) -> Result<DepositArgs, GuiError> {
        Ok(DepositArgs {
            token: order_io.token.address,
            vault_id: order_io.vault_id.ok_or(GuiError::VaultIdNotFound)?,
            amount: parse_units(
                &deposits_map
                    .get(&order_io.token.address)
                    .ok_or(GuiError::TokenNotFound)?,
                // TODO: if decimals are not provided, we should get them from the token contract
                order_io.token.decimals.unwrap_or(18),
            )?
            .get_absolute(),
        })
    }

    async fn check_allowance(
        &self,
        orderbook: &Orderbook,
        deposit_args: &DepositArgs,
        owner: &str,
    ) -> Result<TokenAllowance, GuiError> {
        let allowance = deposit_args
            .read_allowance(
                Address::from_str(owner)?,
                TransactionArgs {
                    orderbook_address: orderbook.address,
                    rpc_url: orderbook.network.rpc.to_string(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(TokenAllowance {
            token: deposit_args.token,
            allowance,
        })
    }

    /// Check allowances for all inputs and outputs of the order
    ///
    /// Returns a vector of [`TokenAllowance`] objects
    #[wasm_bindgen(js_name = "checkAllowances")]
    pub async fn check_allowances(&self, owner: String) -> Result<JsValue, GuiError> {
        let all_vaults = self.get_all_vaults();
        let orderbook = self.get_orderbook()?;
        let deposits_map = self.get_deposits_as_map();

        let mut results = Vec::new();
        for order_io in all_vaults.iter() {
            let deposit_args = self.get_deposit_args(&deposits_map, order_io)?;
            let allowance = self
                .check_allowance(&orderbook, &deposit_args, &owner)
                .await?;
            results.push(TokenAllowance {
                token: allowance.token,
                allowance: allowance.allowance,
            });
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// Generate approval calldatas for all inputs and outputs of the order
    ///
    /// Returns a vector of [`ApprovalCalldata`] objects
    #[wasm_bindgen(js_name = "generateApprovalCalldatas")]
    pub async fn generate_approval_calldatas(&self, owner: String) -> Result<JsValue, GuiError> {
        let all_vaults = self.get_all_vaults();
        let orderbook = self.get_orderbook()?;
        let deposits_map = self.get_deposits_as_map();

        let mut results = Vec::new();
        for order_io in all_vaults.iter() {
            let deposit_args = self.get_deposit_args(&deposits_map, order_io)?;
            let token_allowance = self
                .check_allowance(&orderbook, &deposit_args, &owner)
                .await?;

            if token_allowance.allowance < deposit_args.amount {
                let calldata = deposit_args
                    .get_approve_calldata(
                        TransactionArgs {
                            orderbook_address: orderbook.address,
                            rpc_url: orderbook.network.rpc.to_string(),
                            ..Default::default()
                        },
                        token_allowance.allowance,
                    )
                    .await?;
                results.push(ApprovalCalldata {
                    token: token_allowance.token,
                    calldata: Bytes::copy_from_slice(&calldata),
                });
            }
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    async fn get_deposit_calldatas(
        &self,
        all_vaults: &Vec<OrderIO>,
        deposits_map: &HashMap<Address, String>,
    ) -> Result<Vec<Vec<u8>>, GuiError> {
        let mut results = Vec::new();
        for order_io in all_vaults.iter() {
            let deposit_args = self.get_deposit_args(&deposits_map, order_io)?;
            let calldata = deposit_args.get_deposit_calldata().await?;
            results.push(calldata);
        }
        Ok(results)
    }

    /// Generate deposit calldatas for all inputs and outputs of the order
    ///
    /// Returns a [`DepositCalldatas`] object
    #[wasm_bindgen(js_name = "generateDepositCalldatas")]
    pub async fn generate_deposit_calldatas(&self) -> Result<JsValue, GuiError> {
        let all_vaults = self.get_all_vaults();
        let deposits_map = self.get_deposits_as_map();

        let calldatas = self
            .get_deposit_calldatas(&all_vaults, &deposits_map)
            .await?
            .iter()
            .map(|c| Bytes::copy_from_slice(c))
            .collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&calldatas)?)
    }

    async fn get_add_order_calldata(
        &self,
        orderbook: &Arc<Orderbook>,
    ) -> Result<Vec<u8>, GuiError> {
        let add_order_args = AddOrderArgs::new_from_deployment(
            self.dotrain_order.dotrain(),
            self.deployment.deployment.as_ref().clone(),
        )
        .await?;
        let calldata = add_order_args
            .get_add_order_calldata(TransactionArgs {
                orderbook_address: orderbook.address,
                rpc_url: orderbook.network.rpc.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(calldata.into())
    }

    /// Generate add order calldata
    #[wasm_bindgen(js_name = "generateAddOrderCalldata")]
    pub async fn generate_add_order_calldata(&self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let calldata = self.get_add_order_calldata(&orderbook).await?;
        Ok(serde_wasm_bindgen::to_value(&Bytes::copy_from_slice(
            &calldata,
        ))?)
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(&self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let all_vaults = self.get_all_vaults();
        let deposits_map = self.get_deposits_as_map();
        let mut calls = Vec::new();

        let deposit_calldatas = self
            .get_deposit_calldatas(&all_vaults, &deposits_map)
            .await?;
        for calldata in deposit_calldatas.iter() {
            calls.push(Call3 {
                target: orderbook.address,
                allowFailure: false,
                callData: Bytes::copy_from_slice(calldata),
            });
        }

        let add_order_calldata = self.get_add_order_calldata(&orderbook).await?;
        calls.push(Call3 {
            target: orderbook.address,
            allowFailure: false,
            callData: Bytes::copy_from_slice(&add_order_calldata),
        });

        let aggregate_call = aggregate3Call { calls };
        let calldata = aggregate_call.abi_encode();

        Ok(serde_wasm_bindgen::to_value(&calldata)?)
    }
}
