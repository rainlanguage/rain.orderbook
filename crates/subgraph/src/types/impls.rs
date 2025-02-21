use super::common::*;
use crate::performance::PerformanceError;
use alloy::primitives::U256;
use rain_orderbook_math::BigUintMath;
use std::str::FromStr;

impl SgErc20 {
    pub fn get_decimals(&self) -> Result<u8, PerformanceError> {
        Ok(self
            .decimals
            .as_ref()
            .map(|v| v.0.as_str())
            .unwrap_or("18")
            .parse()?)
    }
}

impl SgTrade {
    /// Scales this trade's io to 18 point decimals in U256
    pub fn scale_18_io(&self) -> Result<(U256, U256), PerformanceError> {
        let input_amount = if self.input_vault_balance_change.amount.0.starts_with('-') {
            &self.input_vault_balance_change.amount.0[1..]
        } else {
            &self.input_vault_balance_change.amount.0
        };
        let output_amount = if self.output_vault_balance_change.amount.0.starts_with('-') {
            &self.output_vault_balance_change.amount.0[1..]
        } else {
            &self.output_vault_balance_change.amount.0
        };
        Ok((
            U256::from_str(input_amount)?
                .scale_18(self.input_vault_balance_change.vault.token.get_decimals()?)?,
            U256::from_str(output_amount)?.scale_18(
                self.output_vault_balance_change
                    .vault
                    .token
                    .get_decimals()?,
            )?,
        ))
    }

    /// Calculates the trade's I/O ratio
    pub fn ratio(&self) -> Result<U256, PerformanceError> {
        let (input, output) = self.scale_18_io()?;
        if output.is_zero() {
            Err(PerformanceError::DivByZero)
        } else {
            Ok(input.div_18(output)?)
        }
    }

    /// Calculates the trade's O/I ratio (inverse)
    pub fn inverse_ratio(&self) -> Result<U256, PerformanceError> {
        let (input, output) = self.scale_18_io()?;
        if output.is_zero() {
            Err(PerformanceError::DivByZero)
        } else {
            Ok(output.div_18(input)?)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgOrderbook, SgTradeEvent, SgTradeStructPartialOrder,
        SgTradeVaultBalanceChange, SgTransaction, SgVaultBalanceChangeVault,
    };
    use alloy::primitives::Address;

    #[test]
    fn test_token_get_decimals() {
        // known decimals
        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };
        let result = token.get_decimals().unwrap();
        assert_eq!(result, 6);

        // unknown decimals, defaults to 18
        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: None,
        };
        let result = token.get_decimals().unwrap();
        assert_eq!(result, 18);
    }

    #[test]
    fn test_scale_18_io() {
        let (input, output) = get_trade().scale_18_io().unwrap();
        let expected_input = U256::from_str("3000000000000000000").unwrap();
        let expected_output = U256::from_str("6000000000000000000").unwrap();
        assert_eq!(input, expected_input);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_ratio_happy() {
        let result = get_trade().ratio().unwrap();
        let expected = U256::from_str("500000000000000000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_ratio_unhappy() {
        let mut trade = get_trade();
        trade.output_vault_balance_change.amount = SgBigInt("0".to_string());
        matches!(trade.ratio().unwrap_err(), PerformanceError::DivByZero);
    }

    #[test]
    fn test_inverse_ratio_happy() {
        let result = get_trade().inverse_ratio().unwrap();
        let expected = U256::from_str("2000000000000000000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_inverse_ratio_unhappy() {
        let mut trade = get_trade();
        trade.input_vault_balance_change.amount = SgBigInt("0".to_string());
        matches!(
            trade.inverse_ratio().unwrap_err(),
            PerformanceError::DivByZero
        );
    }

    // helper to get trade struct
    fn get_trade() -> SgTrade {
        let token_address = Address::from_slice(&[0x11u8; 20]);
        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };
        let input_trade_vault_balance_change = SgTradeVaultBalanceChange {
            id: SgBytes("".to_string()),
            __typename: "".to_string(),
            amount: SgBigInt("3000000".to_string()),
            new_vault_balance: SgBigInt("".to_string()),
            old_vault_balance: SgBigInt("".to_string()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("".to_string()),
                vault_id: SgBigInt("".to_string()),
                token: token.clone(),
            },
            timestamp: SgBigInt("".to_string()),
            transaction: SgTransaction {
                id: SgBytes("".to_string()),
                from: SgBytes("".to_string()),
                block_number: SgBigInt("".to_string()),
                timestamp: SgBigInt("".to_string()),
            },
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        };
        let output_trade_vault_balance_change = SgTradeVaultBalanceChange {
            id: SgBytes("".to_string()),
            __typename: "".to_string(),
            amount: SgBigInt("-6000000".to_string()),
            new_vault_balance: SgBigInt("".to_string()),
            old_vault_balance: SgBigInt("".to_string()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("".to_string()),
                vault_id: SgBigInt("".to_string()),
                token: token.clone(),
            },
            timestamp: SgBigInt("".to_string()),
            transaction: SgTransaction {
                id: SgBytes("".to_string()),
                from: SgBytes("".to_string()),
                block_number: SgBigInt("".to_string()),
                timestamp: SgBigInt("".to_string()),
            },
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        };
        SgTrade {
            id: SgBytes("".to_string()),
            trade_event: SgTradeEvent {
                transaction: SgTransaction {
                    id: SgBytes("".to_string()),
                    from: SgBytes("".to_string()),
                    block_number: SgBigInt("".to_string()),
                    timestamp: SgBigInt("".to_string()),
                },
                sender: SgBytes("".to_string()),
            },
            output_vault_balance_change: output_trade_vault_balance_change,
            input_vault_balance_change: input_trade_vault_balance_change,
            order: SgTradeStructPartialOrder {
                id: SgBytes("".to_string()),
                order_hash: SgBytes("".to_string()),
            },
            timestamp: SgBigInt("".to_string()),
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        }
    }
}
