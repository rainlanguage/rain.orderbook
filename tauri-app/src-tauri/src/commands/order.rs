use crate::error::CommandResult;
use crate::{toast::toast_error, transaction_status::TransactionStatusNoticeRwLock};
use rain_orderbook_common::{
    remove_order::RemoveOrderArgs,
    subgraph::SubgraphArgs,
    transaction::TransactionArgs,
    add_order::AddOrderArgs,
};
use rain_orderbook_subgraph_client::{
    types::{flattened::{OrderFlattened, TryIntoFlattenedError},  order_detail, orders_list},
    TryIntoCsv,
    PaginationArgs
};
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn orders_list(
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<orders_list::Order>> {
    let orders = subgraph_args
        .to_subgraph_client()
        .await?
        .orders_list(pagination_args)
        .await?;
    Ok(orders)
}

#[tauri::command]
pub async fn orders_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<()> {
    let orders = orders_list(subgraph_args, pagination_args).await?;
    let orders_flattened: Vec<OrderFlattened> = orders.into_iter().map(|o| o.try_into()).collect::<Result<Vec<OrderFlattened>, TryIntoFlattenedError>>()?;
    orders_flattened.write_csv(path)?;

    Ok(())
}

#[tauri::command]
pub async fn order_detail(
    id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<order_detail::Order> {
    let order = subgraph_args
        .to_subgraph_client()
        .await?
        .order_detail(id.into())
        .await?;

    Ok(order)
}

#[tauri::command]
pub async fn order_add(
    app_handle: AppHandle,
    add_order_args: AddOrderArgs,
    transaction_args: TransactionArgs,
) -> CommandResult<()> {
    let tx_status_notice = TransactionStatusNoticeRwLock::new("Add order".into(), None);
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

    let tx_status_notice = TransactionStatusNoticeRwLock::new("Remove order".into(), None);
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
