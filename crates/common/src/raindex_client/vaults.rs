use super::*;
use crate::{
    deposit::DepositArgs,
    raindex_client::{orders::RaindexOrderAsIO, transactions::RaindexTransaction},
    transaction::TransactionArgs,
    withdraw::WithdrawArgs,
};
use alloy::primitives::{Address, Bytes, I256, U256};
use rain_orderbook_subgraph_client::{
    types::{
        common::{
            SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgTradeVaultBalanceChange,
            SgVault, SgVaultBalanceChangeUnwrapped, SgVaultsListFilterArgs,
        },
        Id,
    },
    MultiOrderbookSubgraphClient, OrderbookSubgraphClient, SgPaginationArgs,
};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_PAGE_SIZE: u16 = 100;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum RaindexVaultType {
    Input,
    Output,
    InputOutput,
}
impl_wasm_traits!(RaindexVaultType);

/// Represents a vault with balance and token information within a given orderbook.
///
/// A vault is a fundamental component that holds tokens and participates in order execution.
/// Each vault has a unique identifier, current balance, associated token metadata, and
/// belongs to a specific orderbook contract on the blockchain.
///
/// Vaults can serve different roles in relation to orders - they can provide tokens (input),
/// receive tokens (output), or both (input/output), depending on the trading strategy.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVault {
    raindex_client: Arc<RwLock<RaindexClient>>,
    chain_id: u32,
    vault_type: Option<RaindexVaultType>,
    id: String,
    owner: Address,
    vault_id: U256,
    balance: U256,
    token: RaindexVaultToken,
    orderbook: Address,
    orders_as_inputs: Vec<RaindexOrderAsIO>,
    orders_as_outputs: Vec<RaindexOrderAsIO>,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVault {
    #[cfg(target_family = "wasm")]
    fn u256_to_bigint(value: U256) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&value.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }

    #[wasm_bindgen(getter = chainId)]
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    #[wasm_bindgen(getter = vaultType)]
    pub fn vault_type(&self) -> Option<RaindexVaultType> {
        self.vault_type.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn owner(&self) -> String {
        self.owner.to_string()
    }
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> Result<BigInt, RaindexError> {
        Self::u256_to_bigint(self.vault_id)
    }
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> Result<BigInt, RaindexError> {
        Self::u256_to_bigint(self.balance)
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
    #[wasm_bindgen(getter = ordersAsInput)]
    pub fn orders_as_inputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_inputs.clone()
    }
    #[wasm_bindgen(getter = ordersAsOutput)]
    pub fn orders_as_outputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_outputs.clone()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVault {
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    pub fn vault_type(&self) -> Option<RaindexVaultType> {
        self.vault_type.clone()
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn owner(&self) -> Address {
        self.owner
    }
    pub fn vault_id(&self) -> U256 {
        self.vault_id
    }
    pub fn balance(&self) -> U256 {
        self.balance
    }
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
    pub fn orders_as_inputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_inputs.clone()
    }
    pub fn orders_as_outputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_outputs.clone()
    }
}

