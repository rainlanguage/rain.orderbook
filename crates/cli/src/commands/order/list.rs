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
    types::{FlattenError, OrderFlattened},
};
use rain_orderbook_subgraph_client::PaginationArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderListArgs {
    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let orders = subgraph_args
                .to_subgraph_client()
                .await?
                .orders_list_all()
                .await?;
            let orders_flattened: Vec<OrderFlattened> = orders
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderFlattened>, FlattenError>>(
            )?;

            let csv_text = orders_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let pagination_args: PaginationArgs = self.pagination_args.clone().into();
            let orders = subgraph_args
                .to_subgraph_client()
                .await?
                .orders_list(pagination_args)
                .await?;
            let orders_flattened: Vec<OrderFlattened> = orders
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderFlattened>, FlattenError>>(
            )?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::str::FromStr;


    #[tokio::test]
    async fn test_execute() {
        let pagination_args = CliPaginationArgs {
            csv: false,
            page_size: 25,
            page: 1,
        };
        let subgraph_args = CliSubgraphArgs {
            subgraph_url: String::from("https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-sepolia/1.1/gn"),
        };
        let pagination_args: PaginationArgs = pagination_args.clone().into();
        let subgraph_args: SubgraphArgs = subgraph_args.clone().into();
        let orders = subgraph_args
            .to_subgraph_client()
            .await.unwrap()
            .orders_list(pagination_args)
            .await.unwrap();

        let orders_flattened: Vec<OrderFlattened> = orders
            .into_iter()
            .map(|o| o.try_into())
            .collect::<Result<Vec<OrderFlattened>, FlattenError>>(
        ).unwrap();
        
        // Test that the second order throws a FlattenError
        let result: std::result::Result<OrderFlattened, _> = orders_flattened[1].clone().try_into(); 
        
    }


}