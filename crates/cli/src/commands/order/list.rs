use crate::{execute::Execute, subgraph::CliSubgraphCommandArgs};
use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::subgraph::SubgraphArgs;
use rain_orderbook_subgraph_queries::types::orders::Order;

use tracing::debug;
#[derive(Args, Clone)]
pub struct CliOrderListArgs {}

pub type List = CliSubgraphCommandArgs<CliOrderListArgs>;

impl Execute for List {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let orders = subgraph_args.to_subgraph_client().await?.orders().await?;
        debug!("{:#?}", orders);

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
            "Added At (UTC)",
            "Active",
            "Owner",
            "Input Tokens",
            "Output Tokens",
        ]);

    for order in orders.iter() {
        let timestamp_i64 = order.timestamp.0.parse::<i64>()?;
        let timestamp_naive = NaiveDateTime::from_timestamp_opt(timestamp_i64, 0)
            .ok_or(anyhow!("Failed to parse timestamp into NaiveDateTime"))?;
        let timestamp_utc = Utc.from_utc_datetime(&timestamp_naive);

        table.add_row(vec![
            order.id.inner().into(),
            format!("{}", timestamp_utc.format("%Y-%m-%d %H:%M:%S")),
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
