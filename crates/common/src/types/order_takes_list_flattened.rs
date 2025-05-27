use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use alloy::primitives::{utils::format_units, I256};
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderTakeFlattened {
    pub id: String,
    pub timestamp: SgBigInt,
    pub timestamp_display: String,
    pub transaction: SgBytes,
    pub sender: SgBytes,
    pub order_id: SgBytes,
    pub input: SgBigInt,
    pub input_display: String,
    pub input_token_id: SgBytes,
    pub input_token_symbol: Option<String>,
    pub output: SgBigInt,
    pub output_display: String,
    pub output_token_id: SgBytes,
    pub output_token_symbol: Option<String>,
}

impl TryFrom<SgTrade> for OrderTakeFlattened {
    type Error = FlattenError;

    fn try_from(val: SgTrade) -> Result<Self, Self::Error> {
        let timestamp = val.timestamp.clone();
        let input_vault_balance_change = val.input_vault_balance_change.clone();
        let output_vault_balance_change = val.output_vault_balance_change.clone();
        let input_amount = input_vault_balance_change.amount.0.parse::<I256>()?;
        let output_amount = output_vault_balance_change.amount.0.parse::<I256>()?;
        let input_decimals = input_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
            .0
            .parse::<u8>()?;
        let output_decimals = output_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
            .0
            .parse::<u8>()?;

        Ok(Self {
            id: val.id.0,
            timestamp: timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(timestamp.0)?,
            transaction: val.trade_event.transaction.id,
            sender: val.trade_event.sender,
            order_id: val.order.order_hash,
            input: input_vault_balance_change.amount,
            input_display: format_units(input_amount, input_decimals)?,
            input_token_id: input_vault_balance_change.vault.token.id,
            input_token_symbol: input_vault_balance_change.vault.token.symbol,
            output: output_vault_balance_change.amount,
            output_display: format_units(output_amount, output_decimals)?,
            output_token_id: output_vault_balance_change.vault.token.address,
            output_token_symbol: output_vault_balance_change.vault.token.symbol,
        })
    }
}

