use crate::raindex_client::RaindexError;
use crate::take_orders::MinReceiveMode;
use alloy::primitives::Address;
use rain_math_float::Float;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct ParsedTakeOrdersRequest {
    pub sell_token: Address,
    pub buy_token: Address,
    pub sell_amount: Float,
    pub min_receive_mode: MinReceiveMode,
}

pub(crate) fn parse_request(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    min_receive_mode: MinReceiveMode,
) -> Result<ParsedTakeOrdersRequest, RaindexError> {
    let sell_token = Address::from_str(sell_token)?;
    let buy_token = Address::from_str(buy_token)?;
    let sell_amount = Float::parse(sell_amount.to_string())?;
    Ok(ParsedTakeOrdersRequest {
        sell_token,
        buy_token,
        sell_amount,
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
            .sell_amount
            .eq(Float::parse("100".to_string()).unwrap())
            .unwrap());
        assert!(matches!(req.min_receive_mode, MinReceiveMode::Partial));
    }

    #[test]
    fn test_parse_request_invalid_sell_token() {
        let result = parse_request(
            "not-an-address",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "100",
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
            MinReceiveMode::Partial,
        );

        assert!(matches!(result, Err(RaindexError::FromHexError(_))));
    }

    #[test]
    fn test_parse_request_invalid_sell_amount() {
        let result = parse_request(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
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
            MinReceiveMode::Exact,
        );

        assert!(result.is_ok());
        let req = result.unwrap();
        assert!(matches!(req.min_receive_mode, MinReceiveMode::Exact));
    }
}
