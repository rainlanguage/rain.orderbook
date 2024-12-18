use crate::{
    performance::PerformanceError,
    types::common::{Erc20, Trade},
};
use alloy::primitives::{ruint::ParseError, U256};
use rain_orderbook_math::BigUintMath;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VolumeDetails {
    pub total_in: U256,
    pub total_out: U256,
    pub total_vol: U256,
    pub net_vol: U256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VaultVolume {
    pub id: String,
    pub token: Erc20,
    pub vol_details: VolumeDetails,
}

#[cfg(target_family = "wasm")]
mod impls {
    use super::*;
    impl_all_wasm_traits!(VolumeDetails);
    impl_all_wasm_traits!(VaultVolume);
}

/// Get the vaults volume from array of trades of an order
pub fn get_vaults_vol(trades: &[Trade]) -> Result<Vec<VaultVolume>, ParseError> {
    let mut vaults_vol: Vec<VaultVolume> = vec![];
    for trade in trades {
        if let Some(vault_vol) = vaults_vol.iter_mut().find(|v| {
            v.id == trade.input_vault_balance_change.vault.vault_id.0
                && v.token.address.0 == trade.input_vault_balance_change.vault.token.address.0
        }) {
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;
                vault_vol.vol_details.total_out += amount;
                vault_vol.vol_details.total_vol += amount;
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;
                vault_vol.vol_details.total_in += amount;
                vault_vol.vol_details.total_vol += amount;
            }
            vault_vol.vol_details.net_vol =
                if vault_vol.vol_details.total_in >= vault_vol.vol_details.total_out {
                    vault_vol.vol_details.total_in - vault_vol.vol_details.total_out
                } else {
                    vault_vol.vol_details.total_out - vault_vol.vol_details.total_in
                };
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;
                total_out += amount;
                total_vol += amount;
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;
                total_in += amount;
                total_vol += amount;
            }
            vaults_vol.push(VaultVolume {
                id: trade.input_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.input_vault_balance_change.vault.token.clone(),
                vol_details: VolumeDetails {
                    total_in,
                    total_out,
                    total_vol,
                    net_vol: if total_in >= total_out {
                        total_in - total_out
                    } else {
                        total_out - total_in
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
                vault_vol.vol_details.total_out += amount;
                vault_vol.vol_details.total_vol += amount;
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                vault_vol.vol_details.total_in += amount;
                vault_vol.vol_details.total_vol += amount;
            }
            vault_vol.vol_details.net_vol =
                if vault_vol.vol_details.total_in >= vault_vol.vol_details.total_out {
                    vault_vol.vol_details.total_in - vault_vol.vol_details.total_out
                } else {
                    vault_vol.vol_details.total_out - vault_vol.vol_details.total_in
                };
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            if trade.output_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0[1..])?;
                total_out += amount;
                total_vol += amount;
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                total_in += amount;
                total_vol += amount;
            }
            vaults_vol.push(VaultVolume {
                id: trade.output_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.output_vault_balance_change.vault.token.clone(),
                vol_details: VolumeDetails {
                    total_in,
                    total_out,
                    total_vol,
                    net_vol: if total_in >= total_out {
                        total_in - total_out
                    } else {
                        total_out - total_in
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
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
        Transaction, VaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};

    #[test]
    fn test_vaults_vol() {
        let bytes = Bytes("".to_string());
        let bigint = BigInt("".to_string());
        let token1_address = Address::random();
        let token2_address = Address::random();
        let vault_id1 = B256::random();
        let vault_id2 = B256::random();
        let token1 = Erc20 {
            id: Bytes(token1_address.to_string()),
            address: Bytes(token1_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        let token2 = Erc20 {
            id: Bytes(token2_address.to_string()),
            address: Bytes(token2_address.to_string()),
            name: Some("Token2".to_string()),
            symbol: Some("Token2".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        let trade1 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("5".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        let trade2 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: bigint.clone(),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-7".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("3".to_string()),
                new_vault_balance: bigint.clone(),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: bigint.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
                orderbook: Orderbook { id: bytes.clone() },
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
    fn test_to_18_decimals() {
        let token_address = Address::random();
        let token = Erc20 {
            id: Bytes(token_address.to_string()),
            address: Bytes(token_address.to_string()),
            name: Some("Token".to_string()),
            symbol: Some("Token".to_string()),
            decimals: Some(BigInt(6.to_string())),
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
}
