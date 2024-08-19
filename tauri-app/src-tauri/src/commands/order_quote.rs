use crate::error::{CommandError, CommandResult};
use alloy::primitives::{Address, U256};
use rain_orderbook_quote::{BatchQuoteSpec, OrderQuoteValue, QuoteSpec};
use rain_orderbook_subgraph_client::types::orders_list;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct BatchOrderQuotesResponse {
    pub pair_name: String,
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn batch_order_quotes(
    orders: Vec<orders_list::Order>,
    orderbook: Address,
    subgraph_url: String,
    rpc_url: String,
) -> CommandResult<Vec<BatchOrderQuotesResponse>> {
    let mut results: Vec<BatchOrderQuotesResponse> = Vec::new();

    for order in &orders {
        let pairs: Vec<(String, usize, usize)> = order
            .inputs
            .iter()
            .enumerate()
            .flat_map(|(input_index, input)| {
                order
                    .outputs
                    .iter()
                    .enumerate()
                    .map(move |(output_index, output)| {
                        let pair_name = format!(
                            "{}/{}",
                            input.token.symbol.as_deref().unwrap_or("UNKNOWN"),
                            output.token.symbol.as_deref().unwrap_or("UNKNOWN")
                        );
                        (pair_name, input_index, output_index)
                    })
            })
            .collect();

        let order_hash =
            U256::from_str_radix(&order.order_hash.0[2..], 16).map_err(CommandError::from)?;

        let mut quote_specs = Vec::new();

        for (pair_name, input_index, output_index) in pairs {
            let quote_spec = QuoteSpec {
                order_hash,
                input_io_index: input_index as u8,
                output_io_index: output_index as u8,
                signed_context: vec![],
                orderbook,
            };

            quote_specs.push((quote_spec, pair_name));
        }

        let quote_values =
            BatchQuoteSpec(quote_specs.iter().map(|(spec, _)| spec.clone()).collect())
                .do_quote(&subgraph_url, &rpc_url, None, None)
                .await;

        if let Ok(quote_values) = quote_values {
            for (quote_value_result, (_, pair_name)) in quote_values.into_iter().zip(quote_specs) {
                match quote_value_result {
                    Ok(quote_value) => {
                        results.push(BatchOrderQuotesResponse {
                            pair_name,
                            success: true,
                            data: Some(quote_value),
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(BatchOrderQuotesResponse {
                            pair_name,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        } else if let Err(e) = quote_values {
            for (_, pair_name) in quote_specs {
                results.push(BatchOrderQuotesResponse {
                    pair_name,
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}
