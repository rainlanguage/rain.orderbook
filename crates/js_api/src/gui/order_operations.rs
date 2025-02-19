use super::*;
use alloy::{
    primitives::private::rand,
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use rain_orderbook_app_settings::{order::OrderIO, orderbook::Orderbook};
use rain_orderbook_bindings::OrderBook::multicallCall;
use rain_orderbook_common::{
    add_order::AddOrderArgs, deposit::DepositArgs, transaction::TransactionArgs,
};
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
pub enum ApprovalCalldataResult {
    NoDeposits,
    Calldatas(Vec<ApprovalCalldata>),
}
impl_all_wasm_traits!(ApprovalCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum DepositCalldataResult {
    NoDeposits,
    Calldatas(Vec<Bytes>),
}
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct IOVaultIds(HashMap<String, Vec<Option<U256>>>);
impl_all_wasm_traits!(IOVaultIds);

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

#[derive(Debug)]
pub struct VaultAndDeposit {
    pub order_io: OrderIO,
    pub deposit_amount: U256,
    pub index: usize,
}

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

    fn get_transaction_args(&self) -> Result<TransactionArgs, GuiError> {
        let orderbook = self.get_orderbook()?;
        Ok(TransactionArgs {
            orderbook_address: orderbook.address,
            rpc_url: orderbook.network.rpc.to_string(),
            ..Default::default()
        })
    }

    async fn get_deposits_as_map(&self) -> Result<HashMap<Address, U256>, GuiError> {
        let mut map: HashMap<Address, U256> = HashMap::new();
        for d in self.get_deposits()? {
            let token_info = self.get_token_info(d.token.clone()).await?;
            let amount = parse_units(&d.amount, token_info.decimals)?.into();
            map.insert(token_info.address, amount);
        }
        Ok(map)
    }

    async fn get_vaults_and_deposits(
        &self,
        deployment: &GuiDeployment,
    ) -> Result<Vec<VaultAndDeposit>, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        let results = deployment
            .deployment
            .order
            .outputs
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, output)| {
                output
                    .token
                    .as_ref()
                    .map_or(false, |token| deposits_map.contains_key(&token.address))
            })
            .map(|(index, output)| {
                if output.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = output.token.as_ref().unwrap();

                Ok(VaultAndDeposit {
                    order_io: output.clone(),
                    deposit_amount: *deposits_map.get(&token.address).unwrap(),
                    index,
                })
            })
            .collect::<Result<Vec<_>, GuiError>>()?;
        Ok(results)
    }

    async fn check_allowance(
        &self,
        deposit_args: &DepositArgs,
        owner: &str,
    ) -> Result<TokenAllowance, GuiError> {
        let allowance = deposit_args
            .read_allowance(Address::from_str(owner)?, self.get_transaction_args()?)
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

        let vaults_and_deposits = self.get_vaults_and_deposits(&deployment).await?;

        let mut results = Vec::new();
        for VaultAndDeposit {
            order_io,
            deposit_amount,
            index: _,
        } in vaults_and_deposits
        {
            let allowance = self
                .check_allowance(
                    &DepositArgs {
                        token: order_io
                            .token
                            .as_ref()
                            .ok_or(GuiError::SelectTokensNotSet)?
                            .address,
                        vault_id: rand::random(),
                        amount: deposit_amount,
                    },
                    &owner,
                )
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
        self.check_select_tokens()?;

        let deposits_map = self.get_deposits_as_map().await?;
        if deposits_map.is_empty() {
            return Ok(ApprovalCalldataResult::NoDeposits);
        }

        let transaction_args = self.get_transaction_args()?;

        let mut calldatas = Vec::new();
        for (token_address, deposit_amount) in deposits_map {
            let deposit_args = DepositArgs {
                token: token_address.clone(),
                amount: deposit_amount,
                vault_id: U256::default(),
            };

            let token_allowance = self.check_allowance(&deposit_args, &owner).await?;
            if token_allowance.allowance < deposit_amount {
                let approve_call = deposit_args
                    .get_approve_calldata(transaction_args.clone())
                    .await?;
                calldatas.push(ApprovalCalldata {
                    token: token_address,
                    calldata: Bytes::copy_from_slice(&approve_call),
                });
            }
        }

        Ok(ApprovalCalldataResult::Calldatas(calldatas))
    }

    fn populate_vault_ids(&mut self, deployment: &GuiDeployment) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .populate_vault_ids()?;
        Ok(())
    }

    fn update_bindings(&mut self, deployment: &GuiDeployment) -> Result<(), GuiError> {
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

        let vaults_and_deposits = self.get_vaults_and_deposits(&deployment).await?;
        if vaults_and_deposits.is_empty() {
            return Ok(DepositCalldataResult::NoDeposits);
        }

        let mut calldatas = Vec::new();
        for VaultAndDeposit {
            order_io,
            deposit_amount,
            index,
        } in vaults_and_deposits
        {
            let token = order_io
                .token
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?;
            let vault_id = order_io
                .vault_id
                .ok_or(GuiError::VaultIdNotFound(index.to_string()))?;

            if deposit_amount == U256::ZERO {
                continue;
            }

            let deposit_args = DepositArgs {
                token: token.address,
                amount: deposit_amount,
                vault_id,
            };
            let calldata = deposit_args.get_deposit_calldata().await?;
            calldatas.push(Bytes::copy_from_slice(&calldata));
        }

        Ok(DepositCalldataResult::Calldatas(calldatas))
    }

    /// Generate add order calldata
    #[wasm_bindgen(js_name = "generateAddOrderCalldata")]
    pub async fn generate_add_order_calldata(
        &mut self,
    ) -> Result<AddOrderCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        self.check_field_values()?;
        self.populate_vault_ids(&deployment)?;
        self.update_bindings(&deployment)?;
        let deployment = self.get_current_deployment()?;

        let calldata = AddOrderArgs::new_from_deployment(
            self.dotrain_order.dotrain().to_string(),
            deployment.deployment.as_ref().clone(),
        )
        .await?
        .get_add_order_calldata(self.get_transaction_args()?)
        .await?;

        Ok(AddOrderCalldataResult(Bytes::copy_from_slice(&calldata)))
    }

    #[wasm_bindgen(js_name = "generateDepositAndAddOrderCalldatas")]
    pub async fn generate_deposit_and_add_order_calldatas(
        &mut self,
    ) -> Result<DepositAndAddOrderCalldataResult, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        self.check_field_values()?;
        self.populate_vault_ids(&deployment)?;
        self.update_bindings(&deployment)?;

        let mut calls = Vec::new();

        let deposit_calldatas = self.generate_deposit_calldatas().await?;

        let deposit_calldatas = match deposit_calldatas {
            DepositCalldataResult::Calldatas(calldatas) => calldatas,
            DepositCalldataResult::NoDeposits => Vec::new(),
        };

        let add_order_calldata = self.generate_add_order_calldata().await?;

        calls.push(Bytes::copy_from_slice(&add_order_calldata.0));

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
        vault_id: Option<String>,
    ) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .update_vault_id(is_input, index, vault_id)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "getVaultIds")]
    pub fn get_vault_ids(&self) -> Result<IOVaultIds, GuiError> {
        let deployment = self.get_current_deployment()?;
        let map = HashMap::from([
            (
                "input".to_string(),
                deployment
                    .deployment
                    .order
                    .inputs
                    .iter()
                    .map(|input| input.vault_id.clone())
                    .collect(),
            ),
            (
                "output".to_string(),
                deployment
                    .deployment
                    .order
                    .outputs
                    .iter()
                    .map(|output| output.vault_id.clone())
                    .collect(),
            ),
        ]);
        Ok(IOVaultIds(map))
    }

    #[wasm_bindgen(js_name = "hasAnyVaultId")]
    pub fn has_any_vault_id(&self) -> Result<bool, GuiError> {
        let map = self.get_vault_ids()?;
        Ok(map.0.values().any(|ids| ids.iter().any(|id| id.is_some())))
    }

    #[wasm_bindgen(js_name = "updateScenarioBindings")]
    pub fn update_scenario_bindings(&mut self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.update_bindings(&deployment)?;
        Ok(())
    }
}
