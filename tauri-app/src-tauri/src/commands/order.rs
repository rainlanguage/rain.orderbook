use crate::error::CommandResult;
use crate::{toast::toast_error, transaction_status::TransactionStatusNoticeRwLock};
use alloy::primitives::Bytes;
use rain_orderbook_app_settings::{deployment::DeploymentCfg, scenario::ScenarioCfg};
use rain_orderbook_common::{
    add_order::AddOrderArgs, csv::TryIntoCsv, dotrain_order::DotrainOrder,
    remove_order::RemoveOrderArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
    types::FlattenError, types::OrderFlattened,
};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn orders_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let orders = subgraph_args
        .to_subgraph_client()?
        .orders_list_all()
        .await?;
    let orders_flattened: Vec<OrderFlattened> = orders
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?;
    let csv_text = orders_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}

#[tauri::command]
pub async fn order_add(
    app_handle: AppHandle,
    dotrain: String,
    deployment: DeploymentCfg,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice = TransactionStatusNoticeRwLock::new("Add order".into());
    let add_order_args = AddOrderArgs::new_from_deployment(dotrain, deployment).await?;
    add_order_args
        .execute(transaction_args, |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;

    Ok(())
}

#[tauri::command]
pub async fn order_remove(
    app_handle: AppHandle,
    id: String,
    transaction_args: TransactionArgs,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let order = subgraph_args
        .to_subgraph_client()
        .map_err(|e| {
            toast_error(app_handle.clone(), String::from("Subgraph URL is invalid"));
            e
        })?
        .order_detail(id.into())
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;
    let remove_order_args: RemoveOrderArgs = order.into();

    let tx_status_notice = TransactionStatusNoticeRwLock::new("Remove order".into());
    let _ = remove_order_args
        .execute(transaction_args.clone(), |status| {
            tx_status_notice.update_status_and_emit(app_handle.clone(), status);
        })
        .await
        .map_err(|e| {
            tx_status_notice.set_failed_status_and_emit(app_handle.clone(), e.to_string());
        });

    Ok(())
}

#[tauri::command]
pub async fn order_add_calldata(
    app_handle: AppHandle,
    dotrain: String,
    deployment: DeploymentCfg,
    transaction_args: TransactionArgs,
) -> CommandResult<Bytes> {
    let add_order_args = AddOrderArgs::new_from_deployment(dotrain, deployment).await?;
    let calldata = add_order_args
        .get_add_order_calldata(transaction_args)
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn order_remove_calldata(
    app_handle: AppHandle,
    id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<Bytes> {
    let order = subgraph_args
        .to_subgraph_client()
        .map_err(|e| {
            toast_error(app_handle.clone(), String::from("Subgraph URL is invalid"));
            e
        })?
        .order_detail(id.into())
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;
    let remove_order_args: RemoveOrderArgs = order.into();
    let calldata = remove_order_args
        .get_rm_order_calldata()
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;

    Ok(Bytes::from(calldata))
}

#[tauri::command]
pub async fn compose_from_scenario(
    dotrain: String,
    settings: Option<Vec<String>>,
    scenario: ScenarioCfg,
) -> CommandResult<String> {
    let mut dotrain_order = DotrainOrder::new();
    dotrain_order.initialize(dotrain, settings).await?;
    Ok(dotrain_order
        .compose_scenario_to_rainlang(scenario.key)
        .await?)
}

#[tauri::command]
pub async fn validate_raindex_version(dotrain: String, settings: Vec<String>) -> CommandResult<()> {
    let mut dotrain_order = DotrainOrder::new();
    dotrain_order.initialize(dotrain, Some(settings)).await?;
    Ok(dotrain_order.validate_raindex_version().await?)
}
