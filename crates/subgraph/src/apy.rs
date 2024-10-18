use crate::{
    types::common::{Erc20, Order, Trade},
    vol::{get_vaults_vol, VaultVolume},
    OrderbookSubgraphClientError,
};
use alloy::primitives::{
    utils::{format_units, parse_units, ParseUnits, Unit, UnitsError},
    I256, U256,
};
use core::f64;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use typeshare::typeshare;

pub const ONE: &str = "1000000000000000000";
pub const DAY: u64 = 60 * 60 * 24;
pub const YEAR: u64 = DAY * 365;
pub const PREFERED_DENOMINATIONS: [&str; 11] = [
    "usdt", "usdc", "dai", "frax", "mim", "usdp", "weth", "wbtc", "wpol", "wmatic", "wbnb",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct TokenVaultAPY {
    pub id: String,
    pub token: Erc20,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,
    #[typeshare(typescript(type = "string"))]
    pub capital: U256,
    #[typeshare(typescript(type = "number"))]
    pub apy: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct DenominatedAPY {
    #[typeshare(typescript(type = "number"))]
    pub apy: f64,
    pub token: Erc20,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct OrderAPY {
    pub order_id: String,
    pub order_hash: String,
    pub apy: Option<DenominatedAPY>,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    pub inputs_token_vault_apy: Vec<TokenVaultAPY>,
    pub outputs_token_vault_apy: Vec<TokenVaultAPY>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TokenPair {
    input: Erc20,
    output: Erc20,
}

/// Given an order and its trades and optionally a timeframe, will calculates
/// the APY for each of the entire order and for each of its vaults
pub fn get_order_apy(
    order: Order,
    trades: &[Trade],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<OrderAPY, OrderbookSubgraphClientError> {
    let one = I256::from_str(ONE).unwrap();
    if trades.is_empty() {
        return Ok(OrderAPY {
            order_id: order.id.0.clone(),
            order_hash: order.order_hash.0.clone(),
            start_time: start_timestamp.unwrap_or(0),
            end_time: end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64),
            inputs_token_vault_apy: vec![],
            outputs_token_vault_apy: vec![],
            apy: None,
        });
    }
    let vols = get_vaults_vol(trades)?;
    let token_vaults_apy = get_token_vaults_apy(trades, &vols, start_timestamp, end_timestamp)?;

    // build an OrderApy struct
    let mut start_time = u64::MAX;
    let mut end_time = 0_u64;
    let mut inputs: Vec<TokenVaultAPY> = vec![];
    let mut outputs: Vec<TokenVaultAPY> = vec![];
    for item in &token_vaults_apy {
        if item.start_time < start_time {
            start_time = item.start_time;
        }
        if item.end_time > end_time {
            end_time = item.end_time;
        }
        if order
            .inputs
            .iter()
            .any(|v| v.vault_id.0 == item.id && v.token == item.token)
        {
            inputs.push(item.clone());
        }
        if order
            .outputs
            .iter()
            .any(|v| v.vault_id.0 == item.id && v.token == item.token)
        {
            outputs.push(item.clone());
        }
    }
    let mut order_apy = OrderAPY {
        order_id: order.id.0.clone(),
        order_hash: order.order_hash.0.clone(),
        start_time,
        end_time,
        inputs_token_vault_apy: inputs,
        outputs_token_vault_apy: outputs,
        apy: None,
    };

    // get pairs ratios
    let pair_ratio_map = get_pairs_ratio(&order_apy, trades);

    // try to calculate all vaults capital and volume denominated into any of
    // the order's tokens by checking if there is direct ratio between the tokens,
    // multi path ratios are ignored currently and results in None for the APY.
    // if there is a success for any of the denomination tokens, checks if it is
    // among the prefered ones, if not continues the process with remaining tokens.
    // if none of the successfull calcs fulfills any of the prefered denominations
    // will end up picking the first one.
    // if there was no success with any of the order's tokens, simply return None
    // for the APY.
    let mut apy_denominations = vec![];
    for token in &token_vaults_apy {
        let mut noway = false;
        let mut combined_capital = I256::ZERO;
        let mut combined_annual_rate_vol = I256::ZERO;
        for token_vault in &token_vaults_apy {
            // time to year ratio with 4 point decimals
            let annual_rate = I256::from_raw(U256::from(
                ((token_vault.end_time - token_vault.start_time) * 10_000) / YEAR,
            ));
            let token_decimals = token_vault
                .token
                .decimals
                .as_ref()
                .map(|v| v.0.as_str())
                .unwrap_or("18");

            // convert to 18 point decimals
            let vault_capital =
                to_18_decimals(ParseUnits::U256(token_vault.capital), token_decimals);
            let vault_net_vol =
                to_18_decimals(ParseUnits::I256(token_vault.net_vol), token_decimals);
            if vault_capital.is_err() || vault_net_vol.is_err() {
                noway = true;
                break;
            }
            let vault_capital = vault_capital.unwrap().get_signed();
            let vault_net_vol = vault_net_vol.unwrap().get_signed();

            // sum up all capitals and vols in one denomination
            if token_vault.token == token.token {
                combined_capital += vault_capital;
                combined_annual_rate_vol += vault_net_vol.saturating_mul(annual_rate);
            } else {
                let pair = TokenPair {
                    input: token.token.clone(),
                    output: token_vault.token.clone(),
                };
                // convert to current denomination by the direct pair ratio if exists
                if let Some(Some(ratio)) = pair_ratio_map.get(&pair) {
                    combined_capital += vault_capital.saturating_mul(*ratio).saturating_div(one);
                    combined_annual_rate_vol += token_vault
                        .net_vol
                        .saturating_mul(*ratio)
                        .saturating_div(one)
                        .saturating_mul(annual_rate);
                } else {
                    noway = true;
                    break;
                }
            }
        }

        // success
        if !noway {
            // by 4 point decimals
            let int_apy = i64::try_from(
                combined_annual_rate_vol
                    .saturating_mul(I256::from_raw(U256::from(10_000)))
                    .checked_div(combined_capital)
                    .unwrap_or(I256::ZERO),
            )?;
            // div by 10_000 to convert to actual float and again by 10_000 to
            // factor in the anuual rate and then mul by 100 to convert to
            // percentage, so equals to div by 1_000_000,
            let apy = int_apy as f64 / 1_000_000f64;
            let denominated_apy = DenominatedAPY {
                apy,
                token: token.token.clone(),
            };
            // chcek if this token is one of prefered ones and if so return early
            // if not continue to next token denomination
            for denomination in PREFERED_DENOMINATIONS {
                if token
                    .token
                    .symbol
                    .as_ref()
                    .is_some_and(|sym| sym.to_ascii_lowercase().contains(denomination))
                {
                    order_apy.apy = Some(denominated_apy.clone());
                    return Ok(order_apy);
                }
            }
            apy_denominations.push(denominated_apy);
        }
    }

    // none of the order's tokens fulfilled any of the prefered denominations
    // so just pick the first one if there was any success at all
    if !apy_denominations.is_empty() {
        order_apy.apy = Some(apy_denominations[0].clone());
    }

    Ok(order_apy)
}

/// Calculates each token vault apy at the given timeframe
pub fn get_token_vaults_apy(
    trades: &[Trade],
    vols: &[VaultVolume],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<TokenVaultAPY>, OrderbookSubgraphClientError> {
    let mut token_vaults_apy: Vec<TokenVaultAPY> = vec![];
    for vol in vols {
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
                u64::from_str(&v.timestamp.0).unwrap()
                    <= u64::from_str(&first_trade.timestamp.0).unwrap() + DAY
            })
            .collect::<Vec<&&Trade>>()[0];

        // vaults starting capital at end of first day of its first ever trade
        let starting_capital = if first_day_last_trade
            .input_vault_balance_change
            .vault
            .vault_id
            .0
            == vol.id
            && first_day_last_trade.input_vault_balance_change.vault.token == vol.token
        {
            U256::from_str(
                &first_day_last_trade
                    .input_vault_balance_change
                    .new_vault_balance
                    .0,
            )?
        } else {
            U256::from_str(
                &first_day_last_trade
                    .output_vault_balance_change
                    .new_vault_balance
                    .0,
            )?
        };

        // the time range for this token vault
        let mut start = u64::from_str(&first_trade.timestamp.0)?;
        start_timestamp.inspect(|t| {
            if start > *t {
                start = *t;
            }
        });
        let end = end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64);

        // this token vault apy
        let apy = if starting_capital.is_zero() {
            0_f64
        } else {
            // by 4 point decimals
            let change_ratio = i64::try_from(
                vol.net_vol
                    .saturating_mul(I256::from_raw(U256::from(10_000)))
                    .checked_div(I256::from_raw(starting_capital))
                    .unwrap_or(I256::ZERO),
            )? as f64;
            let time_to_year_ratio = ((end - start) as f64) / YEAR as f64;
            (change_ratio * time_to_year_ratio) / 100f64
        };
        token_vaults_apy.push(TokenVaultAPY {
            id: vol.id.clone(),
            token: vol.token.clone(),
            start_time: start,
            end_time: end,
            net_vol: vol.net_vol,
            apy,
            capital: starting_capital,
        });
    }

    Ok(token_vaults_apy)
}

/// Calculates an order's pairs' ratios from their last trades in a given list of trades
fn get_pairs_ratio(order_apy: &OrderAPY, trades: &[Trade]) -> HashMap<TokenPair, Option<I256>> {
    let one = I256::from_str(ONE).unwrap();
    let mut pair_ratio_map: HashMap<TokenPair, Option<I256>> = HashMap::new();
    for input in &order_apy.inputs_token_vault_apy {
        for output in &order_apy.outputs_token_vault_apy {
            if input.token != output.token {
                // find this pairs trades from list of order's trades
                let pair_trades = trades
                    .iter()
                    .filter(|v| {
                        v.input_vault_balance_change.vault.token == input.token
                            && v.output_vault_balance_change.vault.token == output.token
                            && v.input_vault_balance_change.vault.vault_id.0 == input.id
                            && v.output_vault_balance_change.vault.vault_id.0 == output.id
                    })
                    .collect::<Vec<&Trade>>();

                // calculate the pair ratio (in amount/out amount)
                let ratio = if pair_trades.is_empty() {
                    None
                } else {
                    // convert input and output amounts to 18 decimals point
                    // and then calculate the pair ratio
                    let input_amount = to_18_decimals(
                        ParseUnits::U256(
                            U256::from_str(&pair_trades[0].input_vault_balance_change.amount.0)
                                .unwrap(),
                        ),
                        pair_trades[0]
                            .input_vault_balance_change
                            .vault
                            .token
                            .decimals
                            .as_ref()
                            .map(|v| v.0.as_str())
                            .unwrap_or("18"),
                    );
                    let output_amount = to_18_decimals(
                        ParseUnits::U256(
                            U256::from_str(
                                &pair_trades[0].output_vault_balance_change.amount.0[1..],
                            )
                            .unwrap(),
                        ),
                        pair_trades[0]
                            .output_vault_balance_change
                            .vault
                            .token
                            .decimals
                            .as_ref()
                            .map(|v| v.0.as_str())
                            .unwrap_or("18"),
                    );
                    #[allow(clippy::unnecessary_unwrap)]
                    if input_amount.is_err() || output_amount.is_err() {
                        None
                    } else {
                        Some(
                            input_amount
                                .unwrap()
                                .get_signed()
                                .saturating_mul(one)
                                .checked_div(output_amount.unwrap().get_signed())
                                .unwrap_or(I256::MAX),
                        )
                    }
                };
                pair_ratio_map.insert(
                    TokenPair {
                        input: input.token.clone(),
                        output: output.token.clone(),
                    },
                    ratio,
                );
            }
        }
    }
    pair_ratio_map
}

/// Converts a U256 or I256 to a fixed point U256 or I256 given the decimals point
pub fn to_18_decimals<T: TryInto<Unit, Error = UnitsError>>(
    amount: ParseUnits,
    decimals: T,
) -> Result<ParseUnits, UnitsError> {
    parse_units(&format_units(amount, decimals)?, 18)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
        Transaction, Vault, VaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};

    #[test]
    fn test_to_18_decimals() {
        let value = ParseUnits::I256(I256::from_str("-123456789").unwrap());
        let result = to_18_decimals(value, 5).unwrap();
        let expected = ParseUnits::I256(I256::from_str("-1234567890000000000000").unwrap());
        assert_eq!(result, expected);

        let value = ParseUnits::U256(U256::from_str("123456789").unwrap());
        let result = to_18_decimals(value, 12).unwrap();
        let expected = ParseUnits::U256(U256::from_str("123456789000000").unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_pairs_ratio() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token_vault1 = TokenVaultAPY {
            id: vault1.to_string(),
            token: token1.clone(),
            start_time: 0,
            end_time: 0,
            net_vol: I256::ZERO,
            capital: U256::ZERO,
            apy: 0f64,
        };
        let token_vault2 = TokenVaultAPY {
            id: vault2.to_string(),
            token: token2.clone(),
            start_time: 0,
            end_time: 0,
            net_vol: I256::ZERO,
            capital: U256::ZERO,
            apy: 0f64,
        };
        let order_apy = OrderAPY {
            order_id: "".to_string(),
            order_hash: "".to_string(),
            apy: None,
            start_time: 0,
            end_time: 0,
            inputs_token_vault_apy: vec![token_vault1.clone(), token_vault2.clone()],
            outputs_token_vault_apy: vec![token_vault1, token_vault2],
        };
        let result = get_pairs_ratio(&order_apy, &trades);
        let mut expected = HashMap::new();
        expected.insert(
            TokenPair {
                input: token2.clone(),
                output: token1.clone(),
            },
            Some(I256::from_str("2500000000000000000").unwrap()),
        );
        expected.insert(
            TokenPair {
                input: token1.clone(),
                output: token2.clone(),
            },
            Some(I256::from_str("3500000000000000000").unwrap()),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_token_vaults_apy() {
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
            get_token_vaults_apy(&trades, &[vault_vol1, vault_vol2], Some(1), Some(10000001))
                .unwrap();
        let expected = vec![
            TokenVaultAPY {
                id: vault1.to_string(),
                token: token1.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("1000000000000000000").unwrap(),
                capital: U256::from_str("2000000000000000000").unwrap(),
                apy: 15.854895991882293,
            },
            TokenVaultAPY {
                id: vault2.to_string(),
                token: token2.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("2000000000000000000").unwrap(),
                capital: U256::from_str("2000000000000000000").unwrap(),
                apy: 31.709791983764585,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_order_apy() {
        let order = get_order();
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token1_apy = TokenVaultAPY {
            id: vault1.to_string(),
            token: token1.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("5000000000000000000").unwrap(),
            capital: U256::from_str("2000000000000000000").unwrap(),
            apy: 79.27447995941147,
        };
        let token2_apy = TokenVaultAPY {
            id: vault2.to_string(),
            token: token2.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("3000000000000000000").unwrap(),
            capital: U256::from_str("2000000000000000000").unwrap(),
            apy: 47.564687975646876,
        };
        let result = get_order_apy(order, &trades, Some(1), Some(10000001)).unwrap();
        let expected = OrderAPY {
            order_id: "order-id".to_string(),
            order_hash: "".to_string(),
            start_time: 1,
            end_time: 10000001,
            inputs_token_vault_apy: vec![token2_apy.clone(), token1_apy.clone()],
            outputs_token_vault_apy: vec![token2_apy.clone(), token1_apy.clone()],
            apy: Some(DenominatedAPY {
                apy: 70.192857,
                token: token2,
            }),
        };

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
    fn get_order() -> Order {
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
        let vault1 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id1.to_string()),
            balance: BigInt("".to_string()),
            token: token1,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        let vault2 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id2.to_string()),
            balance: BigInt("".to_string()),
            token: token2,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        Order {
            id: Bytes("order-id".to_string()),
            order_bytes: Bytes("".to_string()),
            order_hash: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            outputs: vec![vault1.clone(), vault2.clone()],
            inputs: vec![vault1, vault2],
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            active: true,
            timestamp_added: BigInt("".to_string()),
            meta: None,
            add_events: vec![],
            trades: vec![],
        }
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
            timestamp: BigInt("1".to_string()),
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
                amount: BigInt("7000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
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
        };
        vec![trade1, trade2]
    }
}
