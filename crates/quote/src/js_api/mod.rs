use crate::{error::Error, BatchQuoteSpec, QuoteSpec};
use crate::{
    get_order_quotes, BatchOrderQuotesResponse, BatchQuoteTarget, OrderQuoteValue, QuoteTarget,
};
use alloy::hex::FromHexError;
use alloy::primitives::ruint::ParseError;
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::wasm_traits::TryIntoU256;
use rain_orderbook_subgraph_client::{types::common::SgOrder, utils::make_order_id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum QuoteResultEnum {
    Ok(OrderQuoteValue),
    Err(String),
}
impl_wasm_traits!(QuoteResultEnum);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoQuoteTargetsResult(pub Vec<QuoteResultEnum>);
impl_wasm_traits!(DoQuoteTargetsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoQuoteSpecsResult(pub Vec<QuoteResultEnum>);
impl_wasm_traits!(DoQuoteSpecsResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTargetResult(
    #[tsify(type = "(QuoteTarget | undefined)[]")] pub Vec<Option<QuoteTarget>>,
);
impl_wasm_traits!(QuoteTargetResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DoOrderQuoteResult(pub Vec<BatchOrderQuotesResponse>);
impl_wasm_traits!(DoOrderQuoteResult);

/// Get subgraph represented "order_id" of a QuoteTarget
#[wasm_export(js_name = "getId", unchecked_return_type = "string")]
pub fn get_id(orderbook: &str, order_hash: &str) -> Result<String, QuoteBindingsError> {
    let orderbook = Address::from_hex(orderbook)?;
    let order_hash = U256::from_str(order_hash)?;
    Ok(encode_prefixed(make_order_id(orderbook, order_hash)))
}

/// Quotes the target on the given rpc url
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_export(
    js_name = "doQuoteTargets",
    unchecked_return_type = "DoQuoteTargetsResult"
)]
pub async fn do_quote_targets(
    quote_targets: &BatchQuoteTarget,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
    multicall_address: Option<String>,
) -> Result<DoQuoteTargetsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| v.try_into_u256()).transpose()?;
    let quote_targets: Vec<QuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| QuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = BatchQuoteTarget(quote_targets);

    let mut res: Vec<QuoteResultEnum> = vec![];

    let quotes = batch_quote_target
        .do_quote(rpc_url, block_number, gas_value, multicall_address)
        .await?;

    for quote in quotes {
        match quote {
            Ok(v) => {
                res.push(QuoteResultEnum::Ok(v));
            }
            Err(e) => {
                res.push(QuoteResultEnum::Err(e.to_string()));
            }
        }
    }

    Ok(DoQuoteTargetsResult(res))
}

/// Given a subgraph url, will fetch the order details from the subgraph and
/// then quotes them using the given rpc url.
/// Resolves with array of OrderQuoteValue object or a string error
#[wasm_export(js_name = "doQuoteSpecs", unchecked_return_type = "DoQuoteSpecsResult")]
pub async fn do_quote_specs(
    quote_specs: &BatchQuoteSpec,
    subgraph_url: &str,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
    multicall_address: Option<String>,
) -> Result<DoQuoteSpecsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| v.try_into_u256()).transpose()?;
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let mut res: Vec<QuoteResultEnum> = vec![];

    let quotes = batch_quote_spec
        .do_quote(
            subgraph_url,
            rpc_url,
            block_number,
            gas_value,
            multicall_address,
        )
        .await?;

    for quote in quotes {
        match quote {
            Ok(v) => {
                res.push(QuoteResultEnum::Ok(v));
            }
            Err(e) => {
                res.push(QuoteResultEnum::Err(e.to_string()));
            }
        }
    }

    Ok(DoQuoteSpecsResult(res))
}

/// Given a subgraph url, will fetch orders details and returns their
/// respective quote targets.
/// Resolves with array of QuoteTarget object or undefined if no result
/// found on subgraph for a specific spec
#[wasm_export(
    js_name = "getQuoteTargetFromSubgraph",
    unchecked_return_type = "QuoteTargetResult"
)]
pub async fn get_batch_quote_target_from_subgraph(
    quote_specs: &BatchQuoteSpec,
    subgraph_url: &str,
) -> Result<QuoteTargetResult, QuoteBindingsError> {
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let quote_targets = batch_quote_spec
        .get_batch_quote_target_from_subgraph(subgraph_url)
        .await?;
    Ok(QuoteTargetResult(quote_targets))
}

/// Get the quote for an order
/// Resolves with a BatchOrderQuotesResponse object
#[wasm_export(
    js_name = "getOrderQuote",
    unchecked_return_type = "DoOrderQuoteResult"
)]
pub async fn get_order_quote(
    order: Vec<SgOrder>,
    rpc_url: &str,
    block_number: Option<u64>,
    gas: Option<js_sys::BigInt>,
) -> Result<DoOrderQuoteResult, QuoteBindingsError> {
    let gas_value = gas.map(|v| v.try_into_u256()).transpose()?;
    let order_quotes =
        get_order_quotes(order, block_number, rpc_url.to_string(), gas_value).await?;
    Ok(DoOrderQuoteResult(order_quotes))
}

#[derive(Error, Debug)]
pub enum QuoteBindingsError {
    #[error(transparent)]
    QuoteError(#[from] Error),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    U256ParseError(#[from] ParseError),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

impl QuoteBindingsError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::QuoteError(e) => format!("Quote error: {}", e),
            Self::FromHexError(e) => format!("Failed to parse orderbook address: {}", e),
            Self::U256ParseError(e) => format!("Failed to parse u256 value: {}", e),
            Self::SerdeWasmBindgenError(err) => format!("Data serialization error: {}", err),
        }
    }
}

impl From<QuoteBindingsError> for JsValue {
    fn from(value: QuoteBindingsError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<QuoteBindingsError> for WasmEncodedError {
    fn from(value: QuoteBindingsError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}