/// Token metadata associated with a vault in the Raindex system.
///
/// Contains comprehensive information about the ERC20 token held within a vault,
/// including contract address, human-readable identifiers, and decimal precision.
/// Some fields may be optional as they depend on the token's implementation and
/// whether the metadata has been successfully retrieved from the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultToken {
    id: String,
    address: Address,
    name: Option<String>,
    symbol: Option<String>,
    decimals: Option<U256>,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultToken {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn address(&self) -> String {
        self.address.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> Result<Option<BigInt>, RaindexError> {
        self.decimals
            .map(|decimals| {
                BigInt::from_str(&decimals.to_string())
                    .map_err(|e| RaindexError::JsError(e.to_string().into()))
            })
            .transpose()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVaultToken {
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn address(&self) -> Address {
        self.address
    }
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
    pub fn decimals(&self) -> Option<U256> {
        self.decimals
    }
}

#[wasm_export]
impl RaindexVault {
    #[wasm_export(skip)]
    pub fn get_orderbook_client(&self) -> Result<OrderbookSubgraphClient, RaindexError> {
        let raindex_client = self
            .raindex_client
            .read()
            .map_err(|_| YamlError::ReadLockError)?;
        raindex_client.get_orderbook_client(self.chain_id, self.orderbook.to_string())
    }

    /// Fetches balance change history for a vault
    ///
    /// Retrieves chronological list of deposits, withdrawals, and trades affecting
    /// a vault's balance.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getBalanceChanges();
    /// if (result.error) {
    ///   console.error("Error fetching history:", result.error.readableMsg);
    ///   return;
    /// }
    /// const changes = result.value;
    /// // Do something with the changes
    /// ```
    #[wasm_export(
        js_name = "getBalanceChanges",
        return_description = "Array of balance change events",
        unchecked_return_type = "RaindexVaultBalanceChange[]",
        preserve_js_class
    )]
    pub async fn get_balance_changes(
        &self,
        #[wasm_export(param_description = "Optional page number (default to 1)")] page: Option<u16>,
    ) -> Result<Vec<RaindexVaultBalanceChange>, RaindexError> {
        let client = self.get_orderbook_client()?;
        let balance_changes = client
            .vault_balance_changes_list(
                Id::new(self.id.clone()),
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: 1000,
                },
            )
            .await?;
        let balance_changes = balance_changes
            .into_iter()
            .map(|balance_change| balance_change.try_into())
            .collect::<Result<Vec<RaindexVaultBalanceChange>, RaindexError>>()?;
        Ok(balance_changes)
    }

    fn validate_amount(&self, amount: &str) -> Result<U256, RaindexError> {
        let amount = U256::from_str(amount)?;
        if amount == U256::ZERO {
            return Err(RaindexError::ZeroAmount);
        }
        Ok(amount)
    }

    /// Generates transaction calldata for depositing tokens into a vault
    ///
    /// Creates the contract calldata needed to deposit a specified amount of tokens
    /// into a vault.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getDepositCalldata(
    ///   vault,
    ///   "1000000000000000000"
    /// );
    /// if (result.error) {
    ///   console.error("Cannot generate deposit:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getDepositCalldata",
        return_description = "Encoded transaction calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_deposit_calldata(
        &self,
        #[wasm_export(
            param_description = "Amount to deposit in token's smallest unit (e.g., \"1000000000000000000\" for 1 token with 18 decimals)"
        )]
        amount: String,
    ) -> Result<Bytes, RaindexError> {
        let amount = self.validate_amount(&amount)?;
        Ok(Bytes::copy_from_slice(
            &DepositArgs {
                token: self.token.address,
                vault_id: self.vault_id,
                amount,
            }
            .get_deposit_calldata()
            .await?,
        ))
    }

    /// Generates transaction calldata for withdrawing tokens from a vault
    ///
    /// Creates the contract calldata needed to withdraw a specified amount of tokens
    /// from a vault.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getWithdrawCalldata(
    ///   "500000000000000000"
    /// );
    /// if (result.error) {
    ///   console.error("Cannot generate withdrawal:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getWithdrawCalldata",
        return_description = "Encoded transaction calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_withdraw_calldata(
        &self,
        #[wasm_export(param_description = "Amount to withdraw")] amount: String,
    ) -> Result<Bytes, RaindexError> {
        let amount = self.validate_amount(&amount)?;
        Ok(Bytes::copy_from_slice(
            &WithdrawArgs {
                token: self.token.address,
                vault_id: self.vault_id,
                target_amount: amount,
            }
            .get_withdraw_calldata()
            .await?,
        ))
    }

    fn get_deposit_and_transaction_args(
        &self,
        amount: U256,
    ) -> Result<(DepositArgs, TransactionArgs), RaindexError> {
        let rpc_url = {
            let raindex_client = self
                .raindex_client
                .read()
                .map_err(|_| YamlError::ReadLockError)?;
            raindex_client.get_rpc_url_for_chain(self.chain_id)?
        };
        let deposit_args = DepositArgs {
            token: self.token.address,
            vault_id: self.vault_id,
            amount,
        };
        let transaction_args = TransactionArgs {
            orderbook_address: self.orderbook,
            rpc_url: rpc_url.to_string(),
            ..Default::default()
        };
        Ok((deposit_args, transaction_args))
    }

    /// Generates ERC20 approval calldata for vault deposits
    ///
    /// Creates the contract calldata needed to approve the orderbook contract to spend
    /// tokens for a vault deposit, but only if additional approval is needed.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getApprovalCalldata(
    ///   "2000000000000000000"
    /// );
    /// if (result.error) {
    ///   console.error("Approval error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getApprovalCalldata",
        return_description = "Encoded approval calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_approval_calldata(
        &self,
        #[wasm_export(param_description = "Amount requiring approval")] amount: String,
    ) -> Result<Bytes, RaindexError> {
        let amount = self.validate_amount(&amount)?;

        let (deposit_args, transaction_args) = self.get_deposit_and_transaction_args(amount)?;

        let allowance = deposit_args
            .read_allowance(self.owner, transaction_args.clone())
            .await?;
        if allowance >= amount {
            return Err(RaindexError::ExistingAllowance);
        }

        Ok(Bytes::copy_from_slice(
            &deposit_args.get_approve_calldata(transaction_args).await?,
        ))
    }

    /// Gets the current ERC20 allowance for a vault
    ///
    /// Determines how much the orderbook contract is currently approved to spend
    /// on behalf of the vault owner.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getAllowance();
    /// if (result.error) {
    ///   console.error("Cannot check allowance:", result.error.readableMsg);
    ///   return;
    /// }
    /// const allowance = result.value;
    /// // Do something with the allowance
    /// ```
    #[wasm_export(
        js_name = "getAllowance",
        return_description = "Current allowance amount"
    )]
    pub async fn get_allowance(&self) -> Result<RaindexVaultAllowance, RaindexError> {
        let (deposit_args, transaction_args) = self.get_deposit_and_transaction_args(U256::ZERO)?;
        let allowance = deposit_args
            .read_allowance(self.owner, transaction_args.clone())
            .await?;
        Ok(RaindexVaultAllowance(allowance))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultBalanceChange {
    __typename: String,
    vault_id: U256,
    token: RaindexVaultToken,
    amount: I256,
    new_balance: U256,
    old_balance: U256,
    timestamp: U256,
    transaction: RaindexTransaction,
    orderbook: Address,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultBalanceChange {
    #[wasm_bindgen(getter)]
    pub fn __typename(&self) -> String {
        self.__typename.clone()
    }
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.vault_id.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.amount.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter = newBalance)]
    pub fn new_balance(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.new_balance.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter = oldBalance)]
    pub fn old_balance(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.old_balance.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVaultBalanceChange {
    pub fn __typename(&self) -> String {
        self.__typename.clone()
    }
    pub fn vault_id(&self) -> U256 {
        self.vault_id
    }
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    pub fn amount(&self) -> I256 {
        self.amount
    }
    pub fn new_balance(&self) -> U256 {
        self.new_balance
    }
    pub fn old_balance(&self) -> U256 {
        self.old_balance
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct RaindexVaultAllowance(#[tsify(type = "string")] U256);
impl_wasm_traits!(RaindexVaultAllowance);

impl TryFrom<SgVaultBalanceChangeUnwrapped> for RaindexVaultBalanceChange {
    type Error = RaindexError;
    fn try_from(balance_change: SgVaultBalanceChangeUnwrapped) -> Result<Self, Self::Error> {
        Ok(Self {
            __typename: balance_change.__typename,
            vault_id: U256::from_str(&balance_change.vault.vault_id.0)?,
            token: RaindexVaultToken::try_from(balance_change.vault.token)?,
            amount: I256::from_str(&balance_change.amount.0)?,
            new_balance: U256::from_str(&balance_change.new_vault_balance.0)?,
            old_balance: U256::from_str(&balance_change.old_vault_balance.0)?,
            timestamp: U256::from_str(&balance_change.timestamp.0)?,
            transaction: RaindexTransaction::try_from(balance_change.transaction)?,
            orderbook: Address::from_str(&balance_change.orderbook.id.0)?,
        })
    }
}

impl TryFrom<SgTradeVaultBalanceChange> for RaindexVaultBalanceChange {
    type Error = RaindexError;
    fn try_from(balance_change: SgTradeVaultBalanceChange) -> Result<Self, Self::Error> {
        Ok(Self {
            __typename: balance_change.__typename,
            vault_id: U256::from_str(&balance_change.vault.vault_id.0)?,
            token: RaindexVaultToken::try_from(balance_change.vault.token)?,
            amount: I256::from_str(&balance_change.amount.0)?,
            new_balance: U256::from_str(&balance_change.new_vault_balance.0)?,
            old_balance: U256::from_str(&balance_change.old_vault_balance.0)?,
            timestamp: U256::from_str(&balance_change.timestamp.0)?,
            transaction: RaindexTransaction::try_from(balance_change.transaction)?,
            orderbook: Address::from_str(&balance_change.orderbook.id.0)?,
        })
    }
}

#[wasm_export]
impl RaindexClient {
    /// Fetches vault data from multiple subgraphs across different networks
    ///
    /// Queries multiple subgraphs simultaneously to retrieve vault information
    /// across different blockchain networks.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getVaults(
    ///   {
    ///     owners: ["0x1234567890abcdef1234567890abcdef12345678"],
    ///     hide_zero_balance: true
    ///   },
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching vaults:", result.error.readableMsg);
    ///   return;
    /// }
    /// const vaults = result.value;
    /// // Do something with the vaults
    /// ```
    #[wasm_export(
        js_name = "getVaults",
        return_description = "Array of vaults with their associated subgraph network names",
        unchecked_return_type = "RaindexVault[]",
        preserve_js_class
    )]
    pub async fn get_vaults(
        &self,
        #[wasm_export(param_description = "Specific blockchain networks to query (optional)")]
        chain_ids: Option<ChainIds>,
        #[wasm_export(
            param_description = "Optional filtering options including owners and hide_zero_balance"
        )]
        filters: Option<GetVaultsFilters>,
        #[wasm_export(param_description = "Optional page number (defaults to 1)")] page: Option<
            u16,
        >,
    ) -> Result<Vec<RaindexVault>, RaindexError> {
        let raindex_client = Arc::new(RwLock::new(self.clone()));
        let multi_subgraph_args =
            self.get_multi_subgraph_args(chain_ids.map(|ids| ids.0.to_vec()))?;
        let client = MultiOrderbookSubgraphClient::new(
            multi_subgraph_args.values().flatten().cloned().collect(),
        );

        let vaults = client
            .vaults_list(
                filters
                    .unwrap_or(GetVaultsFilters {
                        owners: vec![],
                        hide_zero_balance: false,
                    })
                    .try_into()?,
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: DEFAULT_PAGE_SIZE,
                },
            )
            .await;
        let vaults = vaults
            .iter()
            .map(|vault| {
                let chain_id = multi_subgraph_args
                    .iter()
                    .find(|(_, args)| args.iter().any(|arg| arg.name == vault.subgraph_name))
                    .map(|(chain_id, _)| *chain_id)
                    .unwrap();
                let vault = RaindexVault::try_from_sg_vault(
                    raindex_client.clone(),
                    chain_id,
                    vault.vault.clone(),
                    None,
                )?;
                Ok(vault)
            })
            .collect::<Result<Vec<RaindexVault>, RaindexError>>()?;
        Ok(vaults)
    }

    /// Fetches detailed information for a specific vault
    ///
    /// Retrieves complete vault information including token details, balance, etc.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getVault(
    ///   137, // Polygon network
    ///   "0x1234567890abcdef1234567890abcdef12345678"
    /// );
    /// if (result.error) {
    ///   console.error("Vault not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const vault = result.value;
    /// // Do something with the vault
    /// ```
    #[wasm_export(
        js_name = "getVault",
        return_description = "Complete vault information",
        unchecked_return_type = "RaindexVault",
        preserve_js_class
    )]
    pub async fn get_vault(
        &self,
        #[wasm_export(param_description = "Chain ID of the network the vault is on")] chain_id: u32,
        #[wasm_export(param_description = "Orderbook contract address")] orderbook_address: String,
        #[wasm_export(param_description = "Unique vault identifier")] vault_id: String,
    ) -> Result<RaindexVault, RaindexError> {
        let client = self.get_orderbook_client(chain_id, orderbook_address)?;
        let vault = RaindexVault::try_from_sg_vault(
            Arc::new(RwLock::new(self.clone())),
            chain_id,
            client.vault_detail(Id::new(vault_id)).await?,
            None,
        )?;
        Ok(vault)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GetVaultsFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    pub hide_zero_balance: bool,
}
impl_wasm_traits!(GetVaultsFilters);

