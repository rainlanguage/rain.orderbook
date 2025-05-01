use crate::IOrderBookV4::{
    EvaluableV3, OrderV3, Quote, SignedContextV1, TakeOrderConfigV3, TakeOrdersConfigV3, IO,
};
use wasm_bindgen_utils::{impl_custom_tsify, impl_wasm_traits, prelude::*};

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
    signedContext: SignedContextV1[];
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
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_utils::prelude::js_sys::JsString;

    #[wasm_bindgen_test]
    fn test_io_tsify() {
        let js_io = to_js_value(&IO::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("token").unwrap().js_in(&js_io));
        assert!(JsString::from_str("decimals").unwrap().js_in(&js_io));
        assert!(JsString::from_str("vaultId").unwrap().js_in(&js_io));
    }

    #[wasm_bindgen_test]
    fn test_quote_tsify() {
        let js_quote = to_js_value(&Quote::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("order").unwrap().js_in(&js_quote));
        assert!(JsString::from_str("inputIOIndex").unwrap().js_in(&js_quote));
        assert!(JsString::from_str("outputIOIndex")
            .unwrap()
            .js_in(&js_quote));
        assert!(JsString::from_str("signedContext")
            .unwrap()
            .js_in(&js_quote));
    }

    #[wasm_bindgen_test]
    fn test_orderv3_tsify() {
        let js_order = to_js_value(&OrderV3::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("owner").unwrap().js_in(&js_order));
        assert!(JsString::from_str("evaluable").unwrap().js_in(&js_order));
        assert!(JsString::from_str("validInputs").unwrap().js_in(&js_order));
        assert!(JsString::from_str("validOutputs").unwrap().js_in(&js_order));
        assert!(JsString::from_str("nonce").unwrap().js_in(&js_order));
    }

    #[wasm_bindgen_test]
    fn test_evaluablev3_tsify() {
        let js_evaluable = to_js_value(&EvaluableV3::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("interpreter")
            .unwrap()
            .js_in(&js_evaluable));
        assert!(JsString::from_str("store").unwrap().js_in(&js_evaluable));
        assert!(JsString::from_str("bytecode").unwrap().js_in(&js_evaluable));
    }

    #[wasm_bindgen_test]
    fn test_signed_contextv1_tsify() {
        let js_signed_context = to_js_value(&SignedContextV1::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("signer")
            .unwrap()
            .js_in(&js_signed_context));
        assert!(JsString::from_str("context")
            .unwrap()
            .js_in(&js_signed_context));
        assert!(JsString::from_str("signature")
            .unwrap()
            .js_in(&js_signed_context));
    }

    #[wasm_bindgen_test]
    fn test_take_order_config_v3_tsify() {
        let js_take_order_config = to_js_value(&TakeOrderConfigV3::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("order")
            .unwrap()
            .js_in(&js_take_order_config));
        assert!(JsString::from_str("inputIOIndex")
            .unwrap()
            .js_in(&js_take_order_config));
        assert!(JsString::from_str("outputIOIndex")
            .unwrap()
            .js_in(&js_take_order_config));
        assert!(JsString::from_str("signedContext")
            .unwrap()
            .js_in(&js_take_order_config));
    }

    #[wasm_bindgen_test]
    fn test_take_orders_config_v3_tsify() {
        let js_take_orders_config = to_js_value(&TakeOrdersConfigV3::default()).unwrap();
        // validate serialized props match the tsify definition
        assert!(JsString::from_str("minimumInput")
            .unwrap()
            .js_in(&js_take_orders_config));
        assert!(JsString::from_str("maximumInput")
            .unwrap()
            .js_in(&js_take_orders_config));
        assert!(JsString::from_str("maximumIORatio")
            .unwrap()
            .js_in(&js_take_orders_config));
        assert!(JsString::from_str("orders")
            .unwrap()
            .js_in(&js_take_orders_config));
        assert!(JsString::from_str("data")
            .unwrap()
            .js_in(&js_take_orders_config));
    }
}
