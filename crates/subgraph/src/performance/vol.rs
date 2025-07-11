use crate::{
    performance::PerformanceError,
    types::common::{SgErc20, SgTrade},
};
use alloy::primitives::U256;
use rain_math_float::Float;
use rain_orderbook_math::{BigUintMath, MathError};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VolumeDetails {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_in: Float,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_out: Float,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub total_vol: Float,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub net_vol: Float,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct VaultVolume {
    pub id: String,
    pub token: SgErc20,
    pub vol_details: VolumeDetails,
}
impl_wasm_traits!(VaultVolume);

#[cfg(target_family = "wasm")]
mod impls {
    use super::*;
    impl_wasm_traits!(VolumeDetails);
}

/// Helper function to update volume details based on an amount
fn update_volume_details(
    vol_details: &mut VolumeDetails,
    amount: &str,
) -> Result<(), PerformanceError> {
    let amount = serde_json::from_str(amount)?;
    let zero = Float::default();

    if amount.lt(zero)? {
        vol_details.total_out = (vol_details.total_out + amount)?;
        vol_details.total_vol = (vol_details.total_vol + amount)?;
    } else {
        vol_details.total_in = (vol_details.total_in + amount)?;
        vol_details.total_vol = (vol_details.total_vol + amount)?;
    }

    vol_details.net_vol = if !((vol_details.total_in.lt(vol_details.total_out))?) {
        (vol_details.total_in - vol_details.total_out)?
    } else {
        (vol_details.total_out - vol_details.total_in)?
    };

    Ok(())
}

/// Helper function to create new volume details from an amount
fn create_volume_details(amount: &str) -> Result<VolumeDetails, PerformanceError> {
    let amount = serde_json::from_str(amount)?;
    let zero = Float::default();

    if amount.lt(zero)? {
        let amount = (-amount)?;
        Ok(VolumeDetails {
            total_in: zero,
            total_out: amount,
            total_vol: amount,
            net_vol: amount,
        })
    } else {
        Ok(VolumeDetails {
            total_in: amount,
            total_out: zero,
            total_vol: amount,
            net_vol: amount,
        })
    }
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
    pub fn is_net_vol_negative(&self) -> Result<bool, PerformanceError> {
        Ok(self.vol_details.total_in.lt(self.vol_details.total_out)?)
    }
}

#[cfg(test)]
mod tests {
    use std::i32;

    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgOrderbook, SgTradeEvent, SgTradeStructPartialOrder,
        SgTradeVaultBalanceChange, SgTransaction, SgVaultBalanceChangeVault,
    };
    use alloy::primitives::{aliases::I224, Address, B256};
    use rain_math_float::Float;

    #[test]
    fn test_is_net_vol_negative() {
        let token_address = Address::random();
        let decimals = 6;

        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token".to_string()),
            symbol: Some("Token".to_string()),
            decimals: Some(SgBigInt(decimals.to_string())),
        };

        // negative vol
        let total_in = Float::from_fixed_decimal(U256::from(20_500_000), decimals).unwrap();
        let total_out = Float::from_fixed_decimal(U256::from(30_000_000), decimals).unwrap();
        let total_vol = Float::from_fixed_decimal(U256::from(50_500_000), decimals).unwrap();
        let net_vol = Float::from_fixed_decimal(U256::from(9_500_000), decimals).unwrap();

        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in,
                total_out,
                total_vol,
                net_vol,
            },
        };
        assert!(vault_vol.is_net_vol_negative().unwrap());

        // positive vol
        let total_in = Float::from_fixed_decimal(U256::from(40_500_000), decimals).unwrap();
        let total_out = Float::from_fixed_decimal(U256::from(30_000_000), decimals).unwrap();
        let total_vol = Float::from_fixed_decimal(U256::from(50_500_000), decimals).unwrap();
        let net_vol = Float::from_fixed_decimal(U256::from(9_500_000), decimals).unwrap();

        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in,
                total_out,
                total_vol,
                net_vol,
            },
        };
        assert!(!vault_vol.is_net_vol_negative().unwrap());

        // equal vol
        let total_in = Float::from_fixed_decimal(U256::from(30_000_000), decimals).unwrap();
        let total_out = Float::from_fixed_decimal(U256::from(30_000_000), decimals).unwrap();
        let total_vol = Float::from_fixed_decimal(U256::from(50_500_000), decimals).unwrap();
        let net_vol = Float::from_fixed_decimal(U256::from(9_500_000), decimals).unwrap();

        let vault_vol = VaultVolume {
            id: "vault-id".to_string(),
            token: token.clone(),
            vol_details: VolumeDetails {
                total_in,
                total_out,
                total_vol,
                net_vol,
            },
        };
        assert!(!vault_vol.is_net_vol_negative().unwrap());
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

        let two = Float::parse("2".to_string()).unwrap();
        let two_str = serde_json::to_string(&two).unwrap();
        let minus_two = (-two).unwrap();
        let minus_two_str = serde_json::to_string(&minus_two).unwrap();
        let three = Float::parse("3".to_string()).unwrap();
        let three_str = serde_json::to_string(&three).unwrap();
        let five = Float::parse("5".to_string()).unwrap();
        let five_str = serde_json::to_string(&five).unwrap();
        let seven = Float::parse("7".to_string()).unwrap();
        let seven_str = serde_json::to_string(&seven).unwrap();
        let minus_seven = (-seven).unwrap();
        let minus_seven_str = serde_json::to_string(&minus_seven).unwrap();
        let twelve = Float::parse("12".to_string()).unwrap();

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
                amount: SgBytes(minus_two_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBytes(vault_id1.to_string()),
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
                amount: SgBytes(five_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBytes(vault_id2.to_string()),
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
                amount: SgBytes(minus_seven_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBytes(vault_id2.to_string()),
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
                amount: SgBytes(three_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBytes(vault_id1.to_string()),
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
                    total_in: five,
                    total_out: seven,
                    total_vol: twelve,
                    net_vol: two,
                },
            },
            VaultVolume {
                id: vault_id1.to_string(),
                token: token1,
                vol_details: VolumeDetails {
                    total_in: three,
                    total_out: two,
                    total_vol: five,
                    net_vol: one,
                },
            },
        ];

        assert_eq!(result.len(), expected.len());

        assert_eq!(result[0].id, expected[0].id);
        assert_eq!(result[0].token, expected[0].token);
        assert!(result[0]
            .vol_details
            .total_in
            .eq(expected[0].vol_details.total_in)
            .unwrap());
        assert!(result[0]
            .vol_details
            .total_out
            .eq(expected[0].vol_details.total_out)
            .unwrap());
        assert!(result[0]
            .vol_details
            .total_vol
            .eq(expected[0].vol_details.total_vol)
            .unwrap());
        assert!(result[0]
            .vol_details
            .net_vol
            .eq(expected[0].vol_details.net_vol)
            .unwrap());

        assert_eq!(result[1].id, expected[1].id);
        assert_eq!(result[1].token, expected[1].token);
        assert!(result[1]
            .vol_details
            .total_in
            .eq(expected[1].vol_details.total_in)
            .unwrap());
        assert!(result[1]
            .vol_details
            .total_out
            .eq(expected[1].vol_details.total_out)
            .unwrap());
        assert!(result[1]
            .vol_details
            .total_vol
            .eq(expected[1].vol_details.total_vol)
            .unwrap());
        assert!(result[1]
            .vol_details
            .net_vol
            .eq(expected[1].vol_details.net_vol)
            .unwrap());
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

        let one = Float::parse("1".to_string()).unwrap();
        let minus_one = (-one).unwrap();
        let one_str = serde_json::to_string(&one).unwrap();
        let minus_one_str = serde_json::to_string(&minus_one).unwrap();

        let max = Float::pack_lossless(I224::MAX, i32::MAX).unwrap();
        let max_str = serde_json::to_string(&max).unwrap();

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
                amount: SgBytes(minus_one_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBytes(vault_id1.to_string()),
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
                amount: SgBytes(max_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBytes(vault_id2.to_string()),
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
                amount: SgBytes(one_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: SgBytes(vault_id2.to_string()),
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
                amount: SgBytes(one_str.clone()),
                new_vault_balance: bytes.clone(),
                old_vault_balance: bytes.clone(),
                vault: SgVaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: SgBytes(vault_id1.to_string()),
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

        // Test invalid amount
        let err = create_volume_details("bad int").unwrap_err();
        assert!(matches!(err, PerformanceError::ParseUnsignedError(_)));
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

    #[test]
    fn test_process_vault_balance_change_overflow() {
        let mut vaults_vol = Vec::new();
        let token = SgErc20 {
            id: SgBytes("token1".to_string()),
            address: SgBytes("token1".to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(18.to_string())),
        };

        // First add a value that's max - 1
        let max_minus_one = U256::MAX - U256::from(1);
        process_vault_balance_change(
            &mut vaults_vol,
            "vault1",
            &token,
            &max_minus_one.to_string(),
        )
        .unwrap();

        // Now try to add 2, which should overflow
        assert!(matches!(
            process_vault_balance_change(&mut vaults_vol, "vault1", &token, "2").unwrap_err(),
            PerformanceError::MathError(MathError::Overflow)
        ));
    }
}
