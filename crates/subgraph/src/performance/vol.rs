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

fn safe_add(a: U256, b: U256) -> Result<U256, PerformanceError> {
    a.checked_add(b)
        .ok_or(PerformanceError::MathError(MathError::Overflow))
}

fn safe_sub(a: U256, b: U256) -> Result<U256, PerformanceError> {
    a.checked_sub(b)
        .ok_or(PerformanceError::MathError(MathError::Overflow))
}

/// Helper function to update volume details based on an amount
fn update_volume_details(
    vol_details: &mut VolumeDetails,
    amount: &str,
) -> Result<(), PerformanceError> {
    if amount.starts_with('-') {
        let amount = U256::from_str(&amount[1..])?;
        vol_details.total_out = safe_add(vol_details.total_out, amount)?;
        vol_details.total_vol = safe_add(vol_details.total_vol, amount)?;
    } else {
        let amount = U256::from_str(amount)?;
        vol_details.total_in = safe_add(vol_details.total_in, amount)?;
        vol_details.total_vol = safe_add(vol_details.total_vol, amount)?;
    }

    vol_details.net_vol = if vol_details.total_in >= vol_details.total_out {
        safe_sub(vol_details.total_in, vol_details.total_out)?
    } else {
        safe_sub(vol_details.total_out, vol_details.total_in)?
    };

    Ok(())
}

/// Helper function to create new volume details from an amount
fn create_volume_details(amount: &str) -> Result<VolumeDetails, PerformanceError> {
    let mut total_in = U256::ZERO;
    let mut total_out = U256::ZERO;
    let mut total_vol = U256::ZERO;

    if amount.starts_with('-') {
        let amount = U256::from_str(&amount[1..])?;
        total_out = safe_add(total_out, amount)?;
        total_vol = safe_add(total_vol, amount)?;
    } else {
        let amount = U256::from_str(amount)?;
        total_in = safe_add(total_in, amount)?;
        total_vol = safe_add(total_vol, amount)?;
    }

    let net_vol = if total_in >= total_out {
        safe_sub(total_in, total_out)?
    } else {
        safe_sub(total_out, total_in)?
    };

    Ok(VolumeDetails {
        total_in,
        total_out,
        total_vol,
        net_vol,
    })
}

/// Helper function to process a vault balance change
fn process_vault_balance_change(
    vaults_vol: &mut Vec<VaultVolume>,
    vault_id: &str,
    token: &SgErc20,
    amount: &str,
) -> Result<(), PerformanceError> {
    if let Some(vault_vol) = vaults_vol
        .iter_mut()
        .find(|v| v.id == vault_id && v.token.address.0 == token.address.0)
    {
        update_volume_details(&mut vault_vol.vol_details, amount)?;
    } else {
        vaults_vol.push(VaultVolume {
            id: vault_id.to_string(),
            token: token.clone(),
            vol_details: create_volume_details(amount)?,
        });
    }
    Ok(())
}

/// Get the vaults volume from array of trades of an order
pub fn get_vaults_vol(trades: &[SgTrade]) -> Result<Vec<VaultVolume>, PerformanceError> {
    let mut vaults_vol: Vec<VaultVolume> = vec![];
    for trade in trades {
        process_vault_balance_change(
            &mut vaults_vol,
            &trade.input_vault_balance_change.vault.vault_id.0,
            &trade.input_vault_balance_change.vault.token,
            &trade.input_vault_balance_change.amount.0,
        )?;

        process_vault_balance_change(
            &mut vaults_vol,
            &trade.output_vault_balance_change.vault.vault_id.0,
            &trade.output_vault_balance_change.vault.token,
            &trade.output_vault_balance_change.amount.0,
        )?;
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

    #[test]
    fn test_update_volume_details() {
        let mut vol_details = VolumeDetails {
            total_in: U256::from(10),
            total_out: U256::from(5),
            total_vol: U256::from(15),
            net_vol: U256::from(5),
        };

        // Test positive amount
        update_volume_details(&mut vol_details, "20").unwrap();
        assert_eq!(vol_details.total_in, U256::from(30));
        assert_eq!(vol_details.total_out, U256::from(5));
        assert_eq!(vol_details.total_vol, U256::from(35));
        assert_eq!(vol_details.net_vol, U256::from(25));

        // Test negative amount
        update_volume_details(&mut vol_details, "-15").unwrap();
        assert_eq!(vol_details.total_in, U256::from(30));
        assert_eq!(vol_details.total_out, U256::from(20));
        assert_eq!(vol_details.total_vol, U256::from(50));
        assert_eq!(vol_details.net_vol, U256::from(10));
    }

    #[test]
    fn test_create_volume_details() {
        // Test positive amount
        let vol_details = create_volume_details("20").unwrap();
        assert_eq!(vol_details.total_in, U256::from(20));
        assert_eq!(vol_details.total_out, U256::from(0));
        assert_eq!(vol_details.total_vol, U256::from(20));
        assert_eq!(vol_details.net_vol, U256::from(20));

        // Test negative amount
        let vol_details = create_volume_details("-15").unwrap();
        assert_eq!(vol_details.total_in, U256::from(0));
        assert_eq!(vol_details.total_out, U256::from(15));
        assert_eq!(vol_details.total_vol, U256::from(15));
        assert_eq!(vol_details.net_vol, U256::from(15));
    }

    #[test]
    fn test_process_vault_balance_change() {
        let mut vaults_vol = Vec::new();
        let token = SgErc20 {
            id: SgBytes("token1".to_string()),
            address: SgBytes("token1".to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };

        // Test new vault
        process_vault_balance_change(&mut vaults_vol, "vault1", &token, "20").unwrap();
        assert_eq!(vaults_vol.len(), 1);
        assert_eq!(vaults_vol[0].id, "vault1");
        assert_eq!(vaults_vol[0].vol_details.total_in, U256::from(20));

        // Test existing vault
        process_vault_balance_change(&mut vaults_vol, "vault1", &token, "-10").unwrap();
        assert_eq!(vaults_vol.len(), 1);
        assert_eq!(vaults_vol[0].vol_details.total_in, U256::from(20));
        assert_eq!(vaults_vol[0].vol_details.total_out, U256::from(10));
    }
}
