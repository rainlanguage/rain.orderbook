use crate::{error::Error, BatchQuoteSpec as MainBatchQuoteSpec, QuoteSpec as MainQuoteSpec};
use crate::{BatchQuoteTarget as MainBatchQuoteTarget, QuoteTarget as MainQuoteTarget};
use alloy_primitives::{
    hex::{encode_prefixed, FromHex},
    keccak256, Address, U256,
};
use alloy_sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV4::OrderV3 as MainOrderV3;
use rain_orderbook_subgraph_client::utils::make_order_id;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::str::FromStr;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{convert::*, describe::WasmDescribe, JsValue, UnwrapThrowExt};

mod impls;

// a serializer fn for serializing Vec<u8> as Uint8Array for js
fn bytes_serilializer<S: serde::Serializer>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

/// Holds quoted order max output and ratio
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct OrderQuoteValue {
    pub max_output: String,
    pub ratio: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EvaluableV3 {
    pub interpreter: String,
    pub store: String,
    #[tsify(type = "Uint8Array")]
    #[serde(serialize_with = "bytes_serilializer")]
    pub bytecode: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct IO {
    pub token: String,
    pub decimals: u8,
    pub vault_id: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct OrderV3 {
    pub owner: String,
    pub evaluable: EvaluableV3,
    pub valid_inputs: Vec<IO>,
    pub valid_outputs: Vec<IO>,
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SignedContextV1 {
    pub signer: String,
    pub context: Vec<String>,
    #[tsify(type = "Uint8Array")]
    #[serde(serialize_with = "bytes_serilializer")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Quote {
    pub order: OrderV3,
    #[serde(rename = "inputIOIndex")]
    pub input_io_index: String,
    #[serde(rename = "outputIOIndex")]
    pub output_io_index: String,
    pub signed_context: Vec<SignedContextV1>,
}

/// A quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct QuoteTarget {
    pub quote_config: Quote,
    pub orderbook: String,
}

/// Batch quote target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[serde(transparent)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct BatchQuoteTarget(pub Vec<QuoteTarget>);

/// A quote target specifier, where the order details need to be fetched from a
/// source (such as subgraph) to build a [QuoteTarget] out of it
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct BatchQuoteSpec(pub Vec<QuoteSpec>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
pub enum QuoteResult {
    Ok(OrderQuoteValue),
    Err(String),
}

/// Get the order hash of an order
#[wasm_bindgen(js_name = "getOrderHash")]
pub fn get_order_hash(order: &OrderV3) -> String {
    encode_prefixed(keccak256(MainOrderV3::from(order.clone()).abi_encode()))
}

/// Get subgraph represented "order_id" of a QuoteTarget
#[wasm_bindgen(js_name = "getId")]
pub fn get_id(orderbook: &str, order_hash: &str) -> String {
    let orderbook = Address::from_hex(orderbook).expect_throw("invalid orderbook address");
    let order_hash = U256::from_str(order_hash).expect_throw("invalid order hash");
    encode_prefixed(make_order_id(orderbook, order_hash))
}

/// Quotes the target on the given rpc url
#[wasm_bindgen(js_name = "doQuoteTargets")]
pub async fn do_quote_targets(
    quote_targets: &BatchQuoteTarget,
    rpc_url: &str,
    block_number: Option<u64>,
    multicall_address: Option<String>,
) -> Result<JsValue, Error> {
    let multicall_address =
        multicall_address.map(|v| Address::from_hex(v).expect_throw("invalid multicall address"));
    let quote_targets: Vec<MainQuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| MainQuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = MainBatchQuoteTarget(quote_targets);
    match batch_quote_target
        .do_quote(rpc_url, block_number, multicall_address)
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
/// Those orders that are not found from subgraph are excluded from quoting,
/// and final result also leaves their place in the array as None
#[wasm_bindgen(js_name = "doQuoteSpecs")]
pub async fn do_quote(
    quote_specs: &BatchQuoteSpec,
    subgraph_url: &str,
    rpc_url: &str,
    block_number: Option<u64>,
    multicall_address: Option<String>,
) -> Result<JsValue, Error> {
    let multicall_address =
        multicall_address.map(|v| Address::from_hex(v).expect_throw("invalid multicall address"));
    let quote_specs: Vec<MainQuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| MainQuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = MainBatchQuoteSpec(quote_specs);
    match batch_quote_spec
        .do_quote(subgraph_url, rpc_url, block_number, multicall_address)
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
/// Those specifiers that were not in the subgraph are returned as None
/// in the resturning array
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
                .map(|e| match e {
                    Some(quote_value) => Some(QuoteTarget::from(quote_value)),
                    None => None,
                })
                .collect::<Vec<_>>(),
        )?),
    }
}
