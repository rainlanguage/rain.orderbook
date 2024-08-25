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
            let csv_text = subgraph_args
                .to_subgraph_client()
                .await?
                .orders_list_all()
                .await?
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?
                .try_into_csv()?;

            println!("{}", csv_text);
        } else {
            let table = build_table(
                subgraph_args
                    .to_subgraph_client()
                    .await?
                    .orders_list(self.pagination_args.clone().into())
                    .await?
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?,
            )?;
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
    use alloy::{hex::encode_prefixed, primitives::B256, sol_types::SolValue};
    use httpmock::MockServer;
    use rain_orderbook_bindings::IOrderBookV4::{OrderV3, IO};
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_csv_execute_happy() {
        // mock subgraph with pagination
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.body_contains("\"skip\":0");
            then.json_body_obj(&get_sg_response(false));
        });
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"orders": []}}));
        });

        let cli_order_list_args = CliOrderListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: true,
                page_size: 25,
                page: 1,
            },
        };

        // should succeed
        assert!(cli_order_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_csv_execute_happy() {
        // mock subgraph
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response(false));
        });

        let cli_order_list_args = CliOrderListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
        };

        // should succeed
        assert!(cli_order_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        // mock sg with corrupt response
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response(true));
        });

        let cli_order_list_args = CliOrderListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
        };

        // should error
        assert!(cli_order_list_args.execute().await.is_err());
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response(corrupt: bool) -> Value {
        let order = OrderV3 {
            validInputs: vec![IO::default()],
            validOutputs: vec![IO::default()],
            ..Default::default()
        };
        json!({
            "data": {
                "orders": [{
                    "id": encode_prefixed(B256::random()),
                    "orderBytes": if corrupt {
                        encode_prefixed(vec![])
                    } else {
                        encode_prefixed(order.abi_encode())
                    },
                    "orderHash": encode_prefixed(B256::random()),
                    "owner": encode_prefixed(order.owner),
                    "outputs": [{
                        "token": {
                            "id": encode_prefixed(order.validOutputs[0].token.0.0),
                            "address": encode_prefixed(order.validOutputs[0].token.0.0),
                            "name": "T1",
                            "symbol": "T1",
                            "decimals": order.validOutputs[0].decimals.to_string()
                        },
                        "balance": "0",
                        "vaultId": order.validOutputs[0].vaultId.to_string(),
                    }],
                    "inputs": [{
                        "token": {
                            "id": encode_prefixed(order.validInputs[0].token.0.0),
                            "address": encode_prefixed(order.validInputs[0].token.0.0),
                            "name": "T2",
                            "symbol": "T2",
                            "decimals": order.validInputs[0].decimals.to_string()
                        },
                        "balance": "0",
                        "vaultId": order.validInputs[0].vaultId.to_string(),
                    }],
                    "active": true,
                    "addEvents": [{
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                        }
                    }],
                    "timestampAdded": "0",
                }]
            }
        })
    }
}
