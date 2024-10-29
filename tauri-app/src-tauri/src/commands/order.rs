use crate::error::CommandResult;
use crate::{toast::toast_error, transaction_status::TransactionStatusNoticeRwLock};
use alloy::primitives::Bytes;
use futures::future::join_all;
use rain_orderbook_app_settings::{deployment::Deployment, scenario::Scenario};
use rain_orderbook_common::{
    add_order::AddOrderArgs, csv::TryIntoCsv, dotrain_order::DotrainOrder,
    remove_order::RemoveOrderArgs, subgraph::SubgraphArgs, transaction::TransactionArgs,
    types::FlattenError, types::OrderDetailExtended, types::OrderFlattened,
};
use rain_orderbook_subgraph_client::{types::common::*, PaginationArgs};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn orders_list(
    subgraph_args_list: Vec<SubgraphArgs>,
    filter_args: OrdersListFilterArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<Order>> {
    let clients_futures = subgraph_args_list
        .into_iter()
        .map(|args| async move { args.to_subgraph_client().await });
    let clients = join_all(clients_futures).await;
    let valid_clients: Vec<_> = clients.into_iter().filter_map(Result::ok).collect();

    let futures = valid_clients.into_iter().map(|client| {
        let filter_args = filter_args.clone();
        let pagination_args = pagination_args.clone();
        async move { client.orders_list(filter_args, pagination_args).await }
    });
    let results = join_all(futures).await;

    let mut all_orders: Vec<Order> = results
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect();

    all_orders.sort_by(|a, b| {
        let a_timestamp = a.timestamp_added.0.parse::<i64>().unwrap_or(0);
        let b_timestamp = b.timestamp_added.0.parse::<i64>().unwrap_or(0);
        b_timestamp.cmp(&a_timestamp)
    });

    Ok(all_orders)
}

#[tauri::command]
pub async fn orders_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let orders = subgraph_args
        .to_subgraph_client()
        .await?
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
pub async fn order_detail(
    id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<OrderDetailExtended> {
    let order = subgraph_args
        .to_subgraph_client()
        .await?
        .order_detail(id.into())
        .await?;
    let order_extended: OrderDetailExtended = order.try_into()?;

    Ok(order_extended)
}

#[tauri::command]
pub async fn order_add(
    app_handle: AppHandle,
    dotrain: String,
    deployment: Deployment,
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
        .await
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
    deployment: Deployment,
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
        .await
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
    settings: Option<String>,
    scenario: Scenario,
) -> CommandResult<String> {
    let order = DotrainOrder::new(dotrain.clone(), settings).await?;
    Ok(order.compose_scenario_to_rainlang(scenario.name).await?)
}

#[tauri::command]
pub async fn validate_raindex_version(dotrain: String, settings: String) -> CommandResult<()> {
    let order = DotrainOrder::new(dotrain.clone(), Some(settings)).await?;
    Ok(order.validate_raindex_version().await?)
}
