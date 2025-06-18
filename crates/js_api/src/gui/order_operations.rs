use super::*;
use alloy::{
    primitives::private::rand,
    primitives::{utils::parse_units, Bytes, U256},
    sol_types::SolCall,
};
use rain_orderbook_app_settings::{order::OrderIOCfg, orderbook::OrderbookCfg};
use rain_orderbook_bindings::OrderBook::multicallCall;
use rain_orderbook_common::{
    add_order::AddOrderArgs, deposit::DepositArgs, transaction::TransactionArgs,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};

pub enum CalldataFunction {
    Allowance,
    Deposit,
    AddOrder,
    DepositAndAddOrder,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]

pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}
impl_wasm_traits!(TokenAllowance);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AllowancesResult(Vec<TokenAllowance>);
impl_wasm_traits!(AllowancesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum ApprovalCalldataResult {
    NoDeposits,
    Calldatas(Vec<ApprovalCalldata>),
}
impl_wasm_traits!(ApprovalCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum DepositCalldataResult {
    NoDeposits,
    Calldatas(#[tsify(type = "string[]")] Vec<Bytes>),
}
impl_wasm_traits!(DepositCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AddOrderCalldataResult(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(AddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DepositAndAddOrderCalldataResult(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(DepositAndAddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct IOVaultIds(
    #[tsify(type = "Map<string, (string | undefined)[]>")] pub HashMap<String, Vec<Option<U256>>>,
);
impl_wasm_traits!(IOVaultIds);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WithdrawCalldataResult(#[tsify(type = "string[]")] Vec<Bytes>);
impl_wasm_traits!(WithdrawCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct ExtendedApprovalCalldata {
    #[tsify(type = "string")]
    pub token: Address,
    #[tsify(type = "string")]
    pub calldata: Bytes,
    pub symbol: String,
}
impl_wasm_traits!(ExtendedApprovalCalldata);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentTransactionArgs {
    approvals: Vec<ExtendedApprovalCalldata>,
    #[tsify(type = "string")]
    deployment_calldata: Bytes,
    #[tsify(type = "string")]
    orderbook_address: Address,
    chain_id: u64,
}
impl_wasm_traits!(DeploymentTransactionArgs);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ApprovalCalldata {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub token: Address,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub calldata: Bytes,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ApprovalCalldata);

#[derive(Debug)]
pub struct VaultAndDeposit {
    pub order_io: OrderIOCfg,
    pub deposit_amount: U256,
    pub index: usize,
}

#[wasm_export]
impl DotrainOrderGui {
    fn get_orderbook(&self) -> Result<Arc<OrderbookCfg>, GuiError> {
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
        deployment: &GuiDeploymentCfg,
    ) -> Result<Vec<VaultAndDeposit>, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        let results = deployment
            .deployment
            .order
            .outputs
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(index, output)| {
                output.token.as_ref().and_then(|token| {
                    deposits_map.get(&token.address).map(|amount| {
                        Ok(VaultAndDeposit {
                            order_io: output.clone(),
                            deposit_amount: *amount,
                            index,
                        })
                    })
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

    fn prepare_calldata_generation(
        &mut self,
        calldata_function: CalldataFunction,
    ) -> Result<GuiDeploymentCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        match calldata_function {
            CalldataFunction::Deposit => {
                self.populate_vault_ids(&deployment)?;
            }
            CalldataFunction::AddOrder | CalldataFunction::DepositAndAddOrder => {
                self.check_field_values()?;
                self.populate_vault_ids(&deployment)?;
                self.update_bindings(&deployment)?;
            }
            _ => {}
        }
        self.get_current_deployment()
    }

    /// Checks token allowances for all deposits against the orderbook contract.
    ///
    /// Queries the blockchain to determine current  allowances for each output token that
    /// will be deposited. This helps determine which tokens need approval before
    /// the order can be created.
    ///
    /// # Parameters
    ///
    /// - `owner` - Wallet address to check allowances for
    ///
    /// # Returns
    ///
    /// - `Ok(AllowancesResult)` - Current allowances for all deposit tokens
    /// - `Err(GuiError)` - If allowance check fails
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.checkAllowances(walletAddress);
    /// if (result.error) {
    ///   console.error("Allowance check failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const [allowance1, allowance2, ...] = result.value;
    /// const {
    ///   // token is the token address
    ///   token,
    ///   // allowance is the current allowance for the token
    ///   allowance,
    /// } = allowance1;
    /// ```
    #[wasm_export(
        js_name = "checkAllowances",
        unchecked_return_type = "AllowancesResult"
    )]
    pub async fn check_allowances(&mut self, owner: String) -> Result<AllowancesResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::Allowance)?;

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

    /// Generates approval calldatas for tokens that need increased allowances.
    ///
    /// Automatically checks current allowances and generates approval calldata only
    /// for tokens where the current allowance is insufficient for the planned deposits.
    ///
    /// # Parameters
    ///
    /// - `owner` - Wallet address that will approve the tokens
    ///
    /// # Returns
    ///
    /// - `Ok(ApprovalCalldataResult::NoDeposits)` - No deposits configured
    /// - `Ok(ApprovalCalldataResult::Calldatas(vec))` - Approval calldatas needed
    /// - `Err(GuiError)` - If allowance checks or calldata generation fails
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.generateApprovalCalldatas(walletAddress);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // If there are approvals
    /// const [approval1, approval2, ...] = result.value;
    /// const {
    ///   // token is the token address
    ///   token,
    ///   // calldata is the approval calldata
    ///   calldata,
    /// } = approval1;
    /// ```
    #[wasm_export(
        js_name = "generateApprovalCalldatas",
        unchecked_return_type = "ApprovalCalldataResult"
    )]
    pub async fn generate_approval_calldatas(
        &mut self,
        owner: String,
    ) -> Result<ApprovalCalldataResult, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        if deposits_map.is_empty() {
            return Ok(ApprovalCalldataResult::NoDeposits);
        }

        let transaction_args = self.get_transaction_args()?;

        let mut calldatas = Vec::new();
        for (token_address, deposit_amount) in deposits_map {
            let deposit_args = DepositArgs {
                token: token_address,
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

    fn populate_vault_ids(&mut self, deployment: &GuiDeploymentCfg) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .populate_vault_ids()?;
        Ok(())
    }

    fn update_bindings(&mut self, deployment: &GuiDeploymentCfg) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_scenario(&deployment.deployment.scenario.key)?
            .update_bindings(
                self.field_values
                    .keys()
                    .map(|k| Ok((k.clone(), self.get_field_value(k.clone())?.value.clone())))
                    .collect::<Result<HashMap<String, String>, GuiError>>()?,
            )?;
        Ok(())
    }

    /// Generates calldata for depositing tokens into orderbook vaults.
    ///
    /// Creates deposit calldatas for all configured deposits, automatically
    /// skipping zero amounts and ensuring vault IDs are properly assigned.
    ///
    /// # Returns
    ///
    /// - `Ok(DepositCalldataResult::NoDeposits)` - No deposits configured
    /// - `Ok(DepositCalldataResult::Calldatas(vec))` - Deposit calldatas to execute
    /// - `Err(SelectTokensNotSet)` - Required tokens haven't been selected
    /// - `Err(VaultIdNotFound)` - Vault ID missing for output token
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.generateDepositCalldatas();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // If there are deposits
    /// const [depositCalldata1, depositCalldata2, ...] = result.value;
    /// const {
    ///   // calldata is the deposit calldata
    ///   calldata,
    /// } = depositCalldata1;
    /// ```
    #[wasm_export(
        js_name = "generateDepositCalldatas",
        unchecked_return_type = "DepositCalldataResult"
    )]
    pub async fn generate_deposit_calldatas(&mut self) -> Result<DepositCalldataResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::Deposit)?;

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

    /// Generates calldata for adding the order to the orderbook.
    ///
    /// Creates the addOrder calldata with all field values applied to the
    /// Rainlang code and proper vault configurations.
    ///
    /// # Returns
    ///
    /// - `Ok(AddOrderCalldataResult)` - Encoded addOrder call ready for execution
    /// - `Err(FieldValueNotSet)` - Required field value missing
    /// - `Err(TokenMustBeSelected)` - Required token not selected
    /// - `Err(GuiError)` - Configuration or compilation error
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.generateAddOrderCalldata();
    /// if (result.error) {
    ///   console.error("Cannot create order:", result.error.readableMsg);
    ///   // Show user what needs to be fixed
    ///   return;
    /// }
    /// const addOrderCalldata = result.value;
    /// // Do something with the add order calldata
    /// ```
    #[wasm_export(
        js_name = "generateAddOrderCalldata",
        unchecked_return_type = "AddOrderCalldataResult"
    )]
    pub async fn generate_add_order_calldata(
        &mut self,
    ) -> Result<AddOrderCalldataResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::AddOrder)?;

        let calldata = AddOrderArgs::new_from_deployment(
            self.dotrain_order.dotrain()?,
            deployment.deployment.as_ref().clone(),
        )
        .await?
        .get_add_order_calldata(self.get_transaction_args()?)
        .await?;
        Ok(AddOrderCalldataResult(Bytes::copy_from_slice(&calldata)))
    }

    /// Generates a multicall combining all deposits and add order in one calldata.
    ///
    /// This is the most efficient way to deploy an order, combining all necessary
    /// operations into a single calldata to minimize gas costs and ensure atomicity.
    ///
    /// # Returns
    ///
    /// - `Ok(DepositAndAddOrderCalldataResult)` - Multicall calldata
    /// - `Err(GuiError)` - If any component generation fails
    ///
    /// # Transaction Structure
    ///
    /// The multicall includes:
    /// 1. AddOrder call (always first)
    /// 2. All deposit calls for non-zero amounts
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.generateDepositAndAddOrderCalldatas();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const multicallData = result.value;
    /// // Do something with the multicall data
    /// ```
    #[wasm_export(
        js_name = "generateDepositAndAddOrderCalldatas",
        unchecked_return_type = "DepositAndAddOrderCalldataResult"
    )]
    pub async fn generate_deposit_and_add_order_calldatas(
        &mut self,
    ) -> Result<DepositAndAddOrderCalldataResult, GuiError> {
        self.prepare_calldata_generation(CalldataFunction::DepositAndAddOrder)?;

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

    /// Configures vault IDs for order inputs or outputs.
    ///
    /// Sets the vault ID for a specific input or output token. Vault IDs determine
    /// which vaults are used for the input or output tokens in the order.
    ///
    /// # Parameters
    ///
    /// - `is_input` - True for input vaults, false for output vaults
    /// - `index` - Zero-based index in the inputs/outputs array
    /// - `vault_id` - Vault ID number as string, or None to clear
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Vault ID set successfully
    /// - `Err(GuiError)` - If order configuration is invalid
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result1 = gui.setVaultId(true, 0, "42");
    /// if (result1.error) {
    ///   console.error("Error:", result1.error.readableMsg);
    ///   return;
    /// }
    /// const result2 = gui.setVaultId(false, 0, "43");
    /// const result3 = gui.setVaultId(false, 0, undefined);
    /// ```
    #[wasm_export(js_name = "setVaultId", unchecked_return_type = "void")]
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

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Gets all configured vault IDs for inputs and outputs.
    ///
    /// Returns the current vault ID configuration showing which vaults are
    /// assigned to each input and output token position.
    ///
    /// # Returns
    ///
    /// - `Ok(IOVaultIds)` - Map with 'input' and 'output' arrays of vault IDs
    /// - `Err(GuiError)` - If deployment configuration is invalid
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = gui.getVaultIds();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // key is either 'input' or 'output'
    /// // value is either undefined or the vault ID
    /// for (const [key, value] of result.value) {
    ///   console.log("Key:", key);
    ///   console.log("Value:", value);
    /// }
    /// ```
    #[wasm_export(js_name = "getVaultIds", unchecked_return_type = "IOVaultIds")]
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
                    .map(|input| input.vault_id)
                    .collect(),
            ),
            (
                "output".to_string(),
                deployment
                    .deployment
                    .order
                    .outputs
                    .iter()
                    .map(|output| output.vault_id)
                    .collect(),
            ),
        ]);
        Ok(IOVaultIds(map))
    }

    /// Checks if any vault IDs have been configured.
    ///
    /// Quick validation to determine if vault configuration has started.
    /// Useful for UI state management and validation flows.
    ///
    /// # Returns
    ///
    /// - `Ok(bool)` - True if at least one vault ID is set
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = gui.hasAnyVaultId();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const hasVaults = result.value;
    /// // Do something with the has vaults
    /// ```
    #[wasm_export(js_name = "hasAnyVaultId", unchecked_return_type = "boolean")]
    pub fn has_any_vault_id(&self) -> Result<bool, GuiError> {
        let map = self.get_vault_ids()?;
        Ok(map.0.values().any(|ids| ids.iter().any(|id| id.is_some())))
    }

    #[wasm_export(skip)]
    pub fn update_scenario_bindings(&mut self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.update_bindings(&deployment)?;
        Ok(())
    }

    /// Gets transaction data for order deployment including approvals.
    ///
    /// This is the comprehensive function that provides everything needed to deploy
    /// an order: approval calldatas, the main deployment transaction, and metadata.
    /// Use this for full transaction orchestration.
    ///
    /// # Parameters
    ///
    /// - `owner` - Wallet address that will deploy the order
    ///
    /// # Returns
    ///
    /// - `Ok(DeploymentTransactionArgs)` - Complete transaction package
    /// - `Err(SelectTokensNotSet)` - Required tokens not selected
    /// - `Err(GuiError)` - Configuration or generation error
    ///
    /// # Transaction Package
    ///
    /// - `approvals` - Token approval calldatas with symbols for UI
    /// - `deploymentCalldata` - Main order deployment calldata
    /// - `orderbookAddress` - Target contract address
    /// - `chainId` - Network identifier
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.getDeploymentTransactionArgs(walletAddress);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const {
    ///   // approvals is an array of extended approval calldatas
    ///   // extended approval calldata includes the token address, calldata, and symbol
    ///   approvals,
    ///   // deploymentCalldata is the multicall calldata for the order
    ///   deploymentCalldata,
    ///   // orderbookAddress is the address of the orderbook
    ///   orderbookAddress,
    ///   // chainId is the chain ID of the network
    ///   chainId,
    /// } = result.value;
    /// ```
    #[wasm_export(
        js_name = "getDeploymentTransactionArgs",
        unchecked_return_type = "DeploymentTransactionArgs"
    )]
    pub async fn get_deployment_transaction_args(
        &mut self,
        owner: String,
    ) -> Result<DeploymentTransactionArgs, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::DepositAndAddOrder)?;

        let mut approvals = Vec::new();
        let approval_calldata = self.generate_approval_calldatas(owner).await?;
        if let ApprovalCalldataResult::Calldatas(calldatas) = approval_calldata {
            let mut output_token_infos = HashMap::new();
            for output in deployment.deployment.order.outputs.clone() {
                if output.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = output.token.as_ref().unwrap();
                let token_info = self.get_token_info(token.key.clone()).await?;
                output_token_infos.insert(token.address, token_info);
            }

            for calldata in calldatas.iter() {
                let token_info = output_token_infos
                    .get(&calldata.token)
                    .ok_or(GuiError::TokenNotFound(calldata.token.to_string()))?;
                approvals.push(ExtendedApprovalCalldata {
                    token: calldata.token,
                    calldata: calldata.calldata.clone(),
                    symbol: token_info.symbol.clone(),
                });
            }
        }

        let deposit_and_add_order_calldata =
            self.generate_deposit_and_add_order_calldatas().await?;

        Ok(DeploymentTransactionArgs {
            approvals,
            deployment_calldata: deposit_and_add_order_calldata.0,
            orderbook_address: deployment
                .deployment
                .order
                .orderbook
                .as_ref()
                .ok_or(GuiError::OrderbookNotFound)?
                .address,
            chain_id: deployment.deployment.order.network.chain_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{initialize_gui, initialize_gui_with_select_tokens};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_generate_deposit_calldatas() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(_) => {
                panic!("should not be calldatas");
            }
            DepositCalldataResult::NoDeposits => {}
        }

        gui.save_deposit("token1".to_string(), "1200".to_string())
            .unwrap();

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(calldatas) => {
                assert_eq!(calldatas.len(), 1);
                assert_eq!(calldatas[0].len(), 164);
            }
            DepositCalldataResult::NoDeposits => {
                panic!("should not be no deposits");
            }
        }

        gui.save_deposit("token1".to_string(), "0".to_string())
            .unwrap();

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(calldatas) => {
                assert!(calldatas.is_empty());
            }
            DepositCalldataResult::NoDeposits => {
                panic!("should not be no deposits");
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_missing_select_tokens() {
        let mut gui = initialize_gui_with_select_tokens().await;

        let err = gui
            .check_allowances(Address::random().to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui.generate_deposit_calldatas().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui.generate_add_order_calldata().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui
            .generate_deposit_and_add_order_calldatas()
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );
    }

    #[wasm_bindgen_test]
    async fn test_missing_field_values() {
        let mut gui = initialize_gui(None).await;

        let err = gui.generate_add_order_calldata().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The value for field 'Field 2 name' is required but has not been set."
        );

        let err = gui
            .generate_deposit_and_add_order_calldatas()
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The value for field 'Field 2 name' is required but has not been set."
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_vault_ids() {
        let gui = initialize_gui(None).await;
        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"][0], Some(U256::from(1)));
        assert_eq!(res.0["output"][0], Some(U256::from(1)));

        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"][0], None);
        assert_eq!(res.0["output"][0], None);

        gui.set_vault_id(true, 0, Some("999".to_string())).unwrap();
        gui.set_vault_id(false, 0, Some("888".to_string())).unwrap();

        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"][0], Some(U256::from(999)));
        assert_eq!(res.0["output"][0], Some(U256::from(888)));
    }

    #[wasm_bindgen_test]
    async fn test_has_any_vault_id() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;
        assert!(!gui.has_any_vault_id().unwrap());
        gui.set_vault_id(true, 0, Some("1".to_string())).unwrap();
        assert!(gui.has_any_vault_id().unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_update_scenario_bindings() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let deployment = gui.get_current_deployment().unwrap();
        assert!(!deployment
            .deployment
            .scenario
            .bindings
            .contains_key("binding-1"));
        assert!(!deployment
            .deployment
            .scenario
            .bindings
            .contains_key("binding-2"));

        gui.save_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.save_field_value("binding-2".to_string(), "200".to_string())
            .unwrap();
        gui.update_scenario_bindings().unwrap();

        let deployment = gui.get_current_deployment().unwrap();
        assert_eq!(deployment.deployment.scenario.bindings["binding-1"], "100");
        assert_eq!(deployment.deployment.scenario.bindings["binding-2"], "200");
    }
}
