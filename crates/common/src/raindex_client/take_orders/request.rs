use crate::raindex_client::RaindexError;
use crate::take_orders::MinReceiveMode;
use alloy::primitives::Address;
use rain_math_float::Float;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct ParsedTakeOrdersRequest {
    pub sell_token: Address,
    pub buy_token: Address,
    pub buy_amount: Float,
    pub price_cap: Float,
    pub min_receive_mode: MinReceiveMode,
}

pub(crate) fn parse_request(
    sell_token: &str,
    buy_token: &str,
    buy_amount: &str,
    price_cap: &str,
    min_receive_mode: MinReceiveMode,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let sell_token = Address::from_str(sell_token)?;
    let buy_token = Address::from_str(buy_token)?;
    let buy_amount = Float::parse(buy_amount.to_string())?;
    let price_cap = Float::parse(price_cap.to_string())?;

    let zero = Float::zero()?;
    if buy_amount.lte(zero)? {
        return Err(RaindexError::NonPositiveBuyAmount);
    }
    if price_cap.lt(zero)? {
        return Err(RaindexError::NegativePriceCap);
    }

    Ok(ParsedTakeOrdersRequest {
        sell_token,
        buy_token,
        buy_amount,
        price_cap,
        min_receive_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_parse_request_valid_inputs() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "2.5",
            MinReceiveMode::Partial,
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
        assert!(req
            .buy_amount
            .eq(Float::parse("100".to_string()).unwrap())
            .unwrap());
        assert!(req
            .price_cap
            .eq(Float::parse("2.5".to_string()).unwrap())
            .unwrap());
        assert!(matches!(req.min_receive_mode, MinReceiveMode::Partial));
    }

    #[test]
    fn test_parse_request_invalid_sell_token() {
        let result = parse_request(
            "not-an-address",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "2",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_token() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "invalid",
            "100",
            "2",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_buy_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "not-a-number",
            "2",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_invalid_price_cap() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "not-a-number",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::Float(_))));
    }

    #[test]
    fn test_parse_request_exact_mode() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "50",
            "1.5",
            MinReceiveMode::Exact,
        );

        assert!(result.is_ok());
        let req = result.unwrap();
        assert!(matches!(req.min_receive_mode, MinReceiveMode::Exact));
    }

    #[test]
    fn test_parse_request_zero_buy_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "0",
            "2",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::NonPositiveBuyAmount)));
    }

    #[test]
    fn test_parse_request_negative_buy_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "-10",
            "2",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::NonPositiveBuyAmount)));
    }

    #[test]
    fn test_parse_request_zero_price_cap_is_valid() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
            "0",
            MinReceiveMode::Partial,
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
            "100",
            "-1",
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::NegativePriceCap)));
    }
}
