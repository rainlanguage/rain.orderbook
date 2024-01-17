use crate::execute::Execute;
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_subgraph_queries::{
    types::orders::{Order, OrdersQuery},
    OrderbookSubgraphClient,
};
use std::str::from_utf8;
use tracing::{debug, info};

#[derive(Args, Clone)]
pub struct CliOrderListArgs {
    #[arg(short, long, help = "Filter by `active`")]
    active: Option<bool>,
}

pub type List = CliOrderListArgs;

impl Execute for List {
    async fn execute(&self) -> Result<()> {
        let orders = OrderbookSubgraphClient::orders(self.active).await?;
        info!("{:#?}", orders);
        let table = build_orders_table(orders)?;
        println!("{}", table);

        Ok(())
    }
}

fn build_orders_table(orders: Vec<Order>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Order ID",
            "Active",
            "Owner",
            "Input Tokens",
            "Output Tokens",
        ]);

    for order in orders.iter() {
        table.add_row(vec![
            order.id.inner().into(),
            format!("{}", order.order_active),
            format!("{}", order.owner.id.0),
            order
                .valid_inputs
                .clone()
                .map_or("".into(), |valid_inputs| {
                    valid_inputs
                        .into_iter()
                        .map(|v| v.token.symbol)
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
            order
                .valid_outputs
                .clone()
                .map_or("".into(), |valid_outputs| {
                    valid_outputs
                        .into_iter()
                        .map(|v| v.token.symbol)
                        .collect::<Vec<String>>()
                        .join(", ")
                }),
        ]);
    }

    Ok(table)
}
