use crate::{
    types::common::{Erc20, Trade},
    utils::{annual_rate, one_18, to_18_decimals},
    vol::VaultVolume,
    OrderbookSubgraphClientError,
};
use alloy::primitives::{utils::ParseUnits, I256, U256};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultAPY {
    pub id: String,
    pub token: Erc20,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,
    #[typeshare(typescript(type = "string"))]
    pub capital: I256,
    #[typeshare(typescript(type = "string"))]
    pub apy: Option<I256>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct TokenPair {
    pub input: Erc20,
    pub output: Erc20,
}

/// Calculates each token vault apy at the given timeframe
/// Trades must be sorted indesc order by timestamp, this is
/// the case if queried from subgraph using this lib functionalities
pub fn get_vaults_apy(
    trades: &[Trade],
    vols: &[VaultVolume],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<VaultAPY>, OrderbookSubgraphClientError> {
    let mut token_vaults_apy: Vec<VaultAPY> = vec![];
    for vol in vols {
        let vol = vol.to_18_decimals()?;
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
        let starting_capital = U256::from_str(&vault_balance_change.new_vault_balance.0)
            .ok()
            .and_then(|amount| {
                to_18_decimals(
                    ParseUnits::U256(amount),
                    vault_balance_change
                        .vault
                        .token
                        .decimals
                        .as_ref()
                        .map(|v| v.0.as_str())
                        .unwrap_or("18"),
                )
                .ok()
            });

        // the time range for this token vault
        let mut start = u64::from_str(&first_trade.timestamp.0)?;
        start_timestamp.inspect(|t| {
            if start > *t {
                start = *t;
            }
        });
        let end = end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64);

        // this token vault apy in 18 decimals point
        let apy = starting_capital.and_then(|starting_capital| {
            (!starting_capital.is_zero())
                .then_some(
                    vol.net_vol
                        .saturating_mul(one_18().get_signed())
                        .saturating_div(starting_capital.get_signed())
                        .saturating_mul(one_18().get_signed())
                        .checked_div(annual_rate(start, end)),
                )
                .flatten()
        });

        // this token vault apy
        token_vaults_apy.push(VaultAPY {
            id: vol.id.clone(),
            token: vol.token.clone(),
            start_time: start,
            end_time: end,
            apy,
            net_vol: vol.net_vol,
            capital: starting_capital
                .unwrap_or(ParseUnits::I256(I256::ZERO))
                .get_signed(),
        });
    }

    Ok(token_vaults_apy)
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
    fn test_get_vaults_apy() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let vault_vol1 = VaultVolume {
            id: vault1.to_string(),
            token: token1.clone(),
            total_in: U256::ZERO,
            total_out: U256::ZERO,
            total_vol: U256::ZERO,
            net_vol: I256::from_str("1000000000000000000").unwrap(),
        };
        let vault_vol2 = VaultVolume {
            id: vault2.to_string(),
            token: token2.clone(),
            total_in: U256::ZERO,
            total_out: U256::ZERO,
            total_vol: U256::ZERO,
            net_vol: I256::from_str("2000000000000000000").unwrap(),
        };
        let result =
            get_vaults_apy(&trades, &[vault_vol1, vault_vol2], Some(1), Some(10000001)).unwrap();
        let expected = vec![
            VaultAPY {
                id: vault1.to_string(),
                token: token1.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("1000000000000000000").unwrap(),
                capital: I256::from_str("5000000000000000000").unwrap(),
                // (1/5) / (10000001_end - 1_start / 31_536_00_year)
                apy: Some(I256::from_str("630720000000000000").unwrap()),
            },
            VaultAPY {
                id: vault2.to_string(),
                token: token2.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("2000000000000000000").unwrap(),
                capital: I256::from_str("5000000000000000000").unwrap(),
                // (2/5) / ((10000001_end - 1_start) / 31_536_00_year)
                apy: Some(I256::from_str("1261440000000000000").unwrap()),
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
