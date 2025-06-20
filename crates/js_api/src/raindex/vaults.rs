use super::*;
use crate::subgraph::vault::{
    GetVaultBalanceChangesResult, GetVaultsResult, VaultAllowanceResult, VaultCalldataResult,
};
use rain_orderbook_subgraph_client::types::common::{
    SgVault, SgVaultWithSubgraphName, SgVaultsListFilterArgs,
};
use rain_orderbook_subgraph_client::SgPaginationArgs;
use wasm_bindgen_utils::wasm_export;

const DEFAULT_VAULT_PAGE_SIZE: u16 = 50;

#[wasm_export]
impl RaindexClient {
    /// Fetches vaults with a given network or all networks.
    ///
    /// This method wraps the original [`get_vaults`](crate::subgraph::vault::get_vaults) function,
    /// automatically resolving the appropriate subgraph URLs based on the provided chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Optional chain ID. If present, queries that specific network.
    ///   If not present, queries all configured networks.
    /// * `filter_args` - Optional filtering criteria (owners, hide_zero_balance)
    /// * `page` - Optional page number (1-based, defaults to 1)
    ///
    /// # Returns
    ///
    /// * `Ok(GetVaultsResult)` - Array of vaults with network information
    /// * `Err(RaindexError)` - Configuration or network errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Query all networks
    /// const allVaults = await client.getVaults();
    ///
    /// // Query specific network
    /// const polygonVaults = await client.getVaults(137);
    ///
    /// // With filtering
    /// const myVaults = await client.getVaults(137, {
    ///   owners: ["0x1234..."],
    ///   hide_zero_balance: true
    /// });
    /// ```
    #[wasm_export(js_name = "getVaults", unchecked_return_type = "GetVaultsResult")]
    pub async fn get_vaults(
        &self,
        chain_id: Option<u64>,
        filter_args: Option<SgVaultsListFilterArgs>,
        page: Option<u16>,
    ) -> Result<GetVaultsResult, RaindexError> {
        let multi_subgraph_args = self.get_multi_subgraph_args(chain_id)?;
        Ok(crate::subgraph::vault::get_vaults(
            multi_subgraph_args,
            filter_args.unwrap_or(SgVaultsListFilterArgs {
                owners: vec![],
                hide_zero_balance: false,
            }),
            SgPaginationArgs {
                page: page.unwrap_or(1),
                page_size: DEFAULT_VAULT_PAGE_SIZE,
            },
        )
        .await?)
    }

    /// Fetches a specific vault by ID with a given network.
    ///
    /// This method wraps the original [`get_vault`](crate::subgraph::vault::get_vault) function,
    /// automatically resolving the subgraph URL from the chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Target network's chain ID
    /// * `vault_id` - Vault identifier
    ///
    /// # Returns
    ///
    /// * `Ok(SgVault)` - Complete vault information
    /// * `Err(RaindexError)` - Vault not found, network, or configuration errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const vault = await client.getVault(
    ///   137,
    ///   "0x1234567890abcdef1234567890abcdef12345678"
    /// );
    /// ```
    #[wasm_export(js_name = "getVault", unchecked_return_type = "SgVault")]
    pub async fn get_vault(
        &self,
        chain_id: u64,
        vault_id: String,
    ) -> Result<SgVault, RaindexError> {
        let subgraph_url = self.get_subgraph_url_for_chain(chain_id)?;
        Ok(crate::subgraph::vault::get_vault(&subgraph_url, &vault_id).await?)
    }

