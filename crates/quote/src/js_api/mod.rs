use crate::{error::Error, BatchQuoteSpec as MainBatchQuoteSpec, QuoteSpec as MainQuoteSpec};
use crate::{
    get_order_quotes, BatchQuoteTarget as MainBatchQuoteTarget, QuoteTarget as MainQuoteTarget,
};
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::js_api::{Quote, SignedContextV1};
use rain_orderbook_bindings::{
    impl_all_wasm_traits,
    wasm_traits::{prelude::*, ToU256},
};
use rain_orderbook_subgraph_client::{types::common::Order, utils::make_order_id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod impls;

/// Holds quoted order max output and ratio
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteValue {
    pub max_output: String,
    pub ratio: String,
}

/// A quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTarget {
    pub quote_config: Quote,
    pub orderbook: String,
}

/// Batch quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(transparent)]
pub struct BatchQuoteTarget(pub Vec<QuoteTarget>);

/// A quote target specifier, where the order details need to be fetched from a
/// source (such as subgraph) to build a [QuoteTarget] out of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSpec {
    pub order_hash: String,
    #[serde(rename = "inputIOIndex")]
    pub input_io_index: u8,
    #[serde(rename = "outputIOIndex")]
    pub output_io_index: u8,
    pub signed_context: Vec<SignedContextV1>,
    pub orderbook: String,
}

/// Batch quote spec
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(transparent)]
pub struct BatchQuoteSpec(pub Vec<QuoteSpec>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(untagged)]
pub enum QuoteResult {
    Ok(OrderQuoteValue),
    Err(String),
}

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
        v.to_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    let quote_targets: Vec<MainQuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| MainQuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = MainBatchQuoteTarget(quote_targets);
    match batch_quote_target
        .do_quote(rpc_url, block_number, gas_value, multicall_address)
        .await
    {
        Err(e) => Err(e),
        Ok(v) => Ok(to_value(
            &v.into_iter().map(QuoteResult::from).collect::<Vec<_>>(),
        )?),
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
        v.to_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    let quote_specs: Vec<MainQuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| MainQuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = MainBatchQuoteSpec(quote_specs);
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
        Ok(v) => Ok(to_value(
            &v.into_iter().map(QuoteResult::from).collect::<Vec<_>>(),
        )?),
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
    let quote_specs: Vec<MainQuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| MainQuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = MainBatchQuoteSpec(quote_specs);
    match batch_quote_spec
        .get_batch_quote_target_from_subgraph(subgraph_url)
        .await
    {
        Err(e) => Err(e),
        Ok(v) => Ok(to_value(
            &v.into_iter()
                .map(|e| e.map(QuoteTarget::from))
                .collect::<Vec<_>>(),
        )?),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub pair_name: String,
    pub input_index: u32,
    pub output_index: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderQuotesResponse {
    pub pair: Pair,
    pub block_number: u64,
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    pub error: Option<String>,
}

/// Get the quote for an order
/// Resolves with a BatchOrderQuotesResponse object
#[wasm_bindgen(js_name = "getOrderQuote")]
pub async fn get_order_quote(
    order: Vec<Order>,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
) -> Result<JsValue, Error> {
    let mut gas_error = "gas, ".to_string();
    let gas_value = gas.map(|v| {
        v.to_u256()
            .inspect_err(|e| gas_error.push_str(&e.to_string()))
            .expect_throw(&gas_error)
    });
    Ok(to_value(
        &get_order_quotes(order, block_number, rpc_url.to_string(), gas_value)
            .await
            .map(|v| {
                v.into_iter()
                    .map(BatchOrderQuotesResponse::from)
                    .collect::<Vec<_>>()
            })?,
    )?)
}
