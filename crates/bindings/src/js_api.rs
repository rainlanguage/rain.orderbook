use crate::IOrderBookV4::{
    EvaluableV3 as MainEvaluableV3, OrderV3 as MainOrderV3, Quote as MainQuote,
    SignedContextV1 as MainSignedContextV1, IO as MainIO,
};
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    keccak256, Address, U256,
};
use alloy::sol_types::SolValue;
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
            interpreter: match Address::from_hex(&value.interpreter) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.interpreter).expect_throw(&e.to_string()),
            },
            store: match Address::from_hex(&value.store) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.store).expect_throw(&e.to_string()),
            },
            bytecode: value.bytecode.into(),
        }
    }
}
impl From<MainEvaluableV3> for EvaluableV3 {
    fn from(value: MainEvaluableV3) -> Self {
        EvaluableV3 {
            interpreter: encode_prefixed(value.interpreter),
            store: encode_prefixed(value.store),
            bytecode: value.bytecode.into(),
        }
    }
}

impl From<IO> for MainIO {
    fn from(value: IO) -> Self {
        MainIO {
            token: match Address::from_hex(&value.token) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.token).expect_throw(&e.to_string()),
            },
            vaultId: match U256::from_str(&value.vault_id) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.vault_id).expect_throw(&e.to_string()),
            },
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
        MainOrderV3 {
            owner: match Address::from_hex(&value.owner) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.owner).expect_throw(&e.to_string()),
            },
            evaluable: MainEvaluableV3::from(value.evaluable),
            validInputs: value.valid_inputs.into_iter().map(MainIO::from).collect(),
            validOutputs: value.valid_outputs.into_iter().map(MainIO::from).collect(),
            nonce: match U256::from_str(&value.nonce) {
                Ok(v) => v.into(),
                Err(e) => U256::from_str(&value.nonce)
                    .expect_throw(&e.to_string())
                    .into(),
            },
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
            .map(|v| match U256::from_str(v) {
                Ok(e) => e,
                Err(e) => U256::from_str(v).expect_throw(&e.to_string()),
            })
            .collect();
        MainSignedContextV1 {
            signer: match Address::from_hex(&value.signer) {
                Ok(v) => v,
                Err(e) => Address::from_hex(&value.signer).expect_throw(&e.to_string()),
            },
            context,
            signature: value.signature.into(),
        }
    }
}
impl From<MainSignedContextV1> for SignedContextV1 {
    fn from(value: MainSignedContextV1) -> Self {
        SignedContextV1 {
            signer: encode_prefixed(value.signer),
            signature: value.signature.to_vec(),
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
            inputIOIndex: match U256::from_str(&value.input_io_index) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.input_io_index).expect_throw(&e.to_string()),
            },
            outputIOIndex: match U256::from_str(&value.output_io_index) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.output_io_index).expect_throw(&e.to_string()),
            },
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

#[macro_export]
macro_rules! impl_wasm_traits {
    ($struct_name:ident) => {
        impl RefFromWasmAbi for $struct_name {
            type Abi = <JsValue as RefFromWasmAbi>::Abi;
            type Anchor = Box<$struct_name>;
            unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new($struct_name::from_abi(js))
            }
        }
        impl LongRefFromWasmAbi for $struct_name {
            type Abi = <JsValue as RefFromWasmAbi>::Abi;
            type Anchor = Box<$struct_name>;
            unsafe fn long_ref_from_abi(js: Self::Abi) -> Self::Anchor {
                Box::new($struct_name::from_abi(js))
            }
        }
        impl VectorIntoWasmAbi for $struct_name {
            type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
            fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                js_value_vector_into_abi(vector)
            }
        }
        impl VectorFromWasmAbi for $struct_name {
            type Abi = <Box<[JsValue]> as IntoWasmAbi>::Abi;
            unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                js_value_vector_from_abi(js)
            }
        }
        impl WasmDescribeVector for $struct_name {
            fn describe_vector() {
                inform(VECTOR);
                $struct_name::describe();
            }
        }
        impl From<$struct_name> for JsValue {
            fn from(value: $struct_name) -> Self {
                to_value(&value).unwrap_throw()
            }
        }
        impl TryFromJsValue for $struct_name {
            type Error = serde_wasm_bindgen::Error;
            fn try_from_js_value(value: JsValue) -> Result<Self, Self::Error> {
                from_value(value)
            }
        }
    };
}

impl_wasm_traits!(IO);
impl_wasm_traits!(EvaluableV3);
impl_wasm_traits!(OrderV3);
impl_wasm_traits!(SignedContextV1);
impl_wasm_traits!(Quote);

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_io_roundtrip() {
        let main_io = MainIO::default();
        let io = IO::from(main_io.clone());
        let expected = MainIO::from(io.clone());
        assert_eq!(main_io, expected);

        let main_io = MainIO::from(io.clone());
        let expected = IO::from(main_io.clone());
        assert_eq!(io, expected);
    }

    #[wasm_bindgen_test]
    fn test_evaluable_roundtrip() {
        let main_evaluable = MainEvaluableV3::default();
        let evaluable = EvaluableV3::from(main_evaluable.clone());
        let expected = MainEvaluableV3::from(evaluable.clone());
        assert_eq!(main_evaluable, expected);

        let main_evaluable = MainEvaluableV3::from(evaluable.clone());
        let expected = EvaluableV3::from(main_evaluable.clone());
        assert_eq!(evaluable, expected);
    }

    #[wasm_bindgen_test]
    fn test_order_roundtrip() {
        let main_order = MainOrderV3::default();
        let order = OrderV3::from(main_order.clone());
        let expected = MainOrderV3::from(order.clone());
        assert_eq!(main_order, expected);

        let main_order = MainOrderV3::from(order.clone());
        let expected = OrderV3::from(main_order.clone());
        assert_eq!(order, expected);
    }

    #[wasm_bindgen_test]
    fn test_signed_context_roundtrip() {
        let main_signed_context = MainSignedContextV1::default();
        let signed_context = SignedContextV1::from(main_signed_context.clone());
        let expected = MainSignedContextV1::from(signed_context.clone());
        assert_eq!(main_signed_context, expected);

        let main_signed_context = MainSignedContextV1::from(signed_context.clone());
        let expected = SignedContextV1::from(main_signed_context.clone());
        assert_eq!(signed_context, expected);
    }

    #[wasm_bindgen_test]
    fn test_quote_roundtrip() {
        let main_quote = MainQuote::default();
        let quote = Quote::from(main_quote.clone());
        let expected = MainQuote::from(quote.clone());
        assert_eq!(main_quote, expected);

        let main_quote = MainQuote::from(quote.clone());
        let expected = Quote::from(main_quote.clone());
        assert_eq!(quote, expected);
    }
}
