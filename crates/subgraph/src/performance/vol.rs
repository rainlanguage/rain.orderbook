use crate::{
    performance::PerformanceError,
    types::common::{SgErc20, SgTrade},
};
use alloy::primitives::U256;
use rain_orderbook_math::{BigUintMath, MathError};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VolumeDetails {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_in: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_out: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_vol: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub net_vol: U256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VaultVolume {
    pub id: String,
    pub token: SgErc20,
    pub vol_details: VolumeDetails,
}

#[cfg(target_family = "wasm")]
mod impls {
    use super::*;
    impl_wasm_traits!(VolumeDetails);
    impl_wasm_traits!(VaultVolume);
}

/// Get the vaults volume from array of trades of an order
pub fn get_vaults_vol(trades: &[SgTrade]) -> Result<Vec<VaultVolume>, PerformanceError> {
    let mut vaults_vol: Vec<VaultVolume> = vec![];
    for trade in trades {
        if let Some(vault_vol) = vaults_vol.iter_mut().find(|v| {
            v.id == trade.input_vault_balance_change.vault.vault_id.0
                && v.token.address.0 == trade.input_vault_balance_change.vault.token.address.0
        }) {
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;

                vault_vol.vol_details.total_out = vault_vol
                    .vol_details
                    .total_out
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;

                vault_vol.vol_details.total_vol = vault_vol
                    .vol_details
                    .total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;

                vault_vol.vol_details.total_in = vault_vol
                    .vol_details
                    .total_in
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;

                vault_vol.vol_details.total_vol = vault_vol
                    .vol_details
                    .total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            }

            vault_vol.vol_details.net_vol =
                if vault_vol.vol_details.total_in >= vault_vol.vol_details.total_out {
                    vault_vol
                        .vol_details
                        .total_in
                        .checked_sub(vault_vol.vol_details.total_out)
                        .ok_or(PerformanceError::MathError(MathError::Overflow))?
                } else {
                    vault_vol
                        .vol_details
                        .total_out
                        .checked_sub(vault_vol.vol_details.total_in)
                        .ok_or(PerformanceError::MathError(MathError::Overflow))?
                };
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;
                total_out = total_out
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
                total_vol = total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;
                total_in = total_in
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
                total_vol = total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            }
            vaults_vol.push(VaultVolume {
                id: trade.input_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.input_vault_balance_change.vault.token.clone(),
                vol_details: VolumeDetails {
                    total_in,
                    total_out,
                    total_vol,
                    net_vol: if total_in >= total_out {
                        total_in
                            .checked_sub(total_out)
                            .ok_or(PerformanceError::MathError(MathError::Overflow))?
                    } else {
                        total_out
                            .checked_sub(total_in)
                            .ok_or(PerformanceError::MathError(MathError::Overflow))?
                    },
                },
            })
        }
        if let Some(vault_vol) = vaults_vol.iter_mut().find(|v| {
            v.id == trade.output_vault_balance_change.vault.vault_id.0
                && v.token.address.0 == trade.output_vault_balance_change.vault.token.address.0
        }) {
            if trade.output_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0[1..])?;

                vault_vol.vol_details.total_out = vault_vol
                    .vol_details
                    .total_out
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;

                vault_vol.vol_details.total_vol = vault_vol
                    .vol_details
                    .total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                vault_vol.vol_details.total_in = vault_vol
                    .vol_details
                    .total_in
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
                vault_vol.vol_details.total_vol = vault_vol
                    .vol_details
                    .total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            }
            vault_vol.vol_details.net_vol =
                if vault_vol.vol_details.total_in >= vault_vol.vol_details.total_out {
                    vault_vol
                        .vol_details
                        .total_in
                        .checked_sub(vault_vol.vol_details.total_out)
                        .ok_or(PerformanceError::MathError(MathError::Overflow))?
                } else {
                    vault_vol
                        .vol_details
                        .total_out
                        .checked_sub(vault_vol.vol_details.total_in)
                        .ok_or(PerformanceError::MathError(MathError::Overflow))?
                };
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            if trade.output_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0[1..])?;
                total_out = total_out
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
                total_vol = total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                total_in = total_in
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
                total_vol = total_vol
                    .checked_add(amount)
                    .ok_or(PerformanceError::MathError(MathError::Overflow))?;
            }
            vaults_vol.push(VaultVolume {
                id: trade.output_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.output_vault_balance_change.vault.token.clone(),
                vol_details: VolumeDetails {
                    total_in,
                    total_out,
                    total_vol,
                    net_vol: if total_in >= total_out {
                        total_in
                            .checked_sub(total_out)
                            .ok_or(PerformanceError::MathError(MathError::Overflow))?
                    } else {
                        total_out
                            .checked_sub(total_in)
                            .ok_or(PerformanceError::MathError(MathError::Overflow))?
                    },
                },
            })
        }
    }
    Ok(vaults_vol)
}

impl VaultVolume {
    pub fn is_net_vol_negative(&self) -> bool {
        match self.vol_details.total_in.cmp(&self.vol_details.total_out) {
            Ordering::Greater => false,
            Ordering::Less => true,
            Ordering::Equal => false,
        }
    }

