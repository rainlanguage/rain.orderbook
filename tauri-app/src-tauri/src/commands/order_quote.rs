use crate::error::{CommandError, CommandResult};
use crate::toast::toast_error;
use alloy::primitives::{Address, U256};
use rain_orderbook_quote::{BatchQuoteSpec, OrderQuoteValue, QuoteSpec};
use tauri::AppHandle;

#[tauri::command]
pub async fn batch_order_quotes(
    app_handle: AppHandle,
    order_hashes: Vec<String>,
    orderbook: Address,
    subgraph_url: String,
    rpc_url: String,
) -> CommandResult<Vec<OrderQuoteValue>> {
    let quote_specs = order_hashes
        .into_iter()
        .map(|hash| {
            let order_hash = U256::from_str_radix(&hash[2..], 16).map_err(CommandError::from)?;
            Ok(QuoteSpec {
                order_hash,
                input_io_index: 0,
                output_io_index: 0,
                signed_context: vec![],
                orderbook,
            })
        })
        .collect::<Result<Vec<QuoteSpec>, CommandError>>()
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;
    let quote_values = BatchQuoteSpec(quote_specs)
        .do_quote(&subgraph_url, &rpc_url, None, None)
        .await
        .map_err(|e| {
            toast_error(app_handle.clone(), e.to_string());
            e
        })?;
    let quote_results: Vec<OrderQuoteValue> = quote_values
        .into_iter()
        .map(|quote_value| quote_value.map_err(|e| CommandError::from(e)))
        .collect::<Result<Vec<OrderQuoteValue>, CommandError>>()?;

    Ok(quote_results)
}
