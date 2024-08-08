use super::*;
use crate::QuoteTarget as MainQuoteTarget;
use crate::{error::Error, OrderQuoteValue as MainOrderQuoteValue, QuoteSpec as MainQuoteSpec};
use alloy_primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::IOrderBookV4::{
    EvaluableV3 as MainEvaluableV3, OrderV3 as MainOrderV3, Quote as MainQuote,
    SignedContextV1 as MainSignedContextV1, IO as MainIO,
};
use serde_wasm_bindgen::{from_value, to_value};
use std::mem::ManuallyDrop;
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

impl From<EvaluableV3> for MainEvaluableV3 {
    fn from(value: EvaluableV3) -> Self {
        MainEvaluableV3 {
            interpreter: Address::from_hex(value.interpreter)
                .expect_throw("invalid interpreter address"),
            store: Address::from_hex(value.store).expect_throw("invalid store address"),
            bytecode: value.bytecode,
        }
    }
}
impl From<MainEvaluableV3> for EvaluableV3 {
    fn from(value: MainEvaluableV3) -> Self {
        EvaluableV3 {
            interpreter: encode_prefixed(value.interpreter),
            store: encode_prefixed(value.store),
            bytecode: value.bytecode,
        }
    }
}

impl From<IO> for MainIO {
    fn from(value: IO) -> Self {
        MainIO {
            token: Address::from_hex(value.token).expect_throw("invalid token address"),
            vaultId: U256::from_str(&value.vault_id).expect_throw("invalid vault id value"),
            decimals: value.decimals,
        }
    }
}
impl From<MainIO> for IO {
    fn from(value: MainIO) -> Self {
        IO {
            token: encode_prefixed(value.token),
            vault_id: encode_prefixed(value.vaultId.to_be_bytes_vec()),
            decimals: value.decimals,
        }
    }
}

impl From<OrderV3> for MainOrderV3 {
    fn from(value: OrderV3) -> Self {
        let valid_inputs = value
            .valid_inputs
            .iter()
            .map(|v| MainIO::from(v.clone()))
            .collect();
        let valid_outputs = value
            .valid_outputs
            .iter()
            .map(|v| MainIO::from(v.clone()))
            .collect();
        MainOrderV3 {
            owner: Address::from_hex(value.owner).expect_throw("invalid owner address"),
            evaluable: MainEvaluableV3::from(value.evaluable),
            validInputs: valid_inputs,
            validOutputs: valid_outputs,
            nonce: U256::from_str(&value.nonce)
                .expect_throw("invalid nonce value")
                .into(),
        }
    }
}
impl From<MainOrderV3> for OrderV3 {
    fn from(value: MainOrderV3) -> Self {
        OrderV3 {
            owner: encode_prefixed(value.owner),
            evaluable: value.evaluable.into(),
            nonce: encode_prefixed(value.nonce),
            valid_inputs: value.validInputs.into_iter().map(IO::from).collect(),
            valid_outputs: value.validOutputs.into_iter().map(IO::from).collect(),
        }
    }
}

impl From<SignedContextV1> for MainSignedContextV1 {
    fn from(value: SignedContextV1) -> Self {
        let context = value
            .context
            .iter()
            .map(|v| U256::from_str(v).expect_throw("invalid context value"))
            .collect();
        MainSignedContextV1 {
            signer: Address::from_hex(value.signer).expect_throw("invalid token address"),
            context,
            signature: value.signature,
        }
    }
}
impl From<MainSignedContextV1> for SignedContextV1 {
    fn from(value: MainSignedContextV1) -> Self {
        SignedContextV1 {
            signer: encode_prefixed(value.signer),
            signature: value.signature,
            context: value
                .context
                .into_iter()
                .map(|v| encode_prefixed(v.to_be_bytes_vec()))
                .collect(),
        }
    }
}

impl From<Quote> for MainQuote {
    fn from(value: Quote) -> Self {
        MainQuote {
            order: MainOrderV3::from(value.order),
            inputIOIndex: U256::from_str(&value.input_io_index)
                .expect_throw("invalid input io index"),
            outputIOIndex: U256::from_str(&value.output_io_index)
                .expect_throw("invalid output io index"),
            signedContext: value
                .signed_context
                .iter()
                .map(|v| MainSignedContextV1::from(v.clone()))
                .collect(),
        }
    }
}
impl From<MainQuote> for Quote {
    fn from(value: MainQuote) -> Self {
        Quote {
            order: OrderV3::from(value.order),
            input_io_index: encode_prefixed(value.inputIOIndex.to_be_bytes_vec()),
            output_io_index: encode_prefixed(value.outputIOIndex.to_be_bytes_vec()),
            signed_context: value
                .signedContext
                .into_iter()
                .map(SignedContextV1::from)
                .collect(),
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

impl RefFromWasmAbi for OrderV3 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = ManuallyDrop<OrderV3>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        ManuallyDrop::new(OrderV3::from_abi(js))
    }
}

impl RefFromWasmAbi for QuoteTarget {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    // type Anchor = ManuallyDrop<QuoteTarget>;
    type Anchor = Box<QuoteTarget>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        // ManuallyDrop::new(QuoteTarget::from_abi(js))
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
