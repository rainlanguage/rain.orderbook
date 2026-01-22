use crate::raindex_client::RaindexError;
use crate::take_orders::{ParsedTakeOrdersMode, TakeOrdersMode};
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
    pub mode: TakeOrdersMode,
    pub amount: String,
    pub price_cap: String,
}
impl_wasm_traits!(TakeOrdersRequest);

#[derive(Debug, Clone)]
pub(crate) struct ParsedTakeOrdersRequest {
    pub taker: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    pub mode: ParsedTakeOrdersMode,
    pub price_cap: Float,
}

pub(crate) fn parse_request(
    request: &TakeOrdersRequest,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let taker = Address::from_str(&request.taker)?;
    let sell_token = Address::from_str(&request.sell_token)?;
    let buy_token = Address::from_str(&request.buy_token)?;
    if sell_token == buy_token {
        return Err(RaindexError::SameTokenPair);
    }
    let price_cap = Float::parse(request.price_cap.to_string())?;
    let mode = ParsedTakeOrdersMode::parse(request.mode, &request.amount)?;

    let zero = Float::zero()?;
    if price_cap.lt(zero)? {
        return Err(RaindexError::NegativePriceCap);
    }

    Ok(ParsedTakeOrdersRequest {
        taker,
        sell_token,
        buy_token,
        mode,
        price_cap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    const TEST_TAKER: &str = "0x1111111111111111111111111111111111111111";

    #[test]
    fn test_parse_request_buy_up_to_mode() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "2.5".to_string(),
        });

        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(
            req.taker,
            address!("1111111111111111111111111111111111111111")
        );
        assert_eq!(
            req.sell_token,
            address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(
            req.buy_token,
            address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert_eq!(req.mode.mode, TakeOrdersMode::BuyUpTo);
        assert!(req
            .mode
            .amount
            .eq(Float::parse("100".to_string()).unwrap())
            .unwrap());
        assert!(req
            .price_cap
            .eq(Float::parse("2.5".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_parse_request_buy_exact_mode() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyExact,
            amount: "50".to_string(),
            price_cap: "1.5".to_string(),
        });

        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.mode.mode, TakeOrdersMode::BuyExact);
        assert!(req
            .mode
            .amount
            .eq(Float::parse("50".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_parse_request_spend_up_to_mode() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::SpendUpTo,
            amount: "100".to_string(),
            price_cap: "2.5".to_string(),
        });

        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.mode.mode, TakeOrdersMode::SpendUpTo);
        assert!(req
            .mode
            .amount
            .eq(Float::parse("100".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_parse_request_spend_exact_mode() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::SpendExact,
            amount: "75".to_string(),
            price_cap: "3.0".to_string(),
        });

        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.mode.mode, TakeOrdersMode::SpendExact);
        assert!(req
            .mode
            .amount
            .eq(Float::parse("75".to_string()).unwrap())
            .unwrap());
    }

    #[test]
    fn test_parse_request_invalid_taker() {
        let result = parse_request(&TakeOrdersRequest {
            taker: "not-an-address".to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_sell_token() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "not-an-address".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_token() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "invalid".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_amount() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "not-a-number".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_invalid_price_cap() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "not-a-number".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_zero_amount() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "0".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::NonPositiveAmount)));
    }

    #[test]
    fn test_parse_request_negative_amount() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::SpendExact,
            amount: "-10".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::NonPositiveAmount)));
    }

    #[test]
    fn test_parse_request_zero_price_cap_is_valid() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "0".to_string(),
        });

        assert!(result.is_ok());
        let req = result.unwrap();
        assert!(req.price_cap.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_parse_request_negative_price_cap() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "-1".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::NegativePriceCap)));
    }

    #[test]
    fn test_parse_request_same_token_pair() {
        let result = parse_request(&TakeOrdersRequest {
            taker: TEST_TAKER.to_string(),
            chain_id: 1,
            sell_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            buy_token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            mode: TakeOrdersMode::BuyUpTo,
            amount: "100".to_string(),
            price_cap: "2".to_string(),
        });

        assert!(matches!(result, Err(RaindexError::SameTokenPair)));
    }
}