    /// Creates a new instance of self with all volume values as 18 decimals point
    pub fn scale_18(&self) -> Result<VaultVolume, PerformanceError> {
        let token_decimals: u8 = self.token.get_decimals()?;
        Ok(VaultVolume {
            id: self.id.clone(),
            token: self.token.clone(),
            vol_details: VolumeDetails {
                total_in: self.vol_details.total_in.scale_18(token_decimals)?,
                total_out: self.vol_details.total_out.scale_18(token_decimals)?,
                total_vol: self.vol_details.total_vol.scale_18(token_decimals)?,
                net_vol: self.vol_details.net_vol.scale_18(token_decimals)?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgOrderbook, SgTradeEvent, SgTradeStructPartialOrder,
        SgTradeVaultBalanceChange, SgTransaction, SgVaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};

    #[test]
    fn test_is_net_vol_negative() {
        let token_address = Address::random();
        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token".to_string()),
            symbol: Some("Token".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };

        // negative vol
        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in: U256::from(20_500_000),
                total_out: U256::from(30_000_000),
                total_vol: U256::from(50_500_000),
                net_vol: U256::from(9_500_000),
            },
        };
        assert!(vault_vol.is_net_vol_negative());

        // positive vol
        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in: U256::from(40_500_000),
                total_out: U256::from(30_000_000),
                total_vol: U256::from(50_500_000),
                net_vol: U256::from(9_500_000),
            },
        };
        assert!(!vault_vol.is_net_vol_negative());

        // equal vol
        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in: U256::from(30_000_000),
                total_out: U256::from(30_000_000),
                total_vol: U256::from(50_500_000),
                net_vol: U256::from(9_500_000),
            },
        };
        assert!(!vault_vol.is_net_vol_negative());
    }

    #[test]
    fn test_vaults_vol() {
        let bytes = SgBytes("".to_string());
        let bigint = SgBigInt("".to_string());
        let token1_address = Address::random();
        let token2_address = Address::random();
        let vault_id1 = B256::random();
        let vault_id2 = B256::random();
        let token1 = SgErc20 {
            id: SgBytes(token1_address.to_string()),
            address: SgBytes(token1_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };
        let token2 = SgErc20 {
            id: SgBytes(token2_address.to_string()),
            address: SgBytes(token2_address.to_string()),
            name: Some("Token2".to_string()),
            symbol: Some("Token2".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };
        let trade1 = SgTrade {
            id: bytes.clone(),
            order: SgTradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: SgTradeEvent {
                sender: bytes.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: SgOrderbook { id: bytes.clone() },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("-2".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("5".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
        };
        let trade2 = SgTrade {
            id: bytes.clone(),
            order: SgTradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: SgTradeEvent {
                sender: bytes.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: SgOrderbook { id: bytes.clone() },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("-7".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("3".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
        };

        let result = get_vaults_vol(&[trade1, trade2]).unwrap();
        let expected = vec![
            VaultVolume {
                id: vault_id2.to_string(),
                token: token2,
                vol_details: VolumeDetails {
                    total_in: U256::from(5),
                    total_out: U256::from(7),
                    total_vol: U256::from(12),
                    net_vol: U256::from(2),
                },
            },
            VaultVolume {
                id: vault_id1.to_string(),
                token: token1,
                vol_details: VolumeDetails {
                    total_in: U256::from(3),
                    total_out: U256::from(2),
                    total_vol: U256::from(5),
                    net_vol: U256::from(1),
                },
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_vaults_vol_overflow() {
        let bytes = SgBytes("".to_string());
        let bigint = SgBigInt("".to_string());
        let token1_address = Address::random();
        let token2_address = Address::random();
        let vault_id1 = B256::random();
        let vault_id2 = B256::random();

        let token1 = SgErc20 {
            id: SgBytes(token1_address.to_string()),
            address: SgBytes(token1_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };

        let token2 = SgErc20 {
            id: SgBytes(token2_address.to_string()),
            address: SgBytes(token2_address.to_string()),
            name: Some("Token2".to_string()),
            symbol: Some("Token2".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };

        // Test overflow on addition
        let trade1 = SgTrade {
            id: bytes.clone(),
            order: SgTradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: SgTradeEvent {
                sender: bytes.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: SgOrderbook { id: bytes.clone() },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("-1".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt(U256::MAX.to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
        };

        let trade2 = SgTrade {
            id: bytes.clone(),
            order: SgTradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: SgTradeEvent {
                sender: bytes.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: SgOrderbook { id: bytes.clone() },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("1".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("1".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: SgTransaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: SgOrderbook { id: bytes.clone() },
            },
        };

        // Test overflow on addition
        let err = get_vaults_vol(&[trade1, trade2]).unwrap_err();
        assert!(matches!(
            err,
            PerformanceError::MathError(MathError::Overflow)
        ));
    }

    #[test]
    fn test_to_18_decimals_ok() {
        let token_address = Address::random();
        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token".to_string()),
            symbol: Some("Token".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };
        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in: U256::from(20_500_000),
                total_out: U256::from(30_000_000),
                total_vol: U256::from(50_500_000),
                net_vol: U256::from(9_500_000),
            },
        };

        let result = vault_vol.scale_18().unwrap();
        let expected = VaultVolume {
            id: "vault-id".to_string(),
            token,
            vol_details: VolumeDetails {
                total_in: U256::from_str("20_500_000_000_000_000_000").unwrap(),
                total_out: U256::from_str("30_000_000_000_000_000_000").unwrap(),
                total_vol: U256::from_str("50_500_000_000_000_000_000").unwrap(),
                net_vol: U256::from_str("9_500_000_000_000_000_000").unwrap(),
            },
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_18_decimals_err() {
        let token_address = Address::random();
        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token".to_string()),
            symbol: Some("Token".to_string()),
            decimals: Some(SgBigInt("bad int".to_string())),
        };
        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in: U256::from(20_500_000),
                total_out: U256::from(30_000_000),
                total_vol: U256::from(50_500_000),
                net_vol: U256::from(9_500_000),
            },
        };

        let err = vault_vol.scale_18().unwrap_err();
        assert!(matches!(err, PerformanceError::ParseIntError(_)));
    }
}
