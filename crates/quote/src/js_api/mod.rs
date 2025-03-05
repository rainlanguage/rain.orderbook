use crate::{error::Error, BatchQuoteSpec, QuoteSpec};
use crate::{get_order_quotes, BatchQuoteTarget, QuoteTarget};
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::wasm_traits::TryIntoU256;
use rain_orderbook_subgraph_client::{types::common::SgOrder, utils::make_order_id};
use std::str::FromStr;
use wasm_bindgen_utils::prelude::*;

/// Get subgraph represented "order_id" of a QuoteTarget
#[wasm_bindgen(js_name = "getId")]
pub fn get_id(orderbook: &str, order_hash: &str) -> String {
    let mut orderbook_error = "orderbook address, ".to_string();
    let mut order_hash_error = "order hash, ".to_string();
    let orderbook = Address::from_hex(orderbook)
        .inspect_err(|e| orderbook_error.push_str(&e.to_string()))
        .expect_throw(&orderbook_error);
    let order_hash = U256::from_str(order_hash)
        .inspect_err(|e| order_hash_error.push_str(&e.to_string()))
        .expect_throw(&order_hash_error);
    encode_prefixed(make_order_id(orderbook, order_hash))
}

/// Quotes the target on the given rpc url
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_bindgen(js_name = "doQuoteTargets")]
pub async fn do_quote_targets(
    quote_targets: &BatchQuoteTarget,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
    multicall_address: Option<String>,
) -> Result<JsValue, Error> {
    let mut multicall_address_error = "multicall address, ".to_string();
    let multicall_address = multicall_address.map(|v| {
        Address::from_hex(v)
            .inspect_err(|e| multicall_address_error.push_str(&e.to_string()))
            .expect_throw(&multicall_address_error)
    });
    let mut gas_error = "gas, ".to_string();
    let gas_value = gas.map(|v| {
        v.try_into_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    let quote_targets: Vec<QuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| QuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = BatchQuoteTarget(quote_targets);
    match batch_quote_target
        .do_quote(rpc_url, block_number, gas_value, multicall_address)
        .await
    {
        Err(e) => Err(e),
        Ok(v) => Ok(js_sys::Array::from_iter(
            v.into_iter()
                .map(|e| e.map_or_else(JsValue::from, JsValue::from)),
        )
        .into()),
    }
}

/// Given a subgraph url, will fetch the order details from the subgraph and
/// then quotes them using the given rpc url.
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_bindgen(js_name = "doQuoteSpecs")]
pub async fn do_quote_specs(
    quote_specs: &BatchQuoteSpec,
    subgraph_url: &str,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
    multicall_address: Option<String>,
) -> Result<JsValue, Error> {
    let mut multicall_address_error = "multicall address, ".to_string();
    let multicall_address = multicall_address.map(|v| {
        Address::from_hex(v)
            .inspect_err(|e| multicall_address_error.push_str(&e.to_string()))
            .expect_throw(&multicall_address_error)
    });
    let mut gas_error = "gas, ".to_string();
    let gas_value = gas.map(|v| {
        v.try_into_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);
    match batch_quote_spec
        .do_quote(
            subgraph_url,
            rpc_url,
            block_number,
            gas_value,
            multicall_address,
        )
        .await
    {
        Err(e) => Err(e),
        Ok(v) => Ok(js_sys::Array::from_iter(
            v.into_iter()
                .map(|e| e.map_or_else(JsValue::from, JsValue::from)),
        )
        .into()),
    }
}

/// Given a subgraph url, will fetch orders details and returns their
/// respective quote targets.
/// Resolves with array of QuoteTarget object or undefined if no result
/// found on subgraph for a specific spec
#[wasm_bindgen(js_name = "getQuoteTargetFromSubgraph")]
pub async fn get_batch_quote_target_from_subgraph(
    quote_specs: &BatchQuoteSpec,
    subgraph_url: &str,
) -> Result<JsValue, Error> {
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);
    match batch_quote_spec
        .get_batch_quote_target_from_subgraph(subgraph_url)
        .await
    {
        Err(e) => Err(e),
        Ok(v) => Ok(to_js_value(
            &v.into_iter()
                .map(|e| e.map(QuoteTarget::from))
                .collect::<Vec<_>>(),
        )?),
    }
}

/// Get the quote for an order
/// Resolves with a BatchOrderQuotesResponse object
#[wasm_bindgen(js_name = "getOrderQuote")]
pub async fn get_order_quote(
    order: Vec<SgOrder>,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
) -> Result<JsValue, Error> {
    let mut gas_error = "gas, ".to_string();
    let gas_value = gas.map(|v| {
        v.try_into_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    Ok(to_js_value(
        &get_order_quotes(order, block_number, rpc_url.to_string(), gas_value).await?,
    )?)
}
