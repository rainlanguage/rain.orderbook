use crate::raindex_client::RaindexError;
use crate::take_orders::{ParsedTakeOrdersMode, TakeOrdersMode};
use alloy::primitives::Address;
use rain_math_float::Float;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct ParsedTakeOrdersRequest {
    pub sell_token: Address,
    pub buy_token: Address,
    pub mode: ParsedTakeOrdersMode,
    pub price_cap: Float,
}

pub(crate) fn parse_request(
    sell_token: &str,
    buy_token: &str,
    mode: TakeOrdersMode,
    amount: &str,
    price_cap: &str,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let sell_token = Address::from_str(sell_token)?;
    let buy_token = Address::from_str(buy_token)?;
    if sell_token == buy_token {
        return Err(RaindexError::SameTokenPair);
    }
    let price_cap = Float::parse(price_cap.to_string())?;
    let mode = ParsedTakeOrdersMode::parse(mode, amount)?;

    let zero = Float::zero()?;
    if price_cap.lt(zero)? {
        return Err(RaindexError::NegativePriceCap);
    }

    Ok(ParsedTakeOrdersRequest {
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

    #[test]
    fn test_parse_request_buy_up_to_mode() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "100",
            "2.5",
        );

        assert!(result.is_ok());
        let req = result.unwrap();
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
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyExact,
            "50",
            "1.5",
        );

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
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::SpendUpTo,
            "100",
            "2.5",
        );

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
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::SpendExact,
            "75",
            "3.0",
        );

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
    fn test_parse_request_invalid_sell_token() {
        let result = parse_request(
            "not-an-address",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "100",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_token() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "invalid",
            TakeOrdersMode::BuyUpTo,
            "100",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "not-a-number",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_invalid_price_cap() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "100",
            "not-a-number",
        );

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_zero_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "0",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::NonPositiveAmount)));
    }

    #[test]
    fn test_parse_request_negative_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::SpendExact,
            "-10",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::NonPositiveAmount)));
    }

    #[test]
    fn test_parse_request_zero_price_cap_is_valid() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "100",
            "0",
        );

        assert!(result.is_ok());
        let req = result.unwrap();
        assert!(req.price_cap.eq(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_parse_request_negative_price_cap() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            TakeOrdersMode::BuyUpTo,
            "100",
            "-1",
        );

        assert!(matches!(result, Err(RaindexError::NegativePriceCap)));
    }

    #[test]
    fn test_parse_request_same_token_pair() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            TakeOrdersMode::BuyUpTo,
            "100",
            "2",
        );

        assert!(matches!(result, Err(RaindexError::SameTokenPair)));
    }
}
