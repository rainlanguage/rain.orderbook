use super::common::*;
use crate::{
    error::ParseNumberError,
    utils::{one_18, to_18_decimals},
};
use alloy::primitives::{utils::ParseUnits, I256, U256};
use std::str::FromStr;

impl Trade {
    /// Converts this trade's input to 18 point decimals in U256/I256
    pub fn input_as_18_decimals(&self) -> Result<ParseUnits, ParseNumberError> {
        Ok(to_18_decimals(
            ParseUnits::U256(U256::from_str(&self.input_vault_balance_change.amount.0)?),
            self.input_vault_balance_change
                .vault
                .token
                .decimals
                .as_ref()
                .map(|v| v.0.as_str())
                .unwrap_or("18"),
        )?)
    }

    /// Converts this trade's output to 18 point decimals in U256/I256
    pub fn output_as_18_decimals(&self) -> Result<ParseUnits, ParseNumberError> {
        Ok(to_18_decimals(
            ParseUnits::I256(I256::from_str(&self.output_vault_balance_change.amount.0)?),
            self.output_vault_balance_change
                .vault
                .token
                .decimals
                .as_ref()
                .map(|v| v.0.as_str())
                .unwrap_or("18"),
        )?)
    }

    /// Calculates the trade's I/O ratio
    pub fn ratio(&self) -> Result<U256, ParseNumberError> {
        Ok(self
            .input_as_18_decimals()?
            .get_absolute()
            .saturating_mul(one_18().get_absolute())
            .checked_div(
                self.output_as_18_decimals()?
                    .get_signed()
                    .saturating_neg()
                    .try_into()?,
            )
            .unwrap_or(U256::MAX))
    }

    /// Calculates the trade's O/I ratio (inverse)
    pub fn inverse_ratio(&self) -> Result<U256, ParseNumberError> {
        Ok(
            TryInto::<U256>::try_into(self.output_as_18_decimals()?.get_signed().saturating_neg())?
                .saturating_mul(one_18().get_absolute())
                .checked_div(self.input_as_18_decimals()?.get_absolute())
                .unwrap_or(U256::MAX),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
        Transaction, VaultBalanceChangeVault,
    };
    use alloy::primitives::Address;

    #[test]
    fn test_input_to_18_decimals() {
        let result = get_trade().input_as_18_decimals().unwrap();
        let expected = U256::from_str("3000000000000000000").unwrap();
        assert_eq!(result.get_absolute(), expected);
    }

    #[test]
    fn test_output_to_18_decimals() {
        let result = get_trade().output_as_18_decimals().unwrap();
        let expected = I256::from_str("-6000000000000000000").unwrap();
        assert_eq!(result.get_signed(), expected);
    }

    #[test]
    fn test_ratio() {
        let result = get_trade().ratio().unwrap();
        let expected = U256::from_str("500000000000000000").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_inverse_ratio() {
        let result = get_trade().inverse_ratio().unwrap();
        let expected = U256::from_str("2000000000000000000").unwrap();
        assert_eq!(result, expected);
    }

    // helper to get trade struct
    fn get_trade() -> Trade {
        let token_address = Address::from_slice(&[0x11u8; 20]);
        let token = Erc20 {
            id: Bytes(token_address.to_string()),
            address: Bytes(token_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(BigInt(6.to_string())),
        };
        let input_trade_vault_balance_change = TradeVaultBalanceChange {
            id: Bytes("".to_string()),
            __typename: "".to_string(),
            amount: BigInt("3000000".to_string()),
            new_vault_balance: BigInt("".to_string()),
            old_vault_balance: BigInt("".to_string()),
            vault: VaultBalanceChangeVault {
                id: Bytes("".to_string()),
                vault_id: BigInt("".to_string()),
                token: token.clone(),
            },
            timestamp: BigInt("".to_string()),
            transaction: Transaction {
                id: Bytes("".to_string()),
                from: Bytes("".to_string()),
                block_number: BigInt("".to_string()),
                timestamp: BigInt("".to_string()),
            },
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
        };
        let output_trade_vault_balance_change = TradeVaultBalanceChange {
            id: Bytes("".to_string()),
            __typename: "".to_string(),
            amount: BigInt("-6000000".to_string()),
            new_vault_balance: BigInt("".to_string()),
            old_vault_balance: BigInt("".to_string()),
            vault: VaultBalanceChangeVault {
                id: Bytes("".to_string()),
                vault_id: BigInt("".to_string()),
                token: token.clone(),
            },
            timestamp: BigInt("".to_string()),
            transaction: Transaction {
                id: Bytes("".to_string()),
                from: Bytes("".to_string()),
                block_number: BigInt("".to_string()),
                timestamp: BigInt("".to_string()),
            },
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
        };
        Trade {
            id: Bytes("".to_string()),
            trade_event: TradeEvent {
                transaction: Transaction {
                    id: Bytes("".to_string()),
                    from: Bytes("".to_string()),
                    block_number: BigInt("".to_string()),
                    timestamp: BigInt("".to_string()),
                },
                sender: Bytes("".to_string()),
            },
            output_vault_balance_change: output_trade_vault_balance_change,
            input_vault_balance_change: input_trade_vault_balance_change,
            order: TradeStructPartialOrder {
                id: Bytes("".to_string()),
                order_hash: Bytes("".to_string()),
            },
            timestamp: BigInt("".to_string()),
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
        }
    }
}
