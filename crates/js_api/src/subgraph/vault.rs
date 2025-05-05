use super::SubgraphError;
use alloy::primitives::{Address, Bytes, U256};
use cynic::Id;
use rain_orderbook_common::deposit::DepositArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;
use rain_orderbook_subgraph_client::types::common::{
    SgOrder, SgVault, SgVaultBalanceChangeUnwrapped, SgVaultWithSubgraphName,
    SgVaultsListFilterArgs,
};
use rain_orderbook_subgraph_client::{
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient, SgPaginationArgs,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct VaultCalldataResult(#[tsify(type = "string")] Bytes);
impl_wasm_traits!(VaultCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetVaultsResult(
    #[tsify(type = "SgVaultWithSubgraphName[]")] Vec<SgVaultWithSubgraphName>,
);
impl_wasm_traits!(GetVaultsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetVaultBalanceChangesResult(
    #[tsify(type = "SgVaultBalanceChangeUnwrapped[]")] Vec<SgVaultBalanceChangeUnwrapped>,
);
impl_wasm_traits!(GetVaultBalanceChangesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct VaultAllowanceResult(#[tsify(type = "string")] U256);
impl_wasm_traits!(VaultAllowanceResult);

/// Fetch all vaults from multiple subgraphs
/// Returns a list of VaultWithSubgraphName structs
#[wasm_export(js_name = "getVaults", unchecked_return_type = "GetVaultsResult")]
pub async fn get_vaults(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: SgVaultsListFilterArgs,
    pagination_args: SgPaginationArgs,
) -> Result<GetVaultsResult, SubgraphError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    Ok(GetVaultsResult(
        client.vaults_list(filter_args, pagination_args).await?,
    ))
}

/// Fetch a single vault
/// Returns the SgVault struct
#[wasm_export(js_name = "getVault", unchecked_return_type = "SgVault")]
pub async fn get_vault(url: &str, id: &str) -> Result<SgVault, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(client.vault_detail(Id::new(id)).await?)
}

/// Fetch balance changes for a vault
/// Returns a list of VaultBalanceChangeUnwrapped structs
#[wasm_export(
    js_name = "getVaultBalanceChanges",
    unchecked_return_type = "GetVaultBalanceChangesResult"
)]
pub async fn get_vault_balance_changes(
    url: &str,
    id: &str,
    pagination_args: SgPaginationArgs,
) -> Result<GetVaultBalanceChangesResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(GetVaultBalanceChangesResult(
        client
            .vault_balance_changes_list(Id::new(id), pagination_args)
            .await?,
    ))
}

/// Get deposit calldata for a vault
/// Returns a string of the calldata
#[wasm_export(
    js_name = "getVaultDepositCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_deposit_calldata(
    vault: &SgVault,
    deposit_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let deposit_amount = validate_amount(deposit_amount)?;

    let deposit_args = DepositArgs {
        token: Address::from_str(&vault.token.address.0)?,
        vault_id: U256::from_str(&vault.vault_id.0)?,
        amount: deposit_amount,
    };

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &deposit_args.get_deposit_calldata().await?,
    )))
}

/// Get withdraw calldata for a vault
#[wasm_export(
    js_name = "getVaultWithdrawCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_withdraw_calldata(
    vault: &SgVault,
    withdraw_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let withdraw_amount = validate_amount(withdraw_amount)?;

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &WithdrawArgs {
            token: Address::from_str(&vault.token.address.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            target_amount: withdraw_amount,
        }
        .get_withdraw_calldata()
        .await?,
    )))
}

#[wasm_export(
    js_name = "getVaultApprovalCalldata",
    unchecked_return_type = "VaultCalldataResult"
)]
pub async fn get_vault_approval_calldata(
    rpc_url: &str,
    vault: &SgVault,
    deposit_amount: &str,
) -> Result<VaultCalldataResult, SubgraphError> {
    let deposit_amount = validate_amount(deposit_amount)?;
    let owner = Address::from_str(&vault.owner.0)?;

    let (deposit_args, transaction_args) =
        get_deposit_and_transaction_args(rpc_url, &vault, deposit_amount)?;

    let allowance = deposit_args
        .read_allowance(owner, transaction_args.clone())
        .await?;
    if allowance > deposit_amount {
        return Err(SubgraphError::InvalidAmount);
    }

    Ok(VaultCalldataResult(Bytes::copy_from_slice(
        &deposit_args.get_approve_calldata(transaction_args).await?,
    )))
}

#[wasm_export(
    js_name = "checkVaultAllowance",
    unchecked_return_type = "VaultAllowanceResult"
)]
pub async fn check_vault_allowance(
    rpc_url: &str,
    vault: &SgVault,
) -> Result<VaultAllowanceResult, SubgraphError> {
    let (deposit_args, transaction_args) =
        get_deposit_and_transaction_args(rpc_url, &vault, U256::ZERO)?;

    Ok(VaultAllowanceResult(
        deposit_args
            .read_allowance(Address::from_str(&vault.owner.0)?, transaction_args.clone())
            .await?,
    ))
}

pub fn validate_amount(amount: &str) -> Result<U256, SubgraphError> {
    let amount = U256::from_str(&amount)?;
    if amount == U256::ZERO {
        return Err(SubgraphError::InvalidAmount);
    }
    Ok(amount)
}

pub fn validate_io_index(
    order: &SgOrder,
    is_input: bool,
    index: u8,
) -> Result<usize, SubgraphError> {
    let index = index as usize;
    if is_input {
        if order.inputs.len() <= index {
            return Err(SubgraphError::InvalidInputIndex);
        }
    } else {
        if order.outputs.len() <= index {
            return Err(SubgraphError::InvalidOutputIndex);
        }
    }
    Ok(index)
}

pub fn get_deposit_and_transaction_args(
    rpc_url: &str,
    vault: &SgVault,
    amount: U256,
) -> Result<(DepositArgs, TransactionArgs), SubgraphError> {
    let deposit_args = DepositArgs {
        token: Address::from_str(&vault.token.address.0)?,
        vault_id: U256::from_str(&vault.vault_id.0)?,
        amount,
    };
    let transaction_args = TransactionArgs {
        orderbook_address: Address::from_str(&vault.orderbook.id.0)?,
        rpc_url: rpc_url.to_string(),
        ..Default::default()
    };
    Ok((deposit_args, transaction_args))
}
