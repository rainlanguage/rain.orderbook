use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_client::{
    types::flattened::{OrderClearFlattened, TryIntoFlattenedError},
    PaginationArgs, TryIntoCsv,
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderClearListArgs {
    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderClearListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let pagination_args: PaginationArgs = self.pagination_args.clone().into();
        let order_clears = subgraph_args
            .to_subgraph_client()
            .await?
            .order_clears_list(pagination_args)
            .await?;
        let order_clears_flattened: Vec<OrderClearFlattened> = order_clears
            .into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<OrderClearFlattened>, TryIntoFlattenedError>>()?;

        if self.pagination_args.csv {
            let csv_text = order_clears_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let table = build_table(order_clears_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(order_clears: Vec<OrderClearFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Order Clear ID",
            "Cleared At",
            "Sender",
            "Clearer",
            "Order A ID",
            "Bounty A",
            "Order B ID",
            "Bounty B",
        ]);

    for order_clear in order_clears.into_iter() {
        table.add_row(vec![
            order_clear.id,
            order_clear.timestamp_display,
            format!("{:?}", order_clear.sender),
            format!("{:?}", order_clear.clearer),
            order_clear.order_a_id,
            format!(
                "{:?} {}",
                order_clear.bounty_amount_a, order_clear.bounty_token_a
            ),
            order_clear.order_b_id,
            format!(
                "{:?} {}",
                order_clear.bounty_amount_b, order_clear.bounty_token_b
            ),
        ]);
    }

    Ok(table)
}
