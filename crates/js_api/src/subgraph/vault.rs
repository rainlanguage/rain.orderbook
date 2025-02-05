use super::SubgraphError;
use alloy::primitives::{Address, Bytes, U256};
use cynic::Id;
use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_common::deposit::DepositArgs;
use rain_orderbook_common::transaction::TransactionArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;
use rain_orderbook_subgraph_client::types::common::{Order, Vault, VaultsListFilterArgs};
use rain_orderbook_subgraph_client::{
    MultiOrderbookSubgraphClient, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError, PaginationArgs,
};
use reqwest::Url;
use std::str::FromStr;

/// Fetch all vaults from multiple subgraphs
/// Returns a list of VaultWithSubgraphName structs
#[wasm_bindgen(js_name = "getVaults")]
pub async fn get_vaults(
    subgraphs: Vec<MultiSubgraphArgs>,
    filter_args: VaultsListFilterArgs,
    pagination_args: PaginationArgs,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = MultiOrderbookSubgraphClient::new(subgraphs);
    let vaults = client.vaults_list(filter_args, pagination_args).await?;
    Ok(to_value(&vaults)?)
}

/// Fetch a single vault
/// Returns the Vault struct
#[wasm_bindgen(js_name = "getVault")]
pub async fn get_vault(url: &str, id: &str) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let vault = client.vault_detail(Id::new(id)).await?;
    Ok(to_value(&vault)?)
}

/// Fetch balance changes for a vault
/// Returns a list of VaultBalanceChangeUnwrapped structs
#[wasm_bindgen(js_name = "getVaultBalanceChanges")]
pub async fn get_vault_balance_changes(
    url: &str,
    id: &str,
    pagination_args: PaginationArgs,
) -> Result<JsValue, OrderbookSubgraphClientError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let changes = client
        .vault_balance_changes_list(Id::new(id), pagination_args)
        .await?;
    Ok(to_value(&changes)?)
}

/// Get deposit calldata for a vault
/// Returns a string of the calldata
#[wasm_bindgen(js_name = "getVaultDepositCalldata")]
pub async fn get_vault_deposit_calldata(
    vault: &Vault,
    deposit_amount: &str,
) -> Result<JsValue, SubgraphError> {
    let deposit_amount = validate_amount(deposit_amount)?;

    let deposit_args = DepositArgs {
        token: Address::from_str(&vault.token.address.0)?,
        vault_id: U256::from_str(&vault.vault_id.0)?,
        amount: deposit_amount,
    };

    Ok(to_value(&Bytes::copy_from_slice(
        &deposit_args.get_deposit_calldata().await?,
    ))?)
}

/// Get withdraw calldata for a vault
#[wasm_bindgen(js_name = "getVaultWithdrawCalldata")]
pub async fn get_vault_withdraw_calldata(
    vault: &Vault,
    withdraw_amount: &str,
) -> Result<JsValue, SubgraphError> {
    let withdraw_amount = validate_amount(withdraw_amount)?;

    Ok(to_value(&Bytes::copy_from_slice(
        &WithdrawArgs {
            token: Address::from_str(&vault.token.address.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            target_amount: withdraw_amount,
        }
        .get_withdraw_calldata()
        .await?,
    ))?)
}

#[wasm_bindgen(js_name = "getVaultApprovalCalldata")]
pub async fn get_vault_approval_calldata(
    rpc_url: &str,
    vault: &Vault,
    deposit_amount: &str,
) -> Result<JsValue, SubgraphError> {
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

    Ok(to_value(&Bytes::copy_from_slice(
        &deposit_args.get_approve_calldata(transaction_args).await?,
    ))?)
}

#[wasm_bindgen(js_name = "checkVaultAllowance")]
pub async fn check_vault_allowance(rpc_url: &str, vault: &Vault) -> Result<JsValue, SubgraphError> {
    let (deposit_args, transaction_args) =
        get_deposit_and_transaction_args(rpc_url, &vault, U256::ZERO)?;

    Ok(to_value(
        &deposit_args
            .read_allowance(Address::from_str(&vault.owner.0)?, transaction_args.clone())
            .await?,
    )?)
}

pub fn validate_amount(amount: &str) -> Result<U256, SubgraphError> {
    let amount = U256::from_str(&amount)?;
    if amount == U256::ZERO {
        return Err(SubgraphError::InvalidAmount);
    }
    Ok(amount)
}

pub fn validate_io_index(order: &Order, is_input: bool, index: u8) -> Result<usize, SubgraphError> {
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
    vault: &Vault,
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
