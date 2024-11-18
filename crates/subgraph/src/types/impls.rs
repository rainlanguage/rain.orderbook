use super::common::*;
use crate::performance::PerformanceError;
use alloy::primitives::U256;
use rain_orderbook_math::BigUintMath;
use std::str::FromStr;

impl Trade {
    /// Converts this trade's io to 18 point decimals in U256
    pub fn scale_18_io(&self) -> Result<(U256, U256), PerformanceError> {
        let amount = if self.output_vault_balance_change.amount.0.starts_with('-') {
            &self.output_vault_balance_change.amount.0[1..]
        } else {
            &self.output_vault_balance_change.amount.0
        };
        Ok((
            U256::from_str(&self.input_vault_balance_change.amount.0)?.scale_18(
                self.input_vault_balance_change
                    .vault
                    .token
                    .decimals
                    .as_ref()
                    .map(|v| v.0.as_str())
                    .unwrap_or("18")
                    .parse()?,
            )?,
            U256::from_str(amount)?.scale_18(
                self.output_vault_balance_change
                    .vault
                    .token
                    .decimals
                    .as_ref()
                    .map(|v| v.0.as_str())
                    .unwrap_or("18")
                    .parse()?,
            )?,
        ))
    }

    /// Calculates the trade's I/O ratio
    pub fn ratio(&self) -> Result<U256, PerformanceError> {
        let (input, output) = self.scale_18_io()?;
        if output.is_zero() && input.is_zero() {
            Ok(U256::ZERO)
        } else if output.is_zero() {
            Ok(U256::MAX)
        } else {
            Ok(input.div_18(output)?)
        }
    }

    /// Calculates the trade's O/I ratio (inverse)
    pub fn inverse_ratio(&self) -> Result<U256, PerformanceError> {
        let (input, output) = self.scale_18_io()?;
        if output.is_zero() && input.is_zero() {
            Ok(U256::ZERO)
        } else if input.is_zero() {
            Ok(U256::MAX)
        } else {
            Ok(output.div_18(input)?)
        }
    }
}

#[cfg(target_family = "wasm")]
mod js_api {
    use super::super::common::{
        AddOrder, BigInt, Bytes, ClearBounty, Deposit, Erc20, Order, OrderAsIO,
        OrderStructPartialTrade, TradeVaultBalanceChange, Transaction, Vault, VaultBalanceChange,
        VaultBalanceChangeVault, Withdrawal,
    };
    use rain_orderbook_bindings::impl_wasm_traits;
    use serde_wasm_bindgen::{from_value, to_value};
    use wasm_bindgen::{convert::*, describe::WasmDescribe};
    use wasm_bindgen::{
        describe::{inform, WasmDescribeVector, VECTOR},
        JsValue, UnwrapThrowExt,
    };

    impl_wasm_traits!(Order);
    impl_wasm_traits!(Vault);
    impl_wasm_traits!(AddOrder);
    impl_wasm_traits!(OrderAsIO);
    impl_wasm_traits!(VaultBalanceChangeVault);
    impl_wasm_traits!(VaultBalanceChange);
    impl_wasm_traits!(Withdrawal);
    impl_wasm_traits!(TradeVaultBalanceChange);
    impl_wasm_traits!(Deposit);
    impl_wasm_traits!(ClearBounty);
    impl_wasm_traits!(OrderStructPartialTrade);
    impl_wasm_traits!(Erc20);
    impl_wasm_traits!(Transaction);
    impl_wasm_traits!(BigInt);
    impl_wasm_traits!(Bytes);
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
    fn test_scale_18_io() {
        let (input, output) = get_trade().scale_18_io().unwrap();
        let expected_input = U256::from_str("3000000000000000000").unwrap();
        let expected_output = U256::from_str("6000000000000000000").unwrap();
        assert_eq!(input, expected_input);
        assert_eq!(output, expected_output);
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
