use crate::error::CommandResult;
use rain_orderbook_common::{
    csv::TryIntoCsv, subgraph::SubgraphArgs, types::OrderTakeFlattened, types::FlattenError
};
use rain_orderbook_subgraph_client::{types::order_takes_list, PaginationArgs};
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn order_takes_list(
    order_id: String,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<order_takes_list::Trade>> {
    let order_takes = subgraph_args
        .to_subgraph_client()
        .await?
        .order_takes_list(order_id.clone().into(), pagination_args)
        .await?;
    Ok(order_takes)
}

#[tauri::command]
pub async fn order_takes_list_write_csv(
    path: PathBuf,
    order_id: String,
    subgraph_args: SubgraphArgs,
) -> CommandResult<()> {
    let order_takes = subgraph_args
        .to_subgraph_client()
        .await?
        .order_takes_list_all(order_id.clone().into())
        .await?;
    let order_takes_flattened: Vec<OrderTakeFlattened> = order_takes
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?;
    let csv_text = order_takes_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}
