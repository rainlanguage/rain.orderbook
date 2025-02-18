use crate::IOrderBookV4::{
    takeOrders2Call, EvaluableV3, OrderV3, Quote, SignedContextV1, TakeOrderConfigV3,
    TakeOrdersConfigV3, IO,
};
use alloy::sol_types::SolValue;
use alloy::{
    primitives::{
        hex::{decode, encode_prefixed},
        keccak256 as main_keccak256,
    },
    sol_types::SolCall,
};
use wasm_bindgen_utils::{impl_custom_tsify, impl_wasm_traits, prelude::*};

/// Get the order hash of an order
#[wasm_bindgen(js_name = "getOrderHash")]
pub fn get_order_hash(order: &OrderV3) -> String {
    encode_prefixed(main_keccak256(order.abi_encode()))
}

/// Get takeOrders2() calldata
#[wasm_bindgen(js_name = "getTakeOrders2Calldata")]
pub fn get_take_orders2_calldata(take_orders_config: TakeOrdersConfigV3) -> js_sys::Uint8Array {
    takeOrders2Call {
        config: take_orders_config,
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

impl_wasm_traits!(IO);
impl_wasm_traits!(Quote);
impl_wasm_traits!(OrderV3);
impl_wasm_traits!(EvaluableV3);
impl_wasm_traits!(SignedContextV1);
impl_wasm_traits!(TakeOrderConfigV3);
impl_wasm_traits!(TakeOrdersConfigV3);

impl_custom_tsify!(
    IO,
    "export interface IO {
    token: string;
    decimals: number;
    vaultId: string;
}"
);
impl_custom_tsify!(
    Quote,
    "export interface Quote {
    order: OrderV3;
    inputIOIndex: string;
    outputIOIndex: string;
    signedContext: SignedContextV1[];
}"
);
impl_custom_tsify!(
    OrderV3,
    "export interface OrderV3 {
    owner: string;
    evaluable: EvaluableV3;
    validInputs: IO[];
    validOutputs: IO[];
    nonce: string;
}"
);
impl_custom_tsify!(
    EvaluableV3,
    "export interface EvaluableV3 {
    interpreter: string;
    store: string;
    bytecode: string;
}"
);
impl_custom_tsify!(
    SignedContextV1,
    "export interface SignedContextV1 {
    signer: string;
    context: string[];
    signature: string;
}"
);
impl_custom_tsify!(
    TakeOrderConfigV3,
    "export interface TakeOrderConfigV3 {
    order: OrderV3;
    inputIOIndex: string;
    outputIOIndex: string;
    signedConetxt: SignedContextV1[];
}"
);
impl_custom_tsify!(
    TakeOrdersConfigV3,
    "export interface TakeOrdersConfigV3 {
    minimumInput: string;
    maximumInput: string;
    maximumIORatio: string;
    orders: TakeOrderConfigV3[];
    data: string;
}"
);

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_take_orders_calldata() {
        let take_orders_config = TakeOrdersConfigV3::default();
        let result = get_take_orders2_calldata(take_orders_config.clone());
        let expected = takeOrders2Call {
            config: take_orders_config,
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