impl TryIntoCsv<OrderTakeFlattened> for Vec<OrderTakeFlattened> {}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderbook, SgTrade, SgTradeEvent, SgTradeStructPartialOrder,
        SgTradeVaultBalanceChange, SgTransaction, SgVaultBalanceChangeVault,
    };

    // Helper to build a default, valid SgTrade instance
    fn mock_sg_trade_default() -> SgTrade {
        SgTrade {
            id: SgBytes("trade001".to_string()),
            timestamp: SgBigInt("1678886400".to_string()),
            trade_event: SgTradeEvent {
                transaction: SgTransaction {
                    id: SgBytes("tx001".to_string()),
                    from: SgBytes("0xfromAddress".to_string()),
                    block_number: SgBigInt("1000".to_string()),
                    timestamp: SgBigInt("1678886300".to_string()),
                },
                sender: SgBytes("0xsenderAddress".to_string()),
            },
            order: SgTradeStructPartialOrder {
                id: SgBytes("orderPartial001".to_string()),
                order_hash: SgBytes("orderHash001".to_string()),
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: SgBytes("inputVBC001".to_string()),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("1000000000000000000".to_string()),
                new_vault_balance: SgBigInt("5000000000000000000".to_string()),
                old_vault_balance: SgBigInt("6000000000000000000".to_string()),
                vault: SgVaultBalanceChangeVault {
                    id: SgBytes("inputVault001".to_string()),
                    vault_id: SgBigInt("101".to_string()),
                    token: SgErc20 {
                        id: SgBytes("inputTokenId001".to_string()),
                        address: SgBytes("0xinputTokenAddress".to_string()),
                        name: Some("Input Token".to_string()),
                        symbol: Some("INPUT_TKN".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                },
                timestamp: SgBigInt("1678886400".to_string()),
                transaction: SgTransaction {
                    id: SgBytes("txVBCInput001".to_string()),
                    from: SgBytes("0xfromAddressVBCIn".to_string()),
                    block_number: SgBigInt("1000".to_string()),
                    timestamp: SgBigInt("1678886400".to_string()),
                },
                orderbook: SgOrderbook {
                    id: SgBytes("orderbookVBCIn001".to_string()),
                },
            },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: SgBytes("outputVBC001".to_string()),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("200000000".to_string()),
                new_vault_balance: SgBigInt("300000000".to_string()),
                old_vault_balance: SgBigInt("100000000".to_string()),
                vault: SgVaultBalanceChangeVault {
                    id: SgBytes("outputVault001".to_string()),
                    vault_id: SgBigInt("202".to_string()),
                    token: SgErc20 {
                        id: SgBytes("outputTokenId001".to_string()),
                        address: SgBytes("0xoutputTokenAddress".to_string()),
                        name: Some("Output Token".to_string()),
                        symbol: Some("OUTPUT_TKN".to_string()),
                        decimals: Some(SgBigInt("8".to_string())),
                    },
                },
                timestamp: SgBigInt("1678886400".to_string()),
                transaction: SgTransaction {
                    id: SgBytes("txVBCOutput001".to_string()),
                    from: SgBytes("0xfromAddressVBCOut".to_string()),
                    block_number: SgBigInt("1000".to_string()),
                    timestamp: SgBigInt("1678886400".to_string()),
                },
                orderbook: SgOrderbook {
                    id: SgBytes("orderbookVBCOut001".to_string()),
                },
            },
            orderbook: SgOrderbook {
                id: SgBytes("mainOrderbook001".to_string()),
            },
        }
    }

    #[test]
    fn test_valid_sgtrade_all_fields() {
        let trade_data = mock_sg_trade_default();
        let result = OrderTakeFlattened::try_from(trade_data.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.id, trade_data.id.0);
        assert_eq!(flattened.timestamp, trade_data.timestamp);
        assert_eq!(
            flattened.timestamp_display,
            format_bigint_timestamp_display(trade_data.timestamp.clone().0).unwrap()
        );

        assert_eq!(flattened.transaction, trade_data.trade_event.transaction.id);
        assert_eq!(flattened.sender, trade_data.trade_event.sender);
        assert_eq!(flattened.order_id, trade_data.order.order_hash);

        assert_eq!(
            flattened.input,
            trade_data.input_vault_balance_change.amount
        );
        assert_eq!(flattened.input_display, "1.000000000000000000");
        assert_eq!(
            flattened.input_token_id,
            trade_data.input_vault_balance_change.vault.token.id
        );
        assert_eq!(
            flattened.input_token_symbol,
            trade_data.input_vault_balance_change.vault.token.symbol
        );

        assert_eq!(
            flattened.output,
            trade_data.output_vault_balance_change.amount
        );
        assert_eq!(flattened.output_display, "2.00000000");
        assert_eq!(
            flattened.output_token_id,
            trade_data.output_vault_balance_change.vault.token.address
        );
        assert_eq!(
            flattened.output_token_symbol,
            trade_data.output_vault_balance_change.vault.token.symbol
        );
    }

    #[test]
    fn test_optional_decimals_symbol_none() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.vault.token.decimals = None;
        trade_data.input_vault_balance_change.vault.token.symbol = None;
        trade_data.output_vault_balance_change.vault.token.decimals = None;
        trade_data.output_vault_balance_change.vault.token.symbol = None;

        let result = OrderTakeFlattened::try_from(trade_data.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.input_display, "1000000000000000000.0");
        assert_eq!(flattened.input_token_symbol, None);
        assert_eq!(flattened.output_display, "200000000.0");
        assert_eq!(flattened.output_token_symbol, None);
    }

    #[test]
    fn test_zero_amounts() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.amount = SgBigInt("0".to_string());
        trade_data.output_vault_balance_change.amount = SgBigInt("0".to_string());

        let result = OrderTakeFlattened::try_from(trade_data.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.input_display, "0.000000000000000000");
        assert_eq!(flattened.output_display, "0.00000000");
    }

    #[test]
    fn test_empty_strings_for_ids_symbols() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.id = SgBytes("".to_string());
        trade_data.trade_event.transaction.id = SgBytes("".to_string());
        trade_data.trade_event.sender = SgBytes("".to_string());
        trade_data.order.order_hash = SgBytes("".to_string());
        trade_data.input_vault_balance_change.vault.token.id = SgBytes("".to_string());
        trade_data.input_vault_balance_change.vault.token.symbol = Some("".to_string());
        trade_data.output_vault_balance_change.vault.token.address = SgBytes("".to_string());
        trade_data.output_vault_balance_change.vault.token.symbol = Some("".to_string());

        let result = OrderTakeFlattened::try_from(trade_data.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.id, "");
        assert_eq!(flattened.transaction, SgBytes("".to_string()));
        assert_eq!(flattened.sender, SgBytes("".to_string()));
        assert_eq!(flattened.order_id, SgBytes("".to_string()));
        assert_eq!(flattened.input_token_id, SgBytes("".to_string()));
        assert_eq!(flattened.input_token_symbol, Some("".to_string()));
        assert_eq!(flattened.output_token_id, SgBytes("".to_string()));
        assert_eq!(flattened.output_token_symbol, Some("".to_string()));
    }

    #[test]
    fn test_unparseable_input_amount() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.amount = SgBigInt("not_a_number".to_string());
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseSignedError(_))),
            "Expected ParseSignedError for unparseable input amount, got {:?}",
            result
        );
    }

    #[test]
    fn test_unparseable_output_amount() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.output_vault_balance_change.amount = SgBigInt("not_a_number".to_string());
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseSignedError(_))),
            "Expected ParseSignedError for unparseable output amount, got {:?}",
            result
        );
    }

    #[test]
    fn test_unparseable_input_decimals() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("not_a_u8".to_string()));
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for unparseable input decimals, got {:?}",
            result
        );
    }

    #[test]
    fn test_input_decimals_out_of_range_positive() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("256".to_string())); // u8 max is 255
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for input decimals out of range (256), got {:?}",
            result
        );
    }

    #[test]
    fn test_input_decimals_out_of_range_negative() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("-1".to_string()));
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for input decimals out of range (-1), got {:?}",
            result
        );
    }

    #[test]
    fn test_unparseable_output_decimals() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.output_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("not_a_u8".to_string()));
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for unparseable output decimals, got {:?}",
            result
        );
    }

    #[test]
    fn test_output_decimals_out_of_range_positive() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.output_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("256".to_string()));
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for output decimals out of range (256), got {:?}",
            result
        );
    }

    #[test]
    fn test_output_decimals_out_of_range_negative() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.output_vault_balance_change.vault.token.decimals =
            Some(SgBigInt("-1".to_string()));
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::ParseIntError(_))),
            "Expected ParseIntError for output decimals out of range (-1), got {:?}",
            result
        );
    }

    #[test]
    fn test_invalid_timestamp_for_display() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.timestamp = SgBigInt("not_a_timestamp".to_string());
        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(
            matches!(result, Err(FlattenError::FormatTimestampDisplayError(_))),
            "Expected FormatTimestampDisplayError for invalid timestamp for display format, got {:?}",
            result
        );
    }

    #[test]
    fn test_amount_i256_boundaries() {
        let mut trade_data_max_input = mock_sg_trade_default();
        trade_data_max_input.input_vault_balance_change.amount = SgBigInt(I256::MAX.to_string());
        let mut trade_data_min_output = mock_sg_trade_default();
        trade_data_min_output.output_vault_balance_change.amount = SgBigInt(I256::MIN.to_string());

        let result_max = OrderTakeFlattened::try_from(trade_data_max_input.clone());
        assert!(result_max.is_ok());
        let flattened_max = result_max.unwrap();
        let input_decimals = trade_data_max_input
            .input_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
            .0
            .parse::<u8>()
            .unwrap();
        assert_eq!(
            flattened_max.input_display,
            format_units(I256::MAX, input_decimals).unwrap()
        );

        let result_min = OrderTakeFlattened::try_from(trade_data_min_output.clone());
        assert!(result_min.is_ok());
        let flattened_min = result_min.unwrap();
        let output_decimals = trade_data_min_output
            .output_vault_balance_change
            .vault
            .token
            .decimals
            .clone()
            .unwrap_or(SgBigInt("0".into()))
            .0
            .parse::<u8>()
            .unwrap();
        assert_eq!(
            flattened_min.output_display,
            format_units(I256::MIN, output_decimals).unwrap()
        );
    }

    #[test]
    fn test_negative_amounts_formatting() {
        let mut trade_data = mock_sg_trade_default();
        trade_data.input_vault_balance_change.amount = SgBigInt("-1234567890123456789".to_string());
        trade_data.output_vault_balance_change.amount = SgBigInt("-98765432".to_string());

        let result = OrderTakeFlattened::try_from(trade_data);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.input_display, "-1.234567890123456789");
        assert_eq!(flattened.output_display, "-0.98765432");
    }
}
