use super::*;
use alloy::{
    primitives::private::rand,
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use rain_orderbook_app_settings::{order::OrderIO, orderbook::Orderbook};
use rain_orderbook_bindings::OrderBook::multicallCall;
use rain_orderbook_common::{deposit::DepositArgs, dotrain_order, transaction::TransactionArgs};
use std::{collections::HashMap, str::FromStr, sync::Arc};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}
impl_all_wasm_traits!(TokenAllowance);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AllowancesResult(Vec<TokenAllowance>);
impl_all_wasm_traits!(AllowancesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct ApprovalCalldataResult(Vec<dotrain_order::calldata::ApprovalCalldata>);
impl_all_wasm_traits!(ApprovalCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DepositCalldataResult(Vec<Bytes>);
impl_all_wasm_traits!(DepositCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WithdrawCalldataResult(Vec<Bytes>);
impl_all_wasm_traits!(WithdrawCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AddOrderCalldataResult(Bytes);
impl_all_wasm_traits!(AddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DepositAndAddOrderCalldataResult(Bytes);
impl_all_wasm_traits!(DepositAndAddOrderCalldataResult);

#[wasm_bindgen]
impl DotrainOrderGui {
    fn get_orderbook(&self) -> Result<Arc<Orderbook>, GuiError> {
        let deployment = self.get_current_deployment()?;
        deployment
            .deployment
            .as_ref()
            .order
            .as_ref()
            .orderbook
            .as_ref()
            .ok_or(GuiError::OrderbookNotFound)
            .cloned()
    }

    async fn get_deposits_as_map(&self) -> Result<HashMap<String, U256>, GuiError> {
        let mut map: HashMap<String, U256> = HashMap::new();
        for d in self.get_deposits()? {
            let token_info = self.get_token_info(d.token.clone()).await?;
            let amount = parse_units(&d.amount, token_info.decimals)?.into();
            map.insert(d.token, amount);
        }
        Ok(map)
    }

    async fn get_vaults_and_deposits(
        &self,
        deployment: &GuiDeployment,
    ) -> Result<Vec<(OrderIO, U256)>, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        let results = deployment
            .deployment
            .order
            .outputs
            .clone()
            .into_iter()
            .filter(|output| {
                output
                    .token
                    .as_ref()
                    .map_or(false, |token| deposits_map.contains_key(&token.key))
            })
            .map(|output| {
                if output.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = output.token.as_ref().unwrap();

                Ok((output.clone(), *deposits_map.get(&token.key).unwrap()))
            })
            .collect::<Result<Vec<_>, GuiError>>()?;
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
    pub async fn check_allowances(&self, owner: String) -> Result<AllowancesResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;

        let orderbook = self.get_orderbook()?;
        let vaults_and_deposits = self.get_vaults_and_deposits(&deployment).await?;

        let mut results = Vec::new();
        for (order_io, amount) in vaults_and_deposits.iter() {
            if order_io.token.is_none() {
                return Err(GuiError::SelectTokensNotSet);
            }
            let token = order_io.token.as_ref().unwrap();

            let deposit_args = DepositArgs {
                token: token.address,
                vault_id: rand::random(),
                amount: *amount,
            };
            let allowance = self
                .check_allowance(&orderbook, &deposit_args, &owner)
                .await?;
            results.push(allowance);
        }

        Ok(AllowancesResult(results))
    }

    /// Generate approval calldatas for deposits
    ///
    /// Returns a vector of [`ApprovalCalldata`] objects
    #[wasm_bindgen(js_name = "generateApprovalCalldatas")]
    pub async fn generate_approval_calldatas(
        &self,
        owner: String,
    ) -> Result<ApprovalCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;

        let calldatas = self
            .dotrain_order
            .generate_approval_calldatas(
                &deployment.key,
                &owner,
                &self.get_deposits_as_map().await?,
            )
            .await?;
        Ok(ApprovalCalldataResult(calldatas))
    }

    fn populate_vault_ids(&mut self, deployment: &GuiDeployment) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .populate_vault_ids()?;
        Ok(())
    }

    fn update_config_source_bindings(
        &mut self,
        deployment: &GuiDeployment,
    ) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_scenario(&deployment.deployment.scenario.key)?
            .update_bindings(
                self.field_values
                    .iter()
                    .map(|(k, _)| Ok((k.clone(), self.get_field_value(k.clone())?.value.clone())))
                    .collect::<Result<HashMap<String, String>, GuiError>>()?,
            )?;
        Ok(())
    }

    /// Generate deposit calldatas for all deposits
    ///
    /// Returns a vector of bytes
    #[wasm_bindgen(js_name = "generateDepositCalldatas")]
    pub async fn generate_deposit_calldatas(&mut self) -> Result<DepositCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        self.populate_vault_ids(&deployment)?;
        let deployment = self.get_current_deployment()?;

        let token_deposits = self
            .get_vaults_and_deposits(&deployment)
            .await?
            .iter()
            .enumerate()
            .map(|(i, (order_io, amount))| {
                let vault_id = order_io
                    .vault_id
                    .ok_or(GuiError::VaultIdNotFound(i.to_string()))?;

                if order_io.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = order_io.token.as_ref().unwrap();

                Ok(((vault_id, token.address), *amount))
            })
            .collect::<Result<HashMap<_, _>, GuiError>>()?;
        let calldatas = self
            .dotrain_order
            .generate_deposit_calldatas(&deployment.key, &token_deposits)
            .await?;
        Ok(DepositCalldataResult(calldatas))
    }

    /// Generate add order calldata
    #[wasm_bindgen(js_name = "generateAddOrderCalldata")]
    pub async fn generate_add_order_calldata(
        &mut self,
    ) -> Result<AddOrderCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        self.populate_vault_ids(&deployment)?;
        self.update_config_source_bindings(&deployment)?;
        let deployment = self.get_current_deployment()?;

        let calldata = self
            .dotrain_order
            .generate_add_order_calldata(&deployment.key)
            .await?;
        Ok(AddOrderCalldataResult(calldata))
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(
        &mut self,
    ) -> Result<DepositAndAddOrderCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        self.populate_vault_ids(&deployment)?;
        self.update_config_source_bindings(&deployment)?;
        let deployment = self.get_current_deployment()?;

        let token_deposits = self
            .get_vaults_and_deposits(&deployment)
            .await?
            .iter()
            .enumerate()
            .map(|(i, (order_io, amount))| {
                let vault_id = order_io
                    .vault_id
                    .ok_or(GuiError::VaultIdNotFound(i.to_string()))?;

                if order_io.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = order_io.token.as_ref().unwrap();

                Ok(((vault_id, token.address), *amount))
            })
            .collect::<Result<HashMap<_, _>, GuiError>>()?;

        let mut calls = Vec::new();
        let deposit_calldatas = self
            .dotrain_order
            .generate_deposit_calldatas(&deployment.key, &token_deposits)
            .await?;
        let add_order_calldata = self
            .dotrain_order
            .generate_add_order_calldata(&deployment.key)
            .await?;

        calls.push(Bytes::copy_from_slice(&add_order_calldata));
        for calldata in deposit_calldatas.iter() {
            calls.push(Bytes::copy_from_slice(calldata));
        }

        Ok(DepositAndAddOrderCalldataResult(Bytes::copy_from_slice(
            &multicallCall { data: calls }.abi_encode(),
        )))
    }

    #[wasm_bindgen(js_name = "setVaultId")]
    pub fn set_vault_id(
        &mut self,
        is_input: bool,
        index: u8,
        vault_id: String,
    ) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .update_vault_id(is_input, index, vault_id)?;
        Ok(())
    }
}
