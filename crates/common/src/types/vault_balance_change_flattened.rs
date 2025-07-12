use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use rain_math_float::Float;
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultBalanceChangeFlattened {
    pub timestamp: SgBigInt,
    pub timestamp_display: String,
    pub from: SgBytes,
    pub amount: SgBytes,
    pub amount_display_signed: String,
    pub change_type_display: String,
    pub balance: SgBytes,
}

impl TryFrom<SgVaultBalanceChangeUnwrapped> for VaultBalanceChangeFlattened {
    type Error = FlattenError;

    fn try_from(val: SgVaultBalanceChangeUnwrapped) -> Result<Self, Self::Error> {
        let amount = Float::from_hex(&val.amount.0)?;
        let amount_display_signed = amount.format18()?;

        Ok(Self {
            timestamp: val.timestamp.clone(),
            timestamp_display: format_bigint_timestamp_display(val.timestamp.0)?,
            from: val.transaction.from,
            amount: val.amount,
            amount_display_signed,
            change_type_display: val.__typename,
            balance: val.new_vault_balance.clone(),
        })
    }
}

impl TryIntoCsv<VaultBalanceChangeFlattened> for Vec<VaultBalanceChangeFlattened> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::timestamp::format_bigint_timestamp_display;
    use rain_orderbook_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderbook, SgTransaction, SgVaultBalanceChangeUnwrapped,
        SgVaultBalanceChangeVault,
    };
    use rain_orderbook_subgraph_client::utils::float::*;

    fn mock_sg_vault_balance_change_unwrapped(
        timestamp_val: &str,
        from_val: &str,
        amount_val: &str,
        typename_val: &str,
        new_balance_val: &str,
        decimals_val: Option<&str>,
    ) -> SgVaultBalanceChangeUnwrapped {
        SgVaultBalanceChangeUnwrapped {
            timestamp: SgBigInt(timestamp_val.to_string()),
            transaction: SgTransaction {
                id: SgBytes("0xtxid".to_string()),
                from: SgBytes(from_val.to_string()),
                block_number: SgBigInt("100".to_string()),
                timestamp: SgBigInt(timestamp_val.to_string()),
            },
            amount: SgBytes(Float::parse(amount_val.to_string()).unwrap().as_hex()),
            __typename: typename_val.to_string(),
            new_vault_balance: SgBytes(Float::parse(new_balance_val.to_string()).unwrap().as_hex()),
            old_vault_balance: SgBytes((*F0).as_hex()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("0xvaultid".to_string()),
                vault_id: SgBytes("1".to_string()),
                token: SgErc20 {
                    id: SgBytes("0xtokenid".to_string()),
                    address: SgBytes("0xtokenaddress".to_string()),
                    name: Some("Test Token".to_string()),
                    symbol: Some("TT".to_string()),
                    decimals: decimals_val.map(|s| SgBigInt(s.to_string())),
                },
            },
            orderbook: SgOrderbook {
                id: SgBytes("0xorderbookid".to_string()),
            },
        }
    }

    #[test]
    fn valid_full_input() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123abc",
            "1",
            "Deposit",
            "2000000000000000000",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.timestamp, val.timestamp);
        assert_eq!(
            flattened.timestamp_display,
            format_bigint_timestamp_display(val.timestamp.0).unwrap()
        );
        assert_eq!(flattened.from, val.transaction.from);
        assert_eq!(flattened.amount, val.amount);
        assert_eq!(flattened.amount_display_signed, "1");
        assert_eq!(flattened.change_type_display, val.__typename);
        assert_eq!(flattened.balance, val.new_vault_balance);
    }

    #[test]
    fn valid_input_zero_decimals() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0xabc123",
            "500",
            "Withdrawal",
            "100",
            Some("0"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        let flattened = result.unwrap();
        assert_eq!(flattened.amount_display_signed, "500");
        assert_eq!(flattened.change_type_display, "Withdrawal");
    }

    #[test]
    fn valid_input_none_decimals() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0xdef456",
            "12345",
            "TradeVaultBalanceChange",
            "54321",
            None,
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.amount_display_signed, "12345");
    }

    #[test]
    fn valid_input_negative_amount_string() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0xghi789",
            "-0.500000000000000000",
            "Trade",
            "500000000000000000",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.amount_display_signed, "-0.5");
    }

    #[test]
    fn amount_parsing_fails_invalid_number() {
        let mut val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "0",
            "Deposit",
            "100",
            Some("18"),
        );

        let invalid_amount = "not_a_number";
        val.amount = SgBytes(invalid_amount.to_string());

        let err = VaultBalanceChangeFlattened::try_from(val).unwrap_err();
        assert!(
            matches!(err, FlattenError::ParseError(_)),
            "Unexpected error: {err:?}"
        );
    }

    #[test]
    fn amount_invalid_amount() {
        let mut mock = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "1",
            "Deposit",
            "100",
            Some("18"),
        );

        let invalid_amount =
            "115792089237316195423570985008687907853269984665640564039457584007913129639936";
        mock.amount = SgBytes(invalid_amount.to_string());

        let err = VaultBalanceChangeFlattened::try_from(mock).unwrap_err();

        assert!(
            matches!(err, FlattenError::ParseError(_)),
            "Unexpected error: {err:?}",
        );
    }

    #[test]
    fn timestamp_formatting_fails() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "not_a_timestamp",
            "0x123",
            "1000",
            "Deposit",
            "1100",
            Some("18"),
        );
        let err = VaultBalanceChangeFlattened::try_from(val).unwrap_err();
        assert!(matches!(err, FlattenError::FormatTimestampDisplayError(_)));
    }

    #[test]
    fn empty_string_for_from_address() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "",
            "1000",
            "Deposit",
            "1100",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.from, val.transaction.from);
        assert_eq!(flattened.from.0, "");
    }

    #[test]
    fn empty_string_for_typename() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "1000",
            "",
            "1100",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val.clone());
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.change_type_display, val.__typename);
        assert_eq!(flattened.change_type_display, "");
    }
}
