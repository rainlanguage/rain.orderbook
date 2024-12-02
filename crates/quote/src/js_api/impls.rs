use super::*;
use crate::QuoteTarget as MainQuoteTarget;
use crate::{BatchOrderQuotesResponse as MainBatchOrderQuotesResponse, Pair as MainPair};
use crate::{OrderQuoteValue as MainOrderQuoteValue, QuoteSpec as MainQuoteSpec};
use alloy::primitives::{
    hex::{encode_prefixed, FromHex},
    Address, U256,
};
use rain_orderbook_bindings::js_api::Quote;
use rain_orderbook_bindings::IOrderBookV4::{
    Quote as MainQuote, SignedContextV1 as MainSignedContextV1,
};
use std::str::FromStr;

impl From<OrderQuoteValue> for MainOrderQuoteValue {
    fn from(value: OrderQuoteValue) -> Self {
        let mut max_output_error = "max output, ".to_string();
        let mut ratio_error = "ratio, ".to_string();
        MainOrderQuoteValue {
            max_output: U256::from_str(&value.max_output)
                .inspect_err(|e| max_output_error.push_str(&e.to_string()))
                .expect_throw(&max_output_error),
            ratio: U256::from_str(&value.ratio)
                .inspect_err(|e| ratio_error.push_str(&e.to_string()))
                .expect_throw(&ratio_error),
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
        let mut orderbook_error = "orderbook address, ".to_string();
        MainQuoteTarget {
            orderbook: Address::from_hex(&value.orderbook)
                .inspect_err(|e| orderbook_error.push_str(&e.to_string()))
                .expect_throw(&orderbook_error),
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
        let mut orderbook_error = "orderbook address, ".to_string();
        let mut order_hash_error = "order hash, ".to_string();
        MainQuoteSpec {
            order_hash: U256::from_str(&value.order_hash)
                .inspect_err(|e| order_hash_error.push_str(&e.to_string()))
                .expect_throw(&order_hash_error),
            input_io_index: value.input_io_index,
            output_io_index: value.output_io_index,
            signed_context: value
                .signed_context
                .into_iter()
                .map(MainSignedContextV1::from)
                .collect(),
            orderbook: Address::from_hex(&value.orderbook)
                .inspect_err(|e| orderbook_error.push_str(&e.to_string()))
                .expect_throw(&orderbook_error),
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

impl From<MainPair> for Pair {
    fn from(value: MainPair) -> Self {
        Pair {
            pair_name: value.pair_name,
            input_index: value.input_index,
            output_index: value.output_index,
        }
    }
}
impl From<Pair> for MainPair {
    fn from(value: Pair) -> Self {
        MainPair {
            pair_name: value.pair_name,
            input_index: value.input_index,
            output_index: value.output_index,
        }
    }
}

impl From<MainBatchOrderQuotesResponse> for BatchOrderQuotesResponse {
    fn from(value: MainBatchOrderQuotesResponse) -> Self {
        let mut block_number_error = "block number, ".to_string();
        BatchOrderQuotesResponse {
            pair: value.pair.into(),
            block_number: u64::try_from(value.block_number)
                .inspect_err(|e| block_number_error.push_str(&e.to_string()))
                .expect_throw(&block_number_error),
            data: value.data.map(OrderQuoteValue::from),
            success: value.success,
            error: value.error,
        }
    }
}
impl From<BatchOrderQuotesResponse> for MainBatchOrderQuotesResponse {
    fn from(value: BatchOrderQuotesResponse) -> Self {
        let mut max_output_error = "max output, ".to_string();
        let mut ratio_error = "ratio, ".to_string();
        MainBatchOrderQuotesResponse {
            pair: value.pair.into(),
            block_number: U256::from(value.block_number),
            data: value.data.map(|e| MainOrderQuoteValue {
                max_output: U256::from_str(&e.max_output)
                    .inspect_err(|e| max_output_error.push_str(&e.to_string()))
                    .expect_throw(&max_output_error),
                ratio: U256::from_str(&e.ratio)
                    .inspect_err(|e| ratio_error.push_str(&e.to_string()))
                    .expect_throw(&ratio_error),
            }),
            success: value.success,
            error: value.error,
        }
    }
}

impl_all_wasm_traits!(Pair);
impl_all_wasm_traits!(QuoteSpec);
impl_all_wasm_traits!(QuoteTarget);
impl_all_wasm_traits!(QuoteResult);
impl_all_wasm_traits!(BatchQuoteSpec);
impl_all_wasm_traits!(OrderQuoteValue);
impl_all_wasm_traits!(BatchQuoteTarget);
impl_all_wasm_traits!(BatchOrderQuotesResponse);

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
    #[should_panic]
    fn test_order_quote_value_unhappy() {
        let main_order_quote_value = MainOrderQuoteValue::default();
        let mut order_quote_value = OrderQuoteValue::from(main_order_quote_value);
        order_quote_value.ratio = "qwe".to_string();
        let _ = MainOrderQuoteValue::from(order_quote_value);
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
    #[should_panic]
    fn test_quote_spec_unhappy() {
        let main_quote_spec = MainQuoteSpec::default();
        let mut quote_spec = QuoteSpec::from(main_quote_spec);
        quote_spec.order_hash = "abcd".to_string();
        let _ = MainQuoteSpec::from(quote_spec);
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

    #[wasm_bindgen_test]
    #[should_panic]
    fn test_quote_target_unhappy() {
        let main_quote_target = MainQuoteTarget::default();
        let mut quote_target = QuoteTarget::from(main_quote_target);
        quote_target.quote_config.order.owner = "0x1234".to_string();
        let _ = MainQuoteTarget::from(quote_target);
    }

    #[wasm_bindgen_test]
    fn test_batch_order_quotes_response_roundtrip() {
        let main_batch_order_quotes_response = MainBatchOrderQuotesResponse::default();
        let batch_order_quotes_response =
            BatchOrderQuotesResponse::from(main_batch_order_quotes_response.clone());
        let expected: MainBatchOrderQuotesResponse =
            MainBatchOrderQuotesResponse::from(batch_order_quotes_response.clone());
        assert_eq!(main_batch_order_quotes_response, expected);

        let main_batch_order_quotes_response =
            MainBatchOrderQuotesResponse::from(batch_order_quotes_response.clone());
        let expected = BatchOrderQuotesResponse::from(main_batch_order_quotes_response.clone());
        assert_eq!(batch_order_quotes_response, expected);
    }

    #[wasm_bindgen_test]
    fn test_pair_roundtrip() {
        let main_pair = MainPair::default();
        let pair = Pair::from(main_pair.clone());
        let expected = MainPair::from(pair.clone());
        assert_eq!(main_pair, expected);

        let main_pair = MainPair::from(pair.clone());
        let expected = Pair::from(main_pair.clone());
        assert_eq!(pair, expected);
    }
}
