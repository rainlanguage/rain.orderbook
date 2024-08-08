use super::*;
use crate::QuoteTarget as MainQuoteTarget;
use crate::{error::Error, OrderQuoteValue as MainOrderQuoteValue, QuoteSpec as MainQuoteSpec};
use alloy_primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::js_api::Quote;
use rain_orderbook_bindings::IOrderBookV4::{
    Quote as MainQuote, SignedContextV1 as MainSignedContextV1,
};
use serde_wasm_bindgen::{from_value, to_value};
use std::str::FromStr;
use wasm_bindgen::{
    describe::{inform, WasmDescribeVector, VECTOR},
    JsValue, UnwrapThrowExt,
};

impl From<OrderQuoteValue> for MainOrderQuoteValue {
    fn from(value: OrderQuoteValue) -> Self {
        MainOrderQuoteValue {
            max_output: U256::from_str(&value.max_output).expect_throw("invalid max output value"),
            ratio: U256::from_str(&value.ratio).expect_throw("invalid ratio value"),
        }
    }
}
impl From<MainOrderQuoteValue> for OrderQuoteValue {
    fn from(value: MainOrderQuoteValue) -> Self {
        OrderQuoteValue {
            max_output: encode_prefixed(value.max_output.to_be_bytes_vec()),
            ratio: encode_prefixed(value.ratio.to_be_bytes_vec()),
        }
    }
}

impl From<QuoteTarget> for MainQuoteTarget {
    fn from(value: QuoteTarget) -> Self {
        MainQuoteTarget {
            orderbook: Address::from_hex(value.orderbook).expect_throw("invalid orderbook address"),
            quote_config: MainQuote::from(value.quote_config),
        }
    }
}
impl From<MainQuoteTarget> for QuoteTarget {
    fn from(value: MainQuoteTarget) -> Self {
        QuoteTarget {
            orderbook: encode_prefixed(value.orderbook),
            quote_config: Quote::from(value.quote_config),
        }
    }
}

impl From<QuoteSpec> for MainQuoteSpec {
    fn from(value: QuoteSpec) -> Self {
        MainQuoteSpec {
            order_hash: U256::from_str(&value.order_hash).expect_throw("invalid order hash"),
            input_io_index: value.input_io_index,
            output_io_index: value.output_io_index,
            signed_context: value
                .signed_context
                .iter()
                .map(|v| MainSignedContextV1::from(v.clone()))
                .collect(),
            orderbook: Address::from_hex(value.orderbook).expect_throw("invalid orderbook address"),
        }
    }
}

impl From<crate::QuoteResult> for super::QuoteResult {
    fn from(value: crate::QuoteResult) -> Self {
        match value {
            Ok(v) => super::QuoteResult::Ok(v.into()),
            Err(e) => super::QuoteResult::Err(e.to_string()),
        }
    }
}

impl LongRefFromWasmAbi for BatchQuoteTarget {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<BatchQuoteTarget>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(BatchQuoteTarget::from_abi(js))
    }
}

impl LongRefFromWasmAbi for BatchQuoteSpec {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<BatchQuoteSpec>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(BatchQuoteSpec::from_abi(js))
    }
}

impl LongRefFromWasmAbi for QuoteTarget {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteTarget>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteTarget::from_abi(js))
    }
}
impl RefFromWasmAbi for QuoteTarget {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteTarget>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteTarget::from_abi(js))
    }
}
impl VectorIntoWasmAbi for QuoteTarget {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for QuoteTarget {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for QuoteTarget {
    fn describe_vector() {
        inform(VECTOR);
        QuoteTarget::describe();
    }
}
impl From<QuoteTarget> for JsValue {
    fn from(value: QuoteTarget) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for QuoteTarget {
    type Error = Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        Ok(from_value(value)?)
    }
}

impl LongRefFromWasmAbi for QuoteSpec {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteSpec>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteSpec::from_abi(js))
    }
}
impl RefFromWasmAbi for QuoteSpec {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteSpec>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteSpec::from_abi(js))
    }
}
impl VectorIntoWasmAbi for QuoteSpec {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for QuoteSpec {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for QuoteSpec {
    fn describe_vector() {
        inform(VECTOR);
        QuoteSpec::describe();
    }
}
impl From<QuoteSpec> for JsValue {
    fn from(value: QuoteSpec) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for QuoteSpec {
    type Error = Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        Ok(from_value(value)?)
    }
}

impl LongRefFromWasmAbi for QuoteResult {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteResult>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteResult::from_abi(js))
    }
}
impl RefFromWasmAbi for QuoteResult {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<QuoteResult>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(QuoteResult::from_abi(js))
    }
}
impl VectorIntoWasmAbi for QuoteResult {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for QuoteResult {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for QuoteResult {
    fn describe_vector() {
        inform(VECTOR);
        QuoteResult::describe();
    }
}
impl From<QuoteResult> for JsValue {
    fn from(value: QuoteResult) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for QuoteResult {
    type Error = Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        Ok(from_value(value)?)
    }
}