    /// Fetches balance change history for a vault.
    ///
    /// This method wraps the original [`get_vault_balance_changes`](crate::subgraph::vault::get_vault_balance_changes) function,
    /// automatically resolving the subgraph URL from the chain ID.
    ///
    /// # Parameters
    ///
    /// * `chain_id` - Target network's chain ID
    /// * `vault_id` - Vault identifier
    /// * `page` - Optional page number (1-based, defaults to 1)
    ///
    /// # Returns
    ///
    /// * `Ok(GetVaultBalanceChangesResult)` - Array of balance change events
    /// * `Err(RaindexError)` - Network, query, or configuration errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const changes = await client.getVaultBalanceChanges(
    ///   137,
    ///   "vault_id_123",
    ///   1
    /// );
    /// ```
    #[wasm_export(
        js_name = "getVaultBalanceChanges",
        unchecked_return_type = "GetVaultBalanceChangesResult"
    )]
    pub async fn get_vault_balance_changes(
        &self,
        chain_id: u64,
        vault_id: String,
        page: Option<u16>,
    ) -> Result<GetVaultBalanceChangesResult, RaindexError> {
        let subgraph_url = self.get_subgraph_url_for_chain(chain_id)?;
        Ok(crate::subgraph::vault::get_vault_balance_changes(
            &subgraph_url,
            &vault_id,
            SgPaginationArgs {
                page: page.unwrap_or(1),
                page_size: DEFAULT_VAULT_PAGE_SIZE,
            },
        )
        .await?)
    }

    /// Generates transaction calldata for depositing tokens into a vault.
    ///
    /// This method wraps the original [`get_vault_deposit_calldata`](crate::subgraph::vault::get_vault_deposit_calldata) function.
    ///
    /// # Parameters
    ///
    /// * `vault` - Target vault object
    /// * `deposit_amount` - Amount to deposit in token's smallest unit
    ///
    /// # Returns
    ///
    /// * `Ok(VaultCalldataResult)` - Encoded transaction calldata
    /// * `Err(RaindexError)` - Invalid amount or encoding errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const calldata = await client.getVaultDepositCalldata(
    ///   vault,
    ///   "1000000000000000000"
    /// );
    /// ```
    #[wasm_export(
        js_name = "getVaultDepositCalldata",
        unchecked_return_type = "VaultCalldataResult"
    )]
    pub async fn get_vault_deposit_calldata(
        &self,
        vault: &SgVault,
        deposit_amount: &str,
    ) -> Result<VaultCalldataResult, RaindexError> {
        Ok(crate::subgraph::vault::get_vault_deposit_calldata(vault, deposit_amount).await?)
    }

    /// Generates transaction calldata for withdrawing tokens from a vault.
    ///
    /// This method wraps the original [`get_vault_withdraw_calldata`](crate::subgraph::vault::get_vault_withdraw_calldata) function.
    ///
    /// # Parameters
    ///
    /// * `vault` - Source vault object
    /// * `withdraw_amount` - Amount to withdraw in token's smallest unit
    ///
    /// # Returns
    ///
    /// * `Ok(VaultCalldataResult)` - Encoded transaction calldata
    /// * `Err(RaindexError)` - Invalid amount or encoding errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const calldata = await client.getVaultWithdrawCalldata(
    ///   vault,
    ///   "500000000000000000"
    /// );
    /// ```
    #[wasm_export(
        js_name = "getVaultWithdrawCalldata",
        unchecked_return_type = "VaultCalldataResult"
    )]
    pub async fn get_vault_withdraw_calldata(
        &self,
        vault: &SgVault,
        withdraw_amount: &str,
    ) -> Result<VaultCalldataResult, RaindexError> {
        Ok(crate::subgraph::vault::get_vault_withdraw_calldata(vault, withdraw_amount).await?)
    }

    /// Generates ERC20 approval calldata for vault deposits.
    ///
    /// This method wraps the original [`get_vault_approval_calldata`](crate::subgraph::vault::get_vault_approval_calldata) function.
    ///
    /// # Parameters
    ///
    /// * `rpc_url` - Blockchain RPC endpoint for checking current allowance
    /// * `vault` - Target vault object
    /// * `deposit_amount` - Amount requiring approval
    ///
    /// # Returns
    ///
    /// * `Ok(VaultCalldataResult)` - Encoded approval calldata
    /// * `Err(RaindexError)` - Sufficient allowance exists or other errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const calldata = await client.getVaultApprovalCalldata(
    ///   "https://polygon-rpc.com",
    ///   vault,
    ///   "2000000000000000000"
    /// );
    /// ```
    #[wasm_export(
        js_name = "getVaultApprovalCalldata",
        unchecked_return_type = "VaultCalldataResult"
    )]
    pub async fn get_vault_approval_calldata(
        &self,
        rpc_url: &str,
        vault: &SgVault,
        deposit_amount: &str,
    ) -> Result<VaultCalldataResult, RaindexError> {
        Ok(
            crate::subgraph::vault::get_vault_approval_calldata(rpc_url, vault, deposit_amount)
                .await?,
        )
    }

    /// Checks current ERC20 allowance for a vault.
    ///
    /// This method wraps the original [`check_vault_allowance`](crate::subgraph::vault::check_vault_allowance) function.
    ///
    /// # Parameters
    ///
    /// * `rpc_url` - Blockchain RPC endpoint
    /// * `vault` - Vault to check allowance for
    ///
    /// # Returns
    ///
    /// * `Ok(VaultAllowanceResult)` - Current allowance amount
    /// * `Err(RaindexError)` - Network or contract errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const allowance = await client.checkVaultAllowance(
    ///   "https://polygon-rpc.com",
    ///   vault
    /// );
    /// ```
    #[wasm_export(
        js_name = "checkVaultAllowance",
        unchecked_return_type = "VaultAllowanceResult"
    )]
    pub async fn check_vault_allowance(
        &self,
        rpc_url: &str,
        vault: &SgVault,
    ) -> Result<VaultAllowanceResult, RaindexError> {
        Ok(crate::subgraph::vault::check_vault_allowance(rpc_url, vault).await?)
    }
}
