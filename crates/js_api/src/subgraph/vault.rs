use super::order::get_sg_order;
use super::SubgraphError;
use alloy::primitives::{Address, Bytes, U256};
use cynic::Id;
use rain_orderbook_bindings::wasm_traits::prelude::*;
use rain_orderbook_common::deposit::DepositArgs;
use rain_orderbook_common::withdraw::WithdrawArgs;
use rain_orderbook_subgraph_client::types::common::VaultsListFilterArgs;
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
    url: &str,
    order_id: &str,
    output_index: u8,
    amount: &str,
) -> Result<JsValue, SubgraphError> {
    let amount = U256::from_str(&amount)?;
    if amount == U256::ZERO {
        return Err(SubgraphError::InvalidAmount);
    }

    let order = get_sg_order(url, order_id).await?;

    let index = output_index as usize;
    if order.outputs.len() <= index {
        return Err(SubgraphError::InvalidOutputIndex);
    }

    let calldata = DepositArgs {
        token: Address::from_str(&order.outputs[index].token.address.0)?,
        vault_id: U256::from_str(&order.outputs[index].vault_id.0)?,
        amount,
    }
    .get_deposit_calldata()
    .await?;

    Ok(to_value(&Bytes::copy_from_slice(&calldata))?)
}

/// Get withdraw calldata for a vault
/// Returns a string of the calldata
#[wasm_bindgen(js_name = "getVaultWithdrawCalldata")]
pub async fn get_vault_withdraw_calldata(
    url: &str,
    order_id: &str,
    input_index: u8,
    amount: &str,
) -> Result<JsValue, SubgraphError> {
    let amount = U256::from_str(&amount)?;
    if amount == U256::ZERO {
        return Err(SubgraphError::InvalidAmount);
    }

    let order = get_sg_order(url, order_id).await?;

    let index = input_index as usize;
    if order.inputs.len() <= index {
        return Err(SubgraphError::InvalidInputIndex);
    }

    let calldata = WithdrawArgs {
        token: Address::from_str(&order.inputs[index].token.address.0)?,
        vault_id: U256::from_str(&order.inputs[index].vault_id.0)?,
        target_amount: amount,
    }
    .get_withdraw_calldata()
    .await?;

    Ok(to_value(&Bytes::copy_from_slice(&calldata))?)
}
