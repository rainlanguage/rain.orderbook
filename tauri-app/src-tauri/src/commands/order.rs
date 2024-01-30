use crate::{
    transaction_status::{TransactionStatusNoticeRwLock},
    toast::toast_error
};
use rain_orderbook_common::{
    subgraph::SubgraphArgs,
    remove_order::RemoveOrderArgs,
    transaction::TransactionArgs,
};
use rain_orderbook_subgraph_client::types::{
    order_detail,
    orders_list
};
use tauri::AppHandle;
use crate::error::CommandResult;

#[tauri::command]
pub async fn orders_list(subgraph_args: SubgraphArgs) -> CommandResult<Vec<orders_list::Order>> {
    let orders = subgraph_args
        .to_subgraph_client()
        .await?
        .orders_list()
        .await?;
    
    Ok(orders)
}

#[tauri::command]
pub async fn order_detail(id: String, subgraph_args: SubgraphArgs) -> CommandResult<order_detail::Order> {
    let order = subgraph_args
        .to_subgraph_client()
        .await?
        .order_detail(id.into())
        .await?;

    Ok(order)
}


#[tauri::command]
pub async fn order_remove(
    app_handle: AppHandle,
    id: String,
    transaction_args: TransactionArgs,
    subgraph_args: SubgraphArgs
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

    let tx_status_notice =
        TransactionStatusNoticeRwLock::new("Remove order".into(), None);
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
