use rain_orderbook_common::{
    subgraph::SubgraphArgs
};
use rain_orderbook_subgraph_queries::types::{
    orders::Order as OrdersListItem,
};

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