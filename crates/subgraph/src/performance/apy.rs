use super::{PerformanceError, YEAR18};
use crate::{
    performance::vol::VaultVolume,
    types::common::{Erc20, Trade},
};
use alloy::primitives::U256;
use chrono::TimeDelta;
#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};
use rain_orderbook_math::{BigUintMath, ONE18};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct APYDetails {
    pub start_time: u64,
    pub end_time: u64,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub net_vol: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub capital: U256,
    #[cfg_attr(target_family = "wasm", tsify(type = "string | undefined"))]
    pub apy: Option<U256>,
    pub is_neg: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct VaultAPY {
    pub id: String,
    pub token: Erc20,
    pub apy_details: Option<APYDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenPair {
    pub input: Erc20,
    pub output: Erc20,
}

#[cfg(target_family = "wasm")]
mod impls {
    use super::*;
    impl_all_wasm_traits!(TokenPair);
    impl_all_wasm_traits!(VaultAPY);
    impl_all_wasm_traits!(APYDetails);
}

/// Calculates each token vault apy at the given timeframe
/// Trades must be sorted in desc order by timestamp, this is
/// the case if queried from subgraph using this lib functionalities
pub fn get_vaults_apy(
    trades: &[Trade],
    vols: &[VaultVolume],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<VaultAPY>, PerformanceError> {
    let mut token_vaults_apy: Vec<VaultAPY> = vec![];
    for vol in vols {
        let vol = vol.scale_18()?;
        // this token vault trades in desc order by timestamp
        let vault_trades = trades
            .iter()
            .filter(|v| {
                (v.input_vault_balance_change.vault.vault_id.0 == vol.id
                    && v.input_vault_balance_change.vault.token == vol.token)
                    || (v.output_vault_balance_change.vault.vault_id.0 == vol.id
                        && v.output_vault_balance_change.vault.token == vol.token)
            })
            .collect::<Vec<&Trade>>();

        if vault_trades.is_empty() {
            token_vaults_apy.push(VaultAPY {
                id: vol.id.clone(),
                token: vol.token.clone(),
                apy_details: None,
            });
            continue;
        }

        // this token vault first trade, indictaes the start time
        // to find the end of the first day to find the starting capital
        let first_trade = vault_trades[vault_trades.len() - 1];
        let first_day_last_trade = vault_trades
            .iter()
            .filter(|v| {
                u64::from_str(&v.timestamp.0)
                    .ok()
                    .zip(u64::from_str(&first_trade.timestamp.0).ok())
                    .is_some_and(|(trade_time, first_trade_time)| {
                        trade_time <= first_trade_time + TimeDelta::days(1).num_seconds() as u64
                    })
            })
            .collect::<Vec<&&Trade>>()[0];

        // vaults starting capital at end of first day of its first ever trade
        // as 18 point decimals
        let vault_balance_change = if first_day_last_trade
            .input_vault_balance_change
            .vault
            .vault_id
            .0
            == vol.id
            && first_day_last_trade.input_vault_balance_change.vault.token == vol.token
        {
            &first_day_last_trade.input_vault_balance_change
        } else {
            &first_day_last_trade.output_vault_balance_change
        };
        let starting_capital = U256::from_str(&vault_balance_change.new_vault_balance.0)?
            .scale_18(vault_balance_change.vault.token.get_decimals()?)
            .map_err(PerformanceError::from)?;

        // the time range for this token vault
        let mut start = u64::from_str(&first_trade.timestamp.0)?;
        start_timestamp.inspect(|t| {
            if start > *t {
                start = *t;
            }
        });
        let end = end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64);

        // this token vault apy in 18 decimals point
        let apy = if !starting_capital.is_zero() {
            match U256::from(end - start)
                .saturating_mul(ONE18)
                .div_18(*YEAR18)
            {
                Err(_) => None,
                Ok(annual_rate_18) => vol
                    .vol_details
                    .net_vol
                    .div_18(starting_capital)
                    .ok()
                    .and_then(|v| v.div_18(annual_rate_18).ok()),
            }
        } else {
            None
        };

        // this token vault apy
        token_vaults_apy.push(VaultAPY {
            id: vol.id.clone(),
            token: vol.token.clone(),
            apy_details: Some(APYDetails {
                start_time: start,
                end_time: end,
                apy,
                is_neg: vol.is_net_vol_negative(),
                net_vol: vol.vol_details.net_vol,
                capital: starting_capital,
            }),
        });
    }

    Ok(token_vaults_apy)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        performance::vol::VolumeDetails,
        types::common::{
            BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
            Transaction, VaultBalanceChangeVault,
        },
    };
    use alloy::primitives::{Address, B256};

    #[test]
    fn test_get_vaults_apy() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let vault_vol1 = VaultVolume {
            id: vault1.to_string(),
            token: token1.clone(),
            vol_details: VolumeDetails {
                total_in: U256::ZERO,
                total_out: U256::ZERO,
                total_vol: U256::ZERO,
                net_vol: U256::from_str("1000000000000000000").unwrap(),
            },
        };
        let vault_vol2 = VaultVolume {
            id: vault2.to_string(),
            token: token2.clone(),
            vol_details: VolumeDetails {
                total_in: U256::ZERO,
                total_out: U256::ZERO,
                total_vol: U256::ZERO,
                net_vol: U256::from_str("2000000000000000000").unwrap(),
            },
        };
        let result =
            get_vaults_apy(&trades, &[vault_vol1, vault_vol2], Some(1), Some(10000001)).unwrap();
        let expected = vec![
            VaultAPY {
                id: vault1.to_string(),
                token: token1.clone(),
                apy_details: Some(APYDetails {
                    start_time: 1,
                    end_time: 10000001,
                    net_vol: U256::from_str("1000000000000000000").unwrap(),
                    capital: U256::from_str("5000000000000000000").unwrap(),
                    // (1/5) / (10000001_end - 1_start / 31_536_00_year)
                    apy: Some(U256::from_str("630720000000000000").unwrap()),
                    is_neg: false,
                }),
            },
            VaultAPY {
                id: vault2.to_string(),
                token: token2.clone(),
                apy_details: Some(APYDetails {
                    start_time: 1,
                    end_time: 10000001,
                    net_vol: U256::from_str("2000000000000000000").unwrap(),
                    capital: U256::from_str("5000000000000000000").unwrap(),
                    // (2/5) / ((10000001_end - 1_start) / 31_536_00_year)
                    apy: Some(U256::from_str("1261440000000000000").unwrap()),
                    is_neg: false,
                }),
            },
        ];

        assert_eq!(result, expected);
    }

    fn get_vault_ids() -> [B256; 2] {
        [
            B256::from_slice(&[0x11u8; 32]),
            B256::from_slice(&[0x22u8; 32]),
        ]
    }
    fn get_tokens() -> [Erc20; 2] {
        let token1_address = Address::from_slice(&[0x11u8; 20]);
        let token2_address = Address::from_slice(&[0x22u8; 20]);
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
        [token1, token2]
    }

    fn get_trades() -> Vec<Trade> {
        let bytes = Bytes("".to_string());
        let bigint = BigInt("".to_string());
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
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
            timestamp: BigInt("1".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("5000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
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
            timestamp: BigInt("2".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("7000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        vec![trade2, trade1]
    }
}
