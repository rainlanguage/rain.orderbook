use crate::IOrderBookV4::{
    EvaluableV3 as MainEvaluableV3, OrderV3 as MainOrderV3, Quote as MainQuote,
    SignedContextV1 as MainSignedContextV1, IO as MainIO,
};
use alloy_primitives::{
    hex::{encode_prefixed, FromHex},
    keccak256, Address, U256,
};
use alloy_sol_types::SolValue;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::str::FromStr;
use tsify::Tsify;
use wasm_bindgen::{
    convert::*,
    describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR},
    prelude::*,
    JsValue, UnwrapThrowExt,
};

// a serializer fn for serializing Vec<u8> as Uint8Array for js
fn bytes_serilializer<S: serde::Serializer>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
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

/// Get the order hash of an order
#[wasm_bindgen(js_name = "getOrderHash")]
pub fn get_order_hash(order: &OrderV3) -> String {
    encode_prefixed(keccak256(MainOrderV3::from(order.clone()).abi_encode()))
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

impl RefFromWasmAbi for OrderV3 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<OrderV3>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(OrderV3::from_abi(js))
    }
}
impl LongRefFromWasmAbi for OrderV3 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<OrderV3>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(OrderV3::from_abi(js))
    }
}
impl VectorIntoWasmAbi for OrderV3 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for OrderV3 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for OrderV3 {
    fn describe_vector() {
        inform(VECTOR);
        OrderV3::describe();
    }
}
impl From<OrderV3> for JsValue {
    fn from(value: OrderV3) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for OrderV3 {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        from_value(value)
    }
}

impl RefFromWasmAbi for EvaluableV3 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<EvaluableV3>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(EvaluableV3::from_abi(js))
    }
}
impl LongRefFromWasmAbi for EvaluableV3 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<EvaluableV3>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(EvaluableV3::from_abi(js))
    }
}
impl VectorIntoWasmAbi for EvaluableV3 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for EvaluableV3 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for EvaluableV3 {
    fn describe_vector() {
        inform(VECTOR);
        EvaluableV3::describe();
    }
}
impl From<EvaluableV3> for JsValue {
    fn from(value: EvaluableV3) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for EvaluableV3 {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        from_value(value)
    }
}

impl RefFromWasmAbi for IO {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<IO>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(IO::from_abi(js))
    }
}
impl LongRefFromWasmAbi for IO {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<IO>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(IO::from_abi(js))
    }
}
impl VectorIntoWasmAbi for IO {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for IO {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for IO {
    fn describe_vector() {
        inform(VECTOR);
        IO::describe();
    }
}
impl From<IO> for JsValue {
    fn from(value: IO) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for IO {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        from_value(value)
    }
}

impl RefFromWasmAbi for SignedContextV1 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<SignedContextV1>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(SignedContextV1::from_abi(js))
    }
}
impl LongRefFromWasmAbi for SignedContextV1 {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<SignedContextV1>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(SignedContextV1::from_abi(js))
    }
}
impl VectorIntoWasmAbi for SignedContextV1 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for SignedContextV1 {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for SignedContextV1 {
    fn describe_vector() {
        inform(VECTOR);
        SignedContextV1::describe();
    }
}
impl From<SignedContextV1> for JsValue {
    fn from(value: SignedContextV1) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for SignedContextV1 {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        from_value(value)
    }
}

impl RefFromWasmAbi for Quote {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<Quote>;
    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(Quote::from_abi(js))
    }
}
impl LongRefFromWasmAbi for Quote {
    type Abi = <JsValue as RefFromWasmAbi>::Abi;
    type Anchor = Box<Quote>;
    unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
        Box::new(Quote::from_abi(js))
    }
}
impl VectorIntoWasmAbi for Quote {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
        js_value_vector_into_abi(vector)
    }
}
impl VectorFromWasmAbi for Quote {
    type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
    unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
        js_value_vector_from_abi(js)
    }
}
impl WasmDescribeVector for Quote {
    fn describe_vector() {
        inform(VECTOR);
        Quote::describe();
    }
}
impl From<Quote> for JsValue {
    fn from(value: Quote) -> Self {
        to_value(&value).unwrap_throw()
    }
}
impl TryFromJsValue for Quote {
    type Error = serde_wasm_bindgen::Error;
    fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
        from_value(value)
    }
}