impl TryFrom<GetVaultsFilters> for SgVaultsListFilterArgs {
    type Error = RaindexError;
    fn try_from(filters: GetVaultsFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            owners: filters
                .owners
                .into_iter()
                .map(|owner| SgBytes(owner.to_string()))
                .collect(),
            hide_zero_balance: filters.hide_zero_balance,
        })
    }
}

impl RaindexVault {
    pub fn try_from_sg_vault(
        raindex_client: Arc<RwLock<RaindexClient>>,
        chain_id: u32,
        vault: SgVault,
        vault_type: Option<RaindexVaultType>,
    ) -> Result<Self, RaindexError> {
        Ok(Self {
            raindex_client,
            chain_id,
            vault_type,
            id: vault.id.0,
            owner: Address::from_str(&vault.owner.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            balance: U256::from_str(&vault.balance.0)?,
            token: vault.token.try_into()?,
            orderbook: Address::from_str(&vault.orderbook.id.0)?,
            orders_as_inputs: vault
                .orders_as_input
                .iter()
                .map(|order| RaindexOrderAsIO::try_from(order.clone()))
                .collect::<Result<Vec<RaindexOrderAsIO>, RaindexError>>()?,
            orders_as_outputs: vault
                .orders_as_output
                .iter()
                .map(|order| RaindexOrderAsIO::try_from(order.clone()))
                .collect::<Result<Vec<RaindexOrderAsIO>, RaindexError>>()?,
        })
    }

    pub fn with_vault_type(&self, vault_type: RaindexVaultType) -> Self {
        Self {
            raindex_client: self.raindex_client.clone(),
            chain_id: self.chain_id,
            vault_type: Some(vault_type),
            id: self.id.clone(),
            owner: self.owner,
            vault_id: self.vault_id,
            balance: self.balance,
            token: self.token.clone(),
            orderbook: self.orderbook,
            orders_as_inputs: self.orders_as_inputs.clone(),
            orders_as_outputs: self.orders_as_outputs.clone(),
        }
    }

    pub fn into_sg_vault(self) -> Result<SgVault, RaindexError> {
        Ok(SgVault {
            id: SgBytes(self.id),
            vault_id: SgBigInt(self.vault_id.to_string()),
            balance: SgBigInt(self.balance.to_string()),
            owner: SgBytes(self.owner.to_string()),
            token: self.token.try_into()?,
            orderbook: SgOrderbook {
                id: SgBytes(self.orderbook.to_string()),
            },
            orders_as_input: self
                .orders_as_inputs
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<SgOrderAsIO>, RaindexError>>()?,
            orders_as_output: self
                .orders_as_outputs
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<SgOrderAsIO>, RaindexError>>()?,
            balance_changes: vec![],
        })
    }
}

impl TryFrom<SgErc20> for RaindexVaultToken {
    type Error = RaindexError;
    fn try_from(erc20: SgErc20) -> Result<Self, Self::Error> {
        Ok(Self {
            id: erc20.id.0,
            address: Address::from_str(&erc20.address.0)?,
            name: erc20.name,
            symbol: erc20.symbol,
            decimals: erc20
                .decimals
                .map(|decimals| U256::from_str(&decimals.0))
                .transpose()?,
        })
    }
}
impl TryFrom<RaindexVaultToken> for SgErc20 {
    type Error = RaindexError;
    fn try_from(token: RaindexVaultToken) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SgBytes(token.id),
            address: SgBytes(token.address.to_string()),
            name: token.name,
            symbol: token.symbol,
            decimals: token
                .decimals
                .map(|decimals| SgBigInt(decimals.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::tests::CHAIN_ID_1_ORDERBOOK_ADDRESS;
        use alloy::sol_types::SolCall;
        use alloy_ethers_typecast::rpc::Response;
        use httpmock::MockServer;
        use rain_orderbook_bindings::{
            IOrderBookV4::{deposit2Call, withdraw2Call},
            IERC20::approveCall,
        };
        use serde_json::{json, Value};

        fn get_vault1_json() -> Value {
            json!({
              "id": "vault1",
              "owner": "0x0000000000000000000000000000000000000000",
              "vaultId": "0x10",
              "balance": "0x10",
              "token": {
                "id": "token1",
                "address": "0x0000000000000000000000000000000000000000",
                "name": "Token 1",
                "symbol": "TKN1",
                "decimals": "18"
              },
              "orderbook": {
                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
              },
              "ordersAsOutput": [],
              "ordersAsInput": [],
              "balanceChanges": []
            })
        }
        fn get_vault2_json() -> Value {
            json!({
                "id": "vault2",
                "owner": "0x0000000000000000000000000000000000000000",
                "vaultId": "0x20",
                "balance": "0x20",
                "token": {
                    "id": "token2",
                    "address": "0x0000000000000000000000000000000000000000",
                    "name": "Token 2",
                    "symbol": "TKN2",
                    "decimals": "18"
                },
                "orderbook": {
                    "id": "0x0000000000000000000000000000000000000000"
                },
                "ordersAsOutput": [],
                "ordersAsInput": [],
                "balanceChanges": []
            })
        }
        #[tokio::test]
        async fn test_get_vaults() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault1_json()]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault2_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let result = raindex_client.get_vaults(None, None, None).await.unwrap();
            assert_eq!(result.len(), 2);

            let vault1 = result[0].clone();
            assert_eq!(vault1.chain_id, 1);
            assert_eq!(vault1.id, "vault1");
            assert_eq!(
                vault1.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault1.vault_id, U256::from_str("0x10").unwrap());
            assert_eq!(vault1.balance, U256::from_str("0x10").unwrap());
            assert_eq!(vault1.token.id, "token1");
            assert_eq!(
                vault1.orderbook,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );

            let vault2 = result[1].clone();
            assert_eq!(vault2.chain_id, 137);
            assert_eq!(vault2.id, "vault2");
            assert_eq!(
                vault2.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault2.vault_id, U256::from_str("0x20").unwrap());
            assert_eq!(vault2.balance, U256::from_str("0x20").unwrap());
            assert_eq!(vault2.token.id, "token2");
            assert_eq!(
                vault2.orderbook,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vault() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            assert_eq!(vault.chain_id, 1);
            assert_eq!(vault.id, "vault1");
            assert_eq!(
                vault.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault.vault_id, U256::from_str("0x10").unwrap());
            assert_eq!(vault.balance, U256::from_str("0x10").unwrap());
            assert_eq!(vault.token.id, "token1");
            assert_eq!(
                vault.orderbook,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vault_balance_changes() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": [
                            {
                                "__typename": "Deposit",
                                "amount": "5000000000000000000",
                                "newVaultBalance": "5000000000000000000",
                                "oldVaultBalance": "0",
                                "vault": {
                                    "id": "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902",
                                    "vaultId": "1",
                                    "token": {
                                        "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "name": "Wrapped Flare",
                                        "symbol": "WFLR",
                                        "decimals": "18"
                                    }
                                },
                                "timestamp": "1734054063",
                                "transaction": {
                                    "id": "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22",
                                    "from": "0x7177b9d00bb5dbcaaf069cc63190902763783b09",
                                    "blockNumber": "34407047",
                                    "timestamp": "1734054063"
                                },
                                "orderbook": {
                                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                                }
                            }
                        ]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": { "vaultBalanceChanges": [] }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            let result = vault.get_balance_changes(None).await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].__typename, "Deposit");
            assert_eq!(result[0].vault_id, U256::from_str("1").unwrap());
            assert_eq!(
                result[0].token.id,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                result[0].token.address,
                Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
            );
            assert_eq!(result[0].token.name, Some("Wrapped Flare".to_string()));
            assert_eq!(result[0].token.symbol, Some("WFLR".to_string()));
            assert_eq!(result[0].token.decimals, Some(U256::from(18)));
            assert_eq!(
                result[0].amount,
                I256::from_str("5000000000000000000").unwrap()
            );
            assert_eq!(
                result[0].new_balance,
                U256::from_str("5000000000000000000").unwrap()
            );
            assert_eq!(result[0].old_balance, U256::from_str("0").unwrap());
            assert_eq!(result[0].timestamp, U256::from_str("1734054063").unwrap());
            assert_eq!(
                result[0].transaction.id(),
                "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22"
            );
            assert_eq!(
                result[0].transaction.from(),
                Address::from_str("0x7177b9d00bB5dbcaaF069CC63190902763783b09").unwrap()
            );
            assert_eq!(result[0].transaction.block_number(), U256::from(34407047));
            assert_eq!(result[0].transaction.timestamp(), U256::from(1734054063));
            assert_eq!(
                result[0].orderbook,
                Address::from_str("0xcee8cd002f151a536394e564b84076c41bbbcd4d").unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vault_deposit_calldata() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            let result = vault.get_deposit_calldata("500".to_string()).await.unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &deposit2Call {
                        token: Address::from_str("0x0000000000000000000000000000000000000000")
                            .unwrap(),
                        vaultId: U256::from_str("0x10").unwrap(),
                        amount: U256::from_str("500").unwrap(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = vault
                .get_deposit_calldata("0".to_string())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());
        }

        #[tokio::test]
        async fn test_get_vault_withdraw_calldata() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            let result = vault
                .get_withdraw_calldata("500".to_string())
                .await
                .unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &withdraw2Call {
                        token: Address::from_str("0x0000000000000000000000000000000000000000")
                            .unwrap(),
                        vaultId: U256::from_str("0x10").unwrap(),
                        targetAmount: U256::from_str("500").unwrap(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = vault
                .get_withdraw_calldata("0".to_string())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());
        }

        #[tokio::test]
        async fn test_get_vault_approval_calldata() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).body(
                    Response::new_success(
                        1,
                        "0x0000000000000000000000000000000000000000000000000000000000000064",
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &rpc_server.url("/rpc1"),
                    &rpc_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            let result = vault
                .get_approval_calldata("600".to_string())
                .await
                .unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &approveCall {
                        spender: Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                        amount: U256::from(600),
                    }
                    .abi_encode(),
                )
            );

            let err = vault
                .get_approval_calldata("0".to_string())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());

            let err = vault
                .get_approval_calldata("90".to_string())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ExistingAllowance.to_string());

            let err = vault
                .get_approval_calldata("100".to_string())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ExistingAllowance.to_string());
        }

        #[tokio::test]
        async fn test_check_vault_allowance() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).body(
                    Response::new_success(
                        1,
                        "0x0000000000000000000000000000000000000000000000000000000000000001",
                    )
                    .to_json_string()
                    .unwrap(),
                );
            });

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &rpc_server.url("/rpc1"),
                    &rpc_server.url("/rpc2"),
                )],
                None,
            )
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    1,
                    CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    "vault1".to_string(),
                )
                .await
                .unwrap();
            let result = vault.get_allowance().await.unwrap();
            assert_eq!(result.0, U256::from(1));
        }
    }
}
