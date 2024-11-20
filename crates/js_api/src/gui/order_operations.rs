use super::*;
use alloy::{
    primitives::private::rand,
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use alloy_ethers_typecast::multicall::IMulticall3::{aggregate3Call, Call3};
use rain_orderbook_app_settings::{order::OrderIO, orderbook::Orderbook};
use rain_orderbook_common::{deposit::DepositArgs, transaction::TransactionArgs};
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

    /// Generate approval calldatas for deposits
    ///
    /// Returns a vector of [`ApprovalCalldata`] objects
    #[wasm_bindgen(js_name = "generateApprovalCalldatas")]
    pub async fn generate_approval_calldatas(&self, owner: String) -> Result<JsValue, GuiError> {
        let calldatas = self
            .dotrain_order
            .generate_approval_calldatas(
                &self.deployment.deployment_name,
                &owner,
                &self.get_deposits_as_map()?,
            )
            .await?;
        Ok(serde_wasm_bindgen::to_value(&calldatas)?)
    }

    /// Generate deposit calldatas for all deposits
    ///
    /// Returns a vector of bytes
    #[wasm_bindgen(js_name = "generateDepositCalldatas")]
    pub async fn generate_deposit_calldatas(&mut self) -> Result<JsValue, GuiError> {
        let token_deposits = self
            .get_vaults_and_deposits()?
            .iter()
            .map(|(order_io, amount)| {
                (
                    (order_io.vault_id.unwrap(), order_io.token.address),
                    *amount,
                )
            })
            .collect::<HashMap<_, _>>();
        let calldatas = self
            .dotrain_order
            .generate_deposit_calldatas(&self.deployment.deployment_name, &token_deposits)
            .await?;
        Ok(serde_wasm_bindgen::to_value(&calldatas)?)
    }

    /// Generate add order calldata
    #[wasm_bindgen(js_name = "generateAddOrderCalldata")]
    pub async fn generate_add_order_calldata(&mut self) -> Result<JsValue, GuiError> {
        let calldata = self
            .dotrain_order
            .generate_add_order_calldata(&self.deployment.deployment_name)
            .await?;
        Ok(serde_wasm_bindgen::to_value(&calldata)?)
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(&mut self) -> Result<JsValue, GuiError> {
        let orderbook = self.get_orderbook()?;
        let token_deposits = self
            .get_vaults_and_deposits()?
            .iter()
            .map(|(order_io, amount)| {
                (
                    (order_io.vault_id.unwrap(), order_io.token.address),
                    *amount,
                )
            })
            .collect::<HashMap<_, _>>();

        let mut calls = Vec::new();
        let deposit_calldatas = self
            .dotrain_order
            .generate_deposit_calldatas(&self.deployment.deployment_name, &token_deposits)
            .await?;
        let add_order_calldata = self
            .dotrain_order
            .generate_add_order_calldata(&self.deployment.deployment_name)
            .await?;

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

    #[wasm_bindgen(js_name = "populateVaultIds")]
    pub fn populate_vault_ids(&mut self) -> Result<(), GuiError> {
        self.dotrain_order
            .populate_vault_ids(&self.deployment.deployment_name)?;
        Ok(())
    }
}
