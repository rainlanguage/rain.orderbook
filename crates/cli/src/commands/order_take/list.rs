use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::{
    csv::TryIntoCsv,
    subgraph::SubgraphArgs,
    types::{FlattenError, OrderTakeFlattened, NO_SYMBOL},
};
use rain_orderbook_subgraph_client::PaginationArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderTakesListArgs {
    #[arg(short = 'i', long, help = "ID of the Order")]
    order_id: String,

    #[clap(flatten)]
    pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderTakesListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let order_takes = subgraph_args
                .to_subgraph_client()
                .await?
                .order_takes_list_all(self.order_id.clone().into())
                .await?;
            let order_takes_flattened: Vec<OrderTakeFlattened> = order_takes
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?;

            let csv_text = order_takes_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let pagination_args: PaginationArgs = self.pagination_args.clone().into();
            let order_takes = subgraph_args
                .to_subgraph_client()
                .await?
                .order_takes_list(self.order_id.clone().into(), pagination_args)
                .await?;
            let order_takes_flattened: Vec<OrderTakeFlattened> = order_takes
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?;

            let table = build_table(order_takes_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(order_take: Vec<OrderTakeFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Taken At", "Sender", "Input", "Output"]);

    for order_take in order_take.into_iter() {
        table.add_row(vec![
            order_take.id,
            order_take.timestamp_display,
            order_take.sender.0,
            format!(
                "{} {}",
                order_take.input_display,
                order_take.input_token_symbol.unwrap_or(NO_SYMBOL.into())
            ),
            format!(
                "{} {}",
                order_take.output_display,
                order_take.output_token_symbol.unwrap_or(NO_SYMBOL.into())
            ),
        ]);
    }

    Ok(table)
}
