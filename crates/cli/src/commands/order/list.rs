use std::{fs::canonicalize, path::PathBuf};

use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::flattened::{OrderFlattened, TryIntoFlattenedError},
    PaginationArgs, WriteCsv,
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderListArgs {
    #[arg(long, help = "Write results to a CSV file at the path provided")]
    pub csv_file: Option<PathBuf>,

    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let pagination_args: PaginationArgs = self.pagination_args.clone().into();
        let orders = subgraph_args
            .to_subgraph_client()
            .await?
            .orders_list(pagination_args)
            .await?;
        let orders_flattened: Vec<OrderFlattened> =
            orders
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderFlattened>, TryIntoFlattenedError>>()?;

        if let Some(csv_file) = self.csv_file.clone() {
            orders_flattened.write_csv(csv_file.clone())?;
            info!("Saved to CSV at {:?}", canonicalize(csv_file.as_path())?);
        } else {
            let table = build_table(orders_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(orders: Vec<OrderFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Order ID",
            "Added At",
            "Active",
            "Owner",
            "Input Tokens",
            "Output Tokens",
        ]);

    for order in orders.into_iter() {
        table.add_row(vec![
            order.id,
            order.timestamp_display,
            format!("{}", order.order_active),
            format!("{}", order.owner.0),
            order.valid_inputs_token_symbols_display,
            order.valid_outputs_token_symbols_display,
        ]);
    }

    Ok(table)
}
