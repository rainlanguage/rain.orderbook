use super::*;
use alloy::{
    primitives::private::rand,
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use alloy_ethers_typecast::multicall::IMulticall3::{aggregate3Call, Call3};
use rain_orderbook_app_settings::{order::OrderIO, orderbook::Orderbook};
use rain_orderbook_bindings::IOrderBookV4::addOrder2Call;
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

    fn get_deposits_as_map(&self) -> Result<HashMap<Address, U256>, GuiError> {
        let mut map: HashMap<Address, U256> = HashMap::new();
        for d in self.get_deposits() {
            let token_decimals = self
                .onchain_token_info
                .get(&d.address)
                .ok_or(GuiError::TokenNotFound)?
                .decimals;
            let amount = parse_units(&d.amount, token_decimals)?.into();
            map.insert(d.address, amount);
        }
        Ok(map)
    }

    fn get_vaults_and_deposits(&self) -> Result<Vec<(OrderIO, U256)>, GuiError> {
        let deposits_map = self.get_deposits_as_map()?;
        let results = self
            .deployment
            .deployment
            .order
            .outputs
            .clone()
            .into_iter()
            .filter(|output| deposits_map.contains_key(&output.token.address))
            .map(|output| {
                (
                    output.clone(),
                    *deposits_map.get(&output.token.address).unwrap(),
                )
            })
            .collect();
        Ok(results)
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
        let orderbook = self.get_orderbook()?;
        let vaults_and_deposits = self.get_vaults_and_deposits()?;

        let mut results = Vec::new();
        for (order_io, amount) in vaults_and_deposits.iter() {
            let deposit_args = DepositArgs {
                token: order_io.token.address,
                vault_id: rand::random(),
                amount: *amount,
            };
            let allowance = self
                .check_allowance(&orderbook, &deposit_args, &owner)
                .await?;
            results.push(allowance);
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    /// Generate approval calldatas for all inputs and outputs of the order
    ///
    /// Returns a vector of [`ApprovalCalldata`] objects
    #[wasm_bindgen(js_name = "generateApprovalCalldatas")]
    pub async fn generate_approval_calldatas(&self, owner: String) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let vaults_and_deposits = self.get_vaults_and_deposits()?;

        let mut results = Vec::new();
        for (order_io, amount) in vaults_and_deposits.iter() {
            let deposit_args = DepositArgs {
                token: order_io.token.address,
                vault_id: rand::random(),
                amount: *amount,
            };
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
        vaults_and_deposits: &Vec<(OrderIO, U256)>,
    ) -> Result<Vec<Vec<u8>>, GuiError> {
        let mut results = Vec::new();
        for (order_io, amount) in vaults_and_deposits.iter() {
            let deposit_args = DepositArgs {
                token: order_io.token.address,
                vault_id: order_io.vault_id.unwrap_or(rand::random()),
                amount: *amount,
            };
            let calldata = deposit_args.get_deposit_calldata().await?;
            results.push(calldata);
        }
        Ok(results)
    }

    /// Generate deposit calldatas for all inputs and outputs of the order
    ///
    /// Returns a [`DepositCalldatas`] object
    #[wasm_bindgen(js_name = "generateDepositCalldatas")]
    pub async fn generate_deposit_calldatas(&mut self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let vaults_and_deposits = self.get_vaults_and_deposits()?;

        let add_order_calldata = self.get_add_order_calldata(&orderbook).await?;
        let decoded_add_order_calldata = addOrder2Call::abi_decode(&add_order_calldata, true)?;
        self.set_vault_ids(&decoded_add_order_calldata);

        let calldatas = self
            .get_deposit_calldatas(&vaults_and_deposits)
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

    fn set_vault_ids(&mut self, decoded_add_order_calldata: &addOrder2Call) {
        let mut order = self.deployment.deployment.order.as_ref().clone();

        for (i, input) in order.inputs.iter_mut().enumerate() {
            input.vault_id = Some(decoded_add_order_calldata.config.validInputs[i].vaultId);
        }
        for (i, output) in order.outputs.iter_mut().enumerate() {
            output.vault_id = Some(decoded_add_order_calldata.config.validOutputs[i].vaultId);
        }

        let mut new_deployment = self.deployment.deployment.as_ref().clone();
        new_deployment.order = Arc::new(order);
        self.deployment.deployment = Arc::new(new_deployment);
    }

    /// Generate add order calldata
    #[wasm_bindgen(js_name = "generateAddOrderCalldata")]
    pub async fn generate_add_order_calldata(&mut self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let calldata = self.get_add_order_calldata(&orderbook).await?;
        Ok(serde_wasm_bindgen::to_value(&Bytes::copy_from_slice(
            &calldata,
        ))?)
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(&mut self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let vaults_and_deposits = self.get_vaults_and_deposits()?;
        let mut calls = Vec::new();

        let add_order_calldata = self.get_add_order_calldata(&orderbook).await?;
        let decoded_add_order_calldata = addOrder2Call::abi_decode(&add_order_calldata, true)?;
        self.set_vault_ids(&decoded_add_order_calldata);

        let deposit_calldatas = self.get_deposit_calldatas(&vaults_and_deposits).await?;

        for calldata in deposit_calldatas.iter() {
            calls.push(Call3 {
                target: orderbook.address,
                allowFailure: false,
                callData: Bytes::copy_from_slice(calldata),
            });
        }
        calls.push(Call3 {
            target: orderbook.address,
            allowFailure: false,
            callData: Bytes::copy_from_slice(&add_order_calldata),
        });

        Ok(serde_wasm_bindgen::to_value(&Bytes::copy_from_slice(
            &aggregate3Call { calls }.abi_encode(),
        ))?)
    }
}
