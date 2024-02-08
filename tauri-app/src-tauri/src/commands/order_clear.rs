use crate::error::CommandResult;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::{flattened::{OrderClearFlattened, TryIntoFlattenedError}, order_clears_list},
    TryIntoCsv,
    PaginationArgs,
};
use std::path::PathBuf;
use std::fs;

#[tauri::command]
pub async fn order_clears_list(
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<Vec<order_clears_list::OrderClear>> {
    let order_clears = subgraph_args
        .to_subgraph_client()
        .await?
        .order_clears_list(pagination_args)
        .await?;
    Ok(order_clears)
}

#[tauri::command]
pub async fn order_clears_list_write_csv(
    path: PathBuf,
    subgraph_args: SubgraphArgs,
    pagination_args: PaginationArgs,
) -> CommandResult<()> {
    let order_clears = order_clears_list(subgraph_args, pagination_args).await?;
    let order_clears_flattened: Vec<OrderClearFlattened> = order_clears
            .into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<OrderClearFlattened>, TryIntoFlattenedError>>()?;
    let csv_text = order_clears_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}