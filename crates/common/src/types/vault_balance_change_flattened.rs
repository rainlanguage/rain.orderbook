use crate::{csv::TryIntoCsv, utils::timestamp::format_bigint_timestamp_display};
use alloy::primitives::{utils::format_units, I256};
use rain_orderbook_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VaultBalanceChangeFlattened {
    pub timestamp: SgBigInt,
    pub timestamp_display: String,
    pub from: SgBytes,
    pub amount: SgBigInt,
    pub amount_display_signed: String,
    pub change_type_display: String,
    pub balance: SgBigInt,
}

impl TryFrom<SgVaultBalanceChangeUnwrapped> for VaultBalanceChangeFlattened {
    type Error = FlattenError;

    fn try_from(val: SgVaultBalanceChangeUnwrapped) -> Result<Self, Self::Error> {
        let amount_display_signed = format_units(
            val.amount.0.parse::<I256>()?,
            val.vault
                .token
                .decimals
                .unwrap_or(SgBigInt("0".into()))
                .0
                .parse::<u8>()?,
        )?;

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
    use alloy::primitives::ParseSignedError;
    use rain_orderbook_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderbook, SgTransaction, SgVaultBalanceChangeUnwrapped,
        SgVaultBalanceChangeVault,
    };

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
            amount: SgBigInt(amount_val.to_string()),
            __typename: typename_val.to_string(),
            new_vault_balance: SgBigInt(new_balance_val.to_string()),
            old_vault_balance: SgBigInt("0".into()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("0xvaultid".to_string()),
                vault_id: SgBigInt("1".to_string()),
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
            "1000000000000000000",
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
        assert_eq!(flattened.amount_display_signed, "1.000000000000000000");
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
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.amount_display_signed, "500.0");
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
        assert_eq!(flattened.amount_display_signed, "12345.0");
    }

    #[test]
    fn valid_input_negative_amount_string() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0xghi789",
            "-500000000000000000",
            "Trade",
            "500000000000000000",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.amount_display_signed, "-0.500000000000000000");
    }

    #[test]
    fn amount_parsing_fails_invalid_number() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "not_a_number",
            "Deposit",
            "100",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            FlattenError::ParseSignedError(_)
        ));
    }

    #[test]
    fn amount_too_large_for_i256() {
        let too_large_val =
            "115792089237316195423570985008687907853269984665640564039457584007913129639936";
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            too_large_val,
            "Deposit",
            "100",
            Some("18"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(
            result.is_err(),
            "Expected error for too large amount, got {:?}",
            result
        );
        let err = result.err().unwrap();
        assert!(matches!(
            err,
            FlattenError::ParseSignedError(ParseSignedError::IntegerOverflow)
        ));
    }

    #[test]
    fn decimals_parsing_fails_invalid_u8() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "1000",
            "Deposit",
            "1100",
            Some("not_a_u8"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            FlattenError::ParseIntError(_)
        ));
    }

    #[test]
    fn decimals_value_too_large_for_u8() {
        let val = mock_sg_vault_balance_change_unwrapped(
            "1678886400",
            "0x123",
            "1000",
            "Deposit",
            "1100",
            Some("256"),
        );
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            FlattenError::ParseIntError(_)
        ));
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
        let result = VaultBalanceChangeFlattened::try_from(val);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            FlattenError::FormatTimestampDisplayError(_)
        ));
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
