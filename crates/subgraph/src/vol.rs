use crate::types::common::{Erc20, Trade};
use alloy::primitives::{ruint::ParseError, I256, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultVolume {
    id: String,
    token: Erc20,
    #[typeshare(typescript(type = "string"))]
    total_in: U256,
    #[typeshare(typescript(type = "string"))]
    total_out: U256,
    #[typeshare(typescript(type = "string"))]
    total_vol: U256,
    #[typeshare(typescript(type = "string"))]
    net_vol: I256,
}

/// Get the vaults volume from array of trades
pub fn get_vaults_vol(trades: &[Trade]) -> Result<Vec<VaultVolume>, ParseError> {
    let mut vaults_vol: Vec<VaultVolume> = vec![];
    for trade in trades {
        if let Some(vault_vol) = vaults_vol.iter_mut().find(|v| {
            v.id == trade.input_vault_balance_change.vault.vault_id.0
                && v.token.address.0 == trade.input_vault_balance_change.vault.token.address.0
        }) {
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;
                vault_vol.total_out += amount;
                vault_vol.total_vol += amount;
                vault_vol.net_vol -= I256::from_raw(amount);
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;
                vault_vol.total_in += amount;
                vault_vol.total_vol += amount;
                vault_vol.net_vol += I256::from_raw(amount);
            }
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            let mut net_vol = I256::ZERO;
            if trade.input_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0[1..])?;
                total_out += amount;
                total_vol += amount;
                net_vol -= I256::from_raw(amount);
            } else {
                let amount = U256::from_str(&trade.input_vault_balance_change.amount.0)?;
                total_in += amount;
                total_vol += amount;
                net_vol += I256::from_raw(amount);
            }
            vaults_vol.push(VaultVolume {
                id: trade.input_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.input_vault_balance_change.vault.token.clone(),
                total_in,
                total_out,
                total_vol,
                net_vol,
            })
        }
        if let Some(vault_vol) = vaults_vol.iter_mut().find(|v| {
            v.id == trade.output_vault_balance_change.vault.vault_id.0
                && v.token.address.0 == trade.output_vault_balance_change.vault.token.address.0
        }) {
            if trade.output_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0[1..])?;
                vault_vol.total_out += amount;
                vault_vol.total_vol += amount;
                vault_vol.net_vol -= I256::from_raw(amount);
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                vault_vol.total_in += amount;
                vault_vol.total_vol += amount;
                vault_vol.net_vol += I256::from_raw(amount);
            }
        } else {
            let mut total_in = U256::ZERO;
            let mut total_out = U256::ZERO;
            let mut total_vol = U256::ZERO;
            let mut net_vol = I256::ZERO;
            if trade.output_vault_balance_change.amount.0.starts_with('-') {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0[1..])?;
                total_out += amount;
                total_vol += amount;
                net_vol -= I256::from_raw(amount);
            } else {
                let amount = U256::from_str(&trade.output_vault_balance_change.amount.0)?;
                total_in += amount;
                total_vol += amount;
                net_vol += I256::from_raw(amount);
            }
            vaults_vol.push(VaultVolume {
                id: trade.output_vault_balance_change.vault.vault_id.0.clone(),
                token: trade.output_vault_balance_change.vault.token.clone(),
                total_in,
                total_out,
                total_vol,
                net_vol,
            })
        }
    }
    Ok(vaults_vol)
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
    fn test_get_vaults_vol() {
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
                total_in: U256::from(5),
                total_out: U256::from(7),
                total_vol: U256::from(12),
                net_vol: I256::from_str("-2").unwrap(),
            },
            VaultVolume {
                id: vault_id1.to_string(),
                token: token1,
                total_in: U256::from(3),
                total_out: U256::from(2),
                total_vol: U256::from(5),
                net_vol: I256::from_str("1").unwrap(),
            },
        ];

        assert_eq!(result, expected);
    }
}
