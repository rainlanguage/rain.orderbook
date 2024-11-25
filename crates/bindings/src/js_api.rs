use crate::wasm_traits::prelude::*;
use crate::IOrderBookV4::{
    takeOrders2Call, EvaluableV3 as MainEvaluableV3, OrderV3 as MainOrderV3, Quote as MainQuote,
    SignedContextV1 as MainSignedContextV1, TakeOrderConfigV3 as MainTakeOrderConfigV3,
    TakeOrdersConfigV3 as MainTakeOrdersConfigV3, IO as MainIO,
};
use alloy::sol_types::SolValue;
use alloy::{
    primitives::{
        hex::{decode, encode_prefixed, FromHex},
        keccak256 as main_keccak256, Address, U256,
    },
    sol_types::SolCall,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// a serializer fn for serializing Vec<u8> as Uint8Array for js
fn bytes_serilializer<S: serde::Serializer>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(val)
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct EvaluableV3 {
    pub interpreter: String,
    pub store: String,
    #[tsify(type = "Uint8Array")]
    #[serde(serialize_with = "bytes_serilializer")]
    pub bytecode: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct IO {
    pub token: String,
    pub decimals: u8,
    pub vault_id: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderV3 {
    pub owner: String,
    pub evaluable: EvaluableV3,
    pub valid_inputs: Vec<IO>,
    pub valid_outputs: Vec<IO>,
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct SignedContextV1 {
    pub signer: String,
    pub context: Vec<String>,
    #[tsify(type = "Uint8Array")]
    #[serde(serialize_with = "bytes_serilializer")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub order: OrderV3,
    #[serde(rename = "inputIOIndex")]
    pub input_io_index: u8,
    #[serde(rename = "outputIOIndex")]
    pub output_io_index: u8,
    pub signed_context: Vec<SignedContextV1>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrderConfigV3 {
    order: OrderV3,
    #[serde(rename = "inputIOIndex")]
    input_io_index: u8,
    #[serde(rename = "outputIOIndex")]
    output_io_index: u8,
    signed_context: Vec<SignedContextV1>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersConfigV3 {
    minimum_input: String,
    maximum_input: String,
    #[serde(rename = "maximumIORatio")]
    maximum_io_ratio: String,
    orders: Vec<TakeOrderConfigV3>,
    #[tsify(type = "Uint8Array")]
    #[serde(serialize_with = "bytes_serilializer")]
    data: Vec<u8>,
}

/// Get the order hash of an order
#[wasm_bindgen(js_name = "getOrderHash")]
pub fn get_order_hash(order: &OrderV3) -> String {
    encode_prefixed(main_keccak256(
        MainOrderV3::from(order.clone()).abi_encode(),
    ))
}

/// Get takeOrders2() calldata
#[wasm_bindgen(js_name = "getTakeOrders2Calldata")]
pub fn get_take_orders2_calldata(take_orders_config: &TakeOrdersConfigV3) -> js_sys::Uint8Array {
    takeOrders2Call {
        config: take_orders_config.clone().into(),
    }
    .abi_encode()
    .as_slice()
    .into()
}

/// calculates keccak256 of the given bytes
#[wasm_bindgen]
pub fn keccak256(bytes: &[u8]) -> String {
    encode_prefixed(main_keccak256(bytes))
}

/// calculate keccak256 of a hex string
#[wasm_bindgen(js_name = "keccak256HexString")]
pub fn keccak256_hex_string(hex_string: &str) -> String {
    let mut err = "".to_string();
    encode_prefixed(main_keccak256(
        decode(hex_string)
            .inspect_err(|e| err.push_str(&e.to_string()))
            .expect_throw(&err),
    ))
}

impl From<EvaluableV3> for MainEvaluableV3 {
    fn from(value: EvaluableV3) -> Self {
        let mut interpreter_error = "interpreter address, ".to_string();
        let mut store_error = "store address, ".to_string();
        MainEvaluableV3 {
            interpreter: Address::from_hex(&value.interpreter)
                .inspect_err(|e| interpreter_error.push_str(&e.to_string()))
                .expect_throw(&interpreter_error),
            store: Address::from_hex(&value.store)
                .inspect_err(|e| store_error.push_str(&e.to_string()))
                .expect_throw(&store_error),
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
        let mut token_error = "token address, ".to_string();
        let mut vault_id_error = "vault id, ".to_string();
        MainIO {
            token: Address::from_hex(&value.token)
                .inspect_err(|e| token_error.push_str(&e.to_string()))
                .expect_throw(&token_error),
            vaultId: U256::from_str(&value.vault_id)
                .inspect_err(|e| vault_id_error.push_str(&e.to_string()))
                .expect_throw(&vault_id_error),
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
        let mut owner_error = "owner address, ".to_string();
        let mut nonce_error = "nonce value, ".to_string();
        MainOrderV3 {
            owner: Address::from_hex(&value.owner)
                .inspect_err(|e| owner_error.push_str(&e.to_string()))
                .expect_throw(&owner_error),
            evaluable: MainEvaluableV3::from(value.evaluable),
            validInputs: value.valid_inputs.into_iter().map(MainIO::from).collect(),
            validOutputs: value.valid_outputs.into_iter().map(MainIO::from).collect(),
            nonce: U256::from_str(&value.nonce)
                .inspect_err(|e| nonce_error.push_str(&e.to_string()))
                .expect_throw(&nonce_error)
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
        let mut context_error = "context, ".to_string();
        let mut signer_error = "signer address, ".to_string();
        let context = value
            .context
            .iter()
            .map(|v| {
                U256::from_str(&v)
                    .inspect_err(|e| context_error.push_str(&e.to_string()))
                    .expect_throw(&context_error)
            })
            .collect();
        MainSignedContextV1 {
            signer: Address::from_hex(&value.signer)
                .inspect_err(|e| signer_error.push_str(&e.to_string()))
                .expect_throw(&signer_error),
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
            inputIOIndex: U256::from(value.input_io_index),
            outputIOIndex: U256::from(value.output_io_index),
            signedContext: value
                .signed_context
                .into_iter()
                .map(MainSignedContextV1::from)
                .collect(),
        }
    }
}
impl From<MainQuote> for Quote {
    fn from(value: MainQuote) -> Self {
        let mut input_io_index_error = "input io index, ".to_string();
        let mut output_io_index_error = "output io index, ".to_string();
        Quote {
            order: OrderV3::from(value.order),
            input_io_index: value
                .inputIOIndex
                .try_into()
                .inspect_err(|e: &alloy::primitives::ruint::FromUintError<u8>| {
                    input_io_index_error.push_str(&e.to_string())
                })
                .expect_throw(&input_io_index_error),
            output_io_index: value
                .outputIOIndex
                .try_into()
                .inspect_err(|e: &alloy::primitives::ruint::FromUintError<u8>| {
                    output_io_index_error.push_str(&e.to_string())
                })
                .expect_throw(&output_io_index_error),
            signed_context: value
                .signedContext
                .into_iter()
                .map(SignedContextV1::from)
                .collect(),
        }
    }
}

impl From<TakeOrderConfigV3> for MainTakeOrderConfigV3 {
    fn from(value: TakeOrderConfigV3) -> Self {
        MainTakeOrderConfigV3 {
            order: MainOrderV3::from(value.order),
            inputIOIndex: U256::from(value.input_io_index),
            outputIOIndex: U256::from(value.output_io_index),
            signedContext: value
                .signed_context
                .into_iter()
                .map(MainSignedContextV1::from)
                .collect(),
        }
    }
}
impl From<MainTakeOrderConfigV3> for TakeOrderConfigV3 {
    fn from(value: MainTakeOrderConfigV3) -> Self {
        let mut input_io_index_error = "input io index, ".to_string();
        let mut output_io_index_error = "output io index, ".to_string();
        TakeOrderConfigV3 {
            order: OrderV3::from(value.order),
            input_io_index: value
                .inputIOIndex
                .try_into()
                .inspect_err(|e: &alloy::primitives::ruint::FromUintError<u8>| {
                    input_io_index_error.push_str(&e.to_string())
                })
                .expect_throw(&input_io_index_error),
            output_io_index: value
                .outputIOIndex
                .try_into()
                .inspect_err(|e: &alloy::primitives::ruint::FromUintError<u8>| {
                    output_io_index_error.push_str(&e.to_string())
                })
                .expect_throw(&output_io_index_error),
            signed_context: value
                .signedContext
                .into_iter()
                .map(SignedContextV1::from)
                .collect(),
        }
    }
}

impl From<TakeOrdersConfigV3> for MainTakeOrdersConfigV3 {
    fn from(value: TakeOrdersConfigV3) -> Self {
        let mut minimum_input_err = "minimum input value, ".to_string();
        let mut maximum_input_err = "maximum input value, ".to_string();
        let mut maximum_io_ratio_err = "maximum io ratio value, ".to_string();
        MainTakeOrdersConfigV3 {
            minimumInput: U256::from_str(&value.minimum_input)
                .inspect_err(|e| minimum_input_err.push_str(&e.to_string()))
                .expect_throw(&minimum_input_err),
            maximumInput: U256::from_str(&value.maximum_input)
                .inspect_err(|e| maximum_input_err.push_str(&e.to_string()))
                .expect_throw(&maximum_input_err),
            maximumIORatio: U256::from_str(&value.maximum_io_ratio)
                .inspect_err(|e| maximum_io_ratio_err.push_str(&e.to_string()))
                .expect_throw(&maximum_io_ratio_err),
            orders: value
                .orders
                .into_iter()
                .map(MainTakeOrderConfigV3::from)
                .collect(),
            data: value.data.into(),
        }
    }
}
impl From<MainTakeOrdersConfigV3> for TakeOrdersConfigV3 {
    fn from(value: MainTakeOrdersConfigV3) -> Self {
        TakeOrdersConfigV3 {
            maximum_input: encode_prefixed(value.minimumInput.to_be_bytes_vec()),
            minimum_input: encode_prefixed(value.minimumInput.to_be_bytes_vec()),
            maximum_io_ratio: encode_prefixed(value.maximumIORatio.to_be_bytes_vec()),
            orders: value
                .orders
                .into_iter()
                .map(TakeOrderConfigV3::from)
                .collect(),
            data: value.data.to_vec(),
        }
    }
}

mod impls {
    use crate::impl_all_wasm_traits;

    impl_all_wasm_traits!(super::IO);
    impl_all_wasm_traits!(super::Quote);
    impl_all_wasm_traits!(super::OrderV3);
    impl_all_wasm_traits!(super::EvaluableV3);
    impl_all_wasm_traits!(super::SignedContextV1);
    impl_all_wasm_traits!(super::TakeOrderConfigV3);
    impl_all_wasm_traits!(super::TakeOrdersConfigV3);
}

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
    #[should_panic]
    fn test_io_unhappy() {
        let main_io = MainIO::default();
        let mut io = IO::from(main_io);
        io.token = "0x1234".to_string();
        let _ = MainIO::from(io);
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
    #[should_panic]
    fn test_evaluable_unhappy() {
        let main_evaluable = MainEvaluableV3::default();
        let mut evaluable = EvaluableV3::from(main_evaluable);
        evaluable.interpreter = "0x1234".to_string();
        let _ = MainEvaluableV3::from(evaluable);
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
    #[should_panic]
    fn test_order_unhappy() {
        let main_order = MainOrderV3::default();
        let mut order = OrderV3::from(main_order);
        order.owner = "0x1234".to_string();
        let _ = MainOrderV3::from(order);
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
    #[should_panic]
    fn test_signed_context_unhappy() {
        let main_signed_context = MainSignedContextV1::default();
        let mut signed_context = SignedContextV1::from(main_signed_context);
        signed_context.signer = "0x1234".to_string();
        let _ = MainSignedContextV1::from(signed_context);
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

    #[wasm_bindgen_test]
    #[should_panic]
    fn test_quote_unhappy() {
        let main_quote = MainQuote::default();
        let mut quote = Quote::from(main_quote);
        quote.order.nonce = "abcd".to_string();
        let _ = MainQuote::from(quote);
    }

    #[wasm_bindgen_test]
    fn test_take_order_config_roundtrip() {
        let main_take_order_config = MainTakeOrderConfigV3::default();
        let take_order_config = TakeOrderConfigV3::from(main_take_order_config.clone());
        let expected = MainTakeOrderConfigV3::from(take_order_config.clone());
        assert_eq!(main_take_order_config, expected);

        let main_take_order_config = MainTakeOrderConfigV3::from(take_order_config.clone());
        let expected = TakeOrderConfigV3::from(main_take_order_config.clone());
        assert_eq!(take_order_config, expected);
    }

    #[wasm_bindgen_test]
    #[should_panic]
    fn test_take_order_config_unhappy() {
        let main_take_order_config = MainTakeOrderConfigV3::default();
        let mut take_order_config = TakeOrderConfigV3::from(main_take_order_config);
        take_order_config.order.nonce = "abcd".to_string();
        let _ = MainTakeOrderConfigV3::from(take_order_config);
    }

    #[wasm_bindgen_test]
    fn test_take_orders_config_roundtrip() {
        let main_take_orders_config = MainTakeOrdersConfigV3::default();
        let take_orders_config = TakeOrdersConfigV3::from(main_take_orders_config.clone());
        let expected = MainTakeOrdersConfigV3::from(take_orders_config.clone());
        assert_eq!(main_take_orders_config, expected);

        let main_take_orders_config = MainTakeOrdersConfigV3::from(take_orders_config.clone());
        let expected = TakeOrdersConfigV3::from(main_take_orders_config.clone());
        assert_eq!(take_orders_config, expected);
    }

    #[wasm_bindgen_test]
    #[should_panic]
    fn test_take_orders_config_unhappy() {
        let main_take_orders_config = MainTakeOrdersConfigV3::default();
        let mut take_orders_config = TakeOrdersConfigV3::from(main_take_orders_config);
        take_orders_config.maximum_input = "abcd".to_string();
        let _ = MainTakeOrdersConfigV3::from(take_orders_config);
    }

    #[wasm_bindgen_test]
    fn test_take_orders_calldata() {
        let main_take_orders_config = MainTakeOrdersConfigV3::default();
        let take_orders_config = TakeOrdersConfigV3::from(main_take_orders_config.clone());
        let result = get_take_orders2_calldata(&take_orders_config);
        let expected = takeOrders2Call {
            config: main_take_orders_config,
        }
        .abi_encode();
        assert_eq!(result.to_vec(), expected);
    }

    #[wasm_bindgen_test]
    fn test_keccak256() {
        let bytes = vec![1, 2];
        let result = keccak256(&bytes);
        let expected =
            "0x22ae6da6b482f9b1b19b0b897c3fd43884180a1c5ee361e1107a1bc635649dda".to_string();
        assert_eq!(result, expected);
    }

    #[wasm_bindgen_test]
    fn test_keccak256_hex_string() {
        let hex_string = "0x0102";
        let result = keccak256_hex_string(&hex_string);
        let expected =
            "0x22ae6da6b482f9b1b19b0b897c3fd43884180a1c5ee361e1107a1bc635649dda".to_string();
        assert_eq!(result, expected);
    }
}
