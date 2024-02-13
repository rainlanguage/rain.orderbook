use crate::error::CommandResult;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::{flattened::{OrderTakeFlattened, TryIntoFlattenedError}, order_takes_list},
    TryIntoCsv,
    PaginationArgs,
};
use std::path::PathBuf;
use std::fs;

#[tauri::command]
pub async fn order_takes_list(
    order_id: String,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<order_takes_list::TakeOrderEntity>> {
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
    pagination_args: PaginationArgs,
) -> CommandResult<()> {
    let order_takes = order_takes_list(order_id.clone().into(), subgraph_args, pagination_args).await?;
    let order_takes_flattened: Vec<OrderTakeFlattened> = order_takes
            .into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<OrderTakeFlattened>, TryIntoFlattenedError>>()?;
    let csv_text = order_takes_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}