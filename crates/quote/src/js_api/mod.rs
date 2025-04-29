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
use rain_orderbook_subgraph_client::{types::common::SgOrder, utils::make_order_id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
#[serde(untagged)]
pub enum QuoteResultEnum {
    Success {
        value: OrderQuoteValue,
        #[tsify(type = "undefined")]
        error: Option<String>,
    },
    Err {
        #[tsify(type = "undefined")]
        value: Option<OrderQuoteValue>,
        error: String,
    },
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
#[cfg(target_family = "wasm")]
#[wasm_export(
    js_name = "doQuoteTargets",
    unchecked_return_type = "DoQuoteTargetsResult"
)]
pub async fn do_quote_targets(
    quote_targets: BatchQuoteTarget,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
    multicall_address: Option<String>,
) -> Result<DoQuoteTargetsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let quote_targets: Vec<QuoteTarget> = quote_targets
        .0
        .iter()
        .map(|v| QuoteTarget::from(v.clone()))
        .collect();
    let batch_quote_target = BatchQuoteTarget(quote_targets);

    let quotes = batch_quote_target
        .do_quote(&rpc_url, block_number, gas_value, multicall_address)
        .await?;

    let res = quotes
        .into_iter()
        .map(|q| match q {
            Ok(v) => QuoteResultEnum::Success {
                value: v,
                error: None,
            },
            Err(e) => QuoteResultEnum::Err {
                value: None,
                error: e.to_string(),
            },
        })
        .collect();

    Ok(DoQuoteTargetsResult(res))
}

/// Given a subgraph url, will fetch the order details from the subgraph and
/// then quotes them using the given rpc url.
/// Resolves with array of OrderQuoteValue object or a string error
#[cfg(target_family = "wasm")]
#[wasm_export(js_name = "doQuoteSpecs", unchecked_return_type = "DoQuoteSpecsResult")]
pub async fn do_quote_specs(
    quote_specs: BatchQuoteSpec,
    subgraph_url: String,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
    multicall_address: Option<String>,
) -> Result<DoQuoteSpecsResult, QuoteBindingsError> {
    let multicall_address = multicall_address
        .map(|v| Address::from_hex(v))
        .transpose()?;
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let quotes = batch_quote_spec
        .do_quote(
            &subgraph_url,
            &rpc_url,
            block_number,
            gas_value,
            multicall_address,
        )
        .await?;

    let res = quotes
        .into_iter()
        .map(|q| match q {
            Ok(v) => QuoteResultEnum::Success {
                value: v,
                error: None,
            },
            Err(e) => QuoteResultEnum::Err {
                value: None,
                error: e.to_string(),
            },
        })
        .collect();

    Ok(DoQuoteSpecsResult(res))
}

/// Given a subgraph url, will fetch orders details and returns their
/// respective quote targets.
/// Resolves with array of QuoteTarget object or undefined if no result
/// found on subgraph for a specific spec
#[cfg(target_family = "wasm")]
#[wasm_export(
    js_name = "getQuoteTargetFromSubgraph",
    unchecked_return_type = "QuoteTargetResult"
)]
pub async fn get_batch_quote_target_from_subgraph(
    quote_specs: BatchQuoteSpec,
    subgraph_url: String,
) -> Result<QuoteTargetResult, QuoteBindingsError> {
    let quote_specs: Vec<QuoteSpec> = quote_specs
        .0
        .iter()
        .map(|v| QuoteSpec::from(v.clone()))
        .collect();
    let batch_quote_spec = BatchQuoteSpec(quote_specs);

    let quote_targets = batch_quote_spec
        .get_batch_quote_target_from_subgraph(&subgraph_url)
        .await?;
    Ok(QuoteTargetResult(quote_targets))
}

/// Get the quote for an order
/// Resolves with a BatchOrderQuotesResponse object
#[cfg(target_family = "wasm")]
#[wasm_export(
    js_name = "getOrderQuote",
    unchecked_return_type = "DoOrderQuoteResult"
)]
pub async fn get_order_quote(
    order: Vec<SgOrder>,
    rpc_url: String,
    block_number: Option<u64>,
    gas: Option<String>,
) -> Result<DoOrderQuoteResult, QuoteBindingsError> {
    let gas_value = gas.map(|v| U256::from_str(&v)).transpose()?;
    let order_quotes = get_order_quotes(order, block_number, rpc_url, gas_value).await?;
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
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

impl QuoteBindingsError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::QuoteError(e) => format!("Quote error: {}", e),
            Self::FromHexError(e) => format!("Failed to parse orderbook address: {}", e),
            Self::U256ParseError(e) => format!("Failed to parse u256 value: {}", e),
            Self::JsError(msg) => format!("A JavaScript error occurred: {}", msg),
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

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::js_api::get_id;
        use alloy::{
            hex::encode_prefixed,
            primitives::{Address, U256},
        };
        use rain_orderbook_subgraph_client::utils::make_order_id;
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_get_id() {
            let orderbook =
                Address::from_str("0x0123456789123456789123456789123456789123").unwrap();
            let order_hash = U256::from(30);
            let expected_id = encode_prefixed(make_order_id(orderbook, order_hash));

            let res = get_id(&orderbook.to_string(), &order_hash.to_string()).unwrap();
            assert_eq!(res, expected_id);

            let err = get_id("invalid-hex", &order_hash.to_string()).unwrap_err();
            assert_eq!(err.to_string(), "Odd number of digits");
            assert_eq!(
                err.to_readable_msg(),
                "Failed to parse orderbook address: Odd number of digits"
            );

            let err = get_id(&orderbook.to_string(), "invalid-hash").unwrap_err();
            assert_eq!(err.to_string(), "digit 18 is out of range for base 10");
            assert_eq!(
                err.to_readable_msg(),
                "Failed to parse u256 value: digit 18 is out of range for base 10"
            );
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod quote_non_wasm_tests {
        use httpmock::MockServer;

        #[tokio::test]
        async fn test_do_quote_targets() {
            let rpc_server = MockServer::start_async().await;
        }
    }
}
