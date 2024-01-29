use crate::transaction_status::{TransactionStatusNoticeRwLock};
use rain_orderbook_common::{
    subgraph::SubgraphArgs,
    remove_order::RemoveOrderArgs,
    transaction::TransactionArgs,
};
use rain_orderbook_subgraph_queries::types::{
    order::Order as OrderDetail,
    orders::Order as OrdersListItem,
};
use tauri::AppHandle;
use crate::toast::toast_error;

#[tauri::command]
pub async fn orders_list(subgraph_args: SubgraphArgs) -> Result<Vec<OrdersListItem>, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .orders()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn order_detail(id: String, subgraph_args: SubgraphArgs) -> Result<OrderDetail, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .order(id.into())
        .await
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn order_remove(
    app_handle: AppHandle,
    id: String,
    transaction_args: TransactionArgs,
    subgraph_args: SubgraphArgs
) -> Result<(), String> {
    println!("order id is {:?}", id);
    let order = subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| {
            let text = String::from("Subgraph URL is invalid");
            toast_error(app_handle.clone(), text.clone());
            text
        })?
        .order(id.into())
        .await
        .map_err(|e| {
            let text = e.to_string();
            toast_error(app_handle.clone(), text.clone());
            text
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
