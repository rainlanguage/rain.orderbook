use super::*;
use crate::QuoteTarget as MainQuoteTarget;
use crate::{OrderQuoteValue as MainOrderQuoteValue, QuoteSpec as MainQuoteSpec};
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::impl_wasm_traits;
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
            max_output: match U256::from_str(&value.max_output) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.max_output).expect_throw(&e.to_string()),
            },
            ratio: match U256::from_str(&value.ratio) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.ratio).expect_throw(&e.to_string()),
            },
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
            orderbook: match Address::from_hex(&value.orderbook) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.orderbook).expect_throw(&e.to_string()),
            },
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
            order_hash: match U256::from_str(&value.order_hash) {
                Ok(v) => v,
                Err(e) => U256::from_str(&value.order_hash).expect_throw(&e.to_string()),
            },
            input_io_index: value.input_io_index,
            output_io_index: value.output_io_index,
            signed_context: value
                .signed_context
                .into_iter()
                .map(MainSignedContextV1::from)
                .collect(),
            orderbook: match Address::from_hex(&value.orderbook) {
                Ok(v) => v,
                Err(e) => Address::from_hex(value.orderbook).expect_throw(&e.to_string()),
            },
        }
    }
}
impl From<MainQuoteSpec> for QuoteSpec {
    fn from(value: MainQuoteSpec) -> Self {
        QuoteSpec {
            orderbook: encode_prefixed(value.orderbook),
            signed_context: value
                .signed_context
                .into_iter()
                .map(SignedContextV1::from)
                .collect(),
            input_io_index: value.input_io_index,
            output_io_index: value.output_io_index,
            order_hash: encode_prefixed(value.order_hash.to_be_bytes_vec()),
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

impl_wasm_traits!(QuoteSpec);
impl_wasm_traits!(QuoteTarget);
impl_wasm_traits!(QuoteResult);

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_order_quote_value_roundtrip() {
        let main_order_quote_value = MainOrderQuoteValue::default();
        let order_quote_value = OrderQuoteValue::from(main_order_quote_value);
        let expected = MainOrderQuoteValue::from(order_quote_value.clone());
        assert_eq!(main_order_quote_value, expected);

        let main_order_quote_value = MainOrderQuoteValue::from(order_quote_value.clone());
        let expected = OrderQuoteValue::from(main_order_quote_value);
        assert_eq!(order_quote_value, expected);
    }

    #[wasm_bindgen_test]
    fn test_quote_spec_roundtrip() {
        let main_quote_spec = MainQuoteSpec::default();
        let quote_spec = QuoteSpec::from(main_quote_spec.clone());
        let expected = MainQuoteSpec::from(quote_spec.clone());
        assert_eq!(main_quote_spec, expected);

        let main_quote_spec = MainQuoteSpec::from(quote_spec.clone());
        let expected = QuoteSpec::from(main_quote_spec.clone());
        assert_eq!(quote_spec, expected);
    }

    #[wasm_bindgen_test]
    fn test_quote_target_roundtrip() {
        let main_quote_target = MainQuoteTarget::default();
        let quote_target = QuoteTarget::from(main_quote_target.clone());
        let expected = MainQuoteTarget::from(quote_target.clone());
        assert_eq!(main_quote_target, expected);

        let main_quote_target = MainQuoteTarget::from(quote_target.clone());
        let expected = QuoteTarget::from(main_quote_target.clone());
        assert_eq!(quote_target, expected);
    }
}
