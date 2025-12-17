use crate::raindex_client::RaindexError;
use crate::take_orders::MinReceiveMode;
use alloy::primitives::Address;
use rain_math_float::Float;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersRequest {
    pub taker: String,
    pub chain_id: u32,
    pub sell_token: String,
    pub buy_token: String,
    pub buy_amount: String,
    pub price_cap: String,
    pub min_receive_mode: MinReceiveMode,
}
impl_wasm_traits!(TakeOrdersRequest);

#[derive(Debug, Clone)]
pub(crate) struct ParsedTakeOrdersRequest {
    pub taker: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub buy_amount: Float,
    pub price_cap: Float,
    pub min_receive_mode: MinReceiveMode,
}

pub(crate) fn parse_request_from_struct(
    req: &TakeOrdersRequest,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let taker = Address::from_str(&req.taker)?;
    let sell_token = Address::from_str(&req.sell_token)?;
    let buy_token = Address::from_str(&req.buy_token)?;
    let buy_amount = Float::parse(req.buy_amount.clone())?;
    let price_cap = Float::parse(req.price_cap.clone())?;

    let zero = Float::zero()?;
    if buy_amount.lte(zero)? {
        return Err(RaindexError::NonPositiveBuyAmount);
    }
    if price_cap.lt(zero)? {
        return Err(RaindexError::NegativePriceCap);
    }

    Ok(ParsedTakeOrdersRequest {
        taker,
        sell_token,
        buy_token,
        buy_amount,
        price_cap,
        min_receive_mode: req.min_receive_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    fn make_request(
        taker: &str,
        sell_token: &str,
        buy_token: &str,
        buy_amount: &str,
        price_cap: &str,
        min_receive_mode: MinReceiveMode,
    ) -> TakeOrdersRequest {
        TakeOrdersRequest {
            taker: taker.to_string(),
            chain_id: 1,
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            buy_amount: buy_amount.to_string(),
            price_cap: price_cap.to_string(),
            min_receive_mode,
        }
    }

    #[test]
    fn test_parse_request_valid_inputs() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "2.5",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.sell_token,
            address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(
            parsed.buy_token,
            address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert!(parsed
            .buy_amount
            .eq(Float::parse("100".to_string()).unwrap())
            .unwrap());
        assert!(parsed
            .price_cap
            .eq(Float::parse("2.5".to_string()).unwrap())
            .unwrap());
        assert!(matches!(parsed.min_receive_mode, MinReceiveMode::Partial));
    }

    #[test]
    fn test_parse_request_invalid_sell_token() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "not-an-address",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "2",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_token() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "invalid",
            "100",
            "2",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_amount() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "not-a-number",
            "2",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_invalid_price_cap() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "not-a-number",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_exact_mode() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "50",
            "1.5",
            MinReceiveMode::Exact,
        );
        let result = parse_request_from_struct(&req);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(matches!(parsed.min_receive_mode, MinReceiveMode::Exact));
    }

    #[test]
    fn test_parse_request_zero_buy_amount() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "0",
            "2",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::NonPositiveBuyAmount)));
    }

    #[test]
    fn test_parse_request_negative_buy_amount() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "-10",
            "2",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::NonPositiveBuyAmount)));
    }

    #[test]
    fn test_parse_request_zero_price_cap_is_valid() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "0",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.price_cap.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_parse_request_negative_price_cap() {
        let req = make_request(
            "0x1111111111111111111111111111111111111111",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "-1",
            MinReceiveMode::Partial,
        );
        let result = parse_request_from_struct(&req);

        assert!(matches!(result, Err(RaindexError::NegativePriceCap)));
    }
}
