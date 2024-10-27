use super::common::*;
use crate::apy::{get_vaults_apy, TokenPair};
use crate::schema;
use crate::utils::annual_rate;
use crate::{
    types::common::{Erc20, Order, Trade},
    utils::one_18,
    vol::get_vaults_vol,
    OrderbookSubgraphClientError,
};
use alloy::primitives::{I256, U256};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use typeshare::typeshare;

#[derive(cynic::QueryVariables, Debug)]
#[typeshare]
pub struct BatchOrderDetailQueryVariables {
    #[cynic(rename = "id_list")]
    pub id_list: OrderIdList,
}

#[derive(cynic::InputObject, Debug, Clone)]
#[cynic(graphql_type = "Order_filter")]
#[typeshare]
pub struct OrderIdList {
    #[cynic(rename = "id_in")]
    pub id_in: Vec<Bytes>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "BatchOrderDetailQueryVariables")]
#[typeshare]
pub struct BatchOrderDetailQuery {
    #[arguments(where: $id_list)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "OrdersListQueryVariables")]
#[typeshare]
pub struct OrdersListQuery {
    #[arguments(orderBy: "timestampAdded", orderDirection: "desc", skip: $skip, first: $first, where: $filters)]
    pub orders: Vec<Order>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "IdQueryVariables")]
#[typeshare]
pub struct OrderDetailQuery {
    #[arguments(id: $id)]
    pub order: Option<Order>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultPerformance {
    /// vault id
    pub id: String,
    /// vault token
    pub token: Erc20,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,

    // vol segment
    #[typeshare(typescript(type = "string"))]
    pub total_in_vol: U256,
    #[typeshare(typescript(type = "string"))]
    pub total_out_vol: U256,
    #[typeshare(typescript(type = "string"))]
    pub total_vol: U256,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,

    // apy segment
    #[typeshare(typescript(type = "string"))]
    pub starting_capital: I256,
    #[typeshare(typescript(type = "string"))]
    pub apy: Option<I256>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct DenominatedPerformance {
    #[typeshare(typescript(type = "string"))]
    pub apy: I256,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,
    #[typeshare(typescript(type = "string"))]
    pub starting_capital: I256,
    pub token: Erc20,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct OrderPerformance {
    /// Order subgraph id
    pub order_id: String,
    /// Order hash
    pub order_hash: String,
    /// Order's orderbook
    pub orderbook: String,
    /// Order's measured performance as a whole
    pub denominated_performance: Option<DenominatedPerformance>,
    /// Start timestamp of the performance measring timeframe
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    /// End timestamp of the performance measuring timeframe
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    /// Ordder's input vaults isolated performance
    pub inputs_vaults: Vec<VaultPerformance>,
    /// Ordder's output vaults isolated performance
    pub outputs_vaults: Vec<VaultPerformance>,
}

impl OrderPerformance {
    /// Given an order and its trades and optionally a timeframe, will calculates
    /// the order performance, (apy and volume)
    /// Trades must be sorted indesc order by timestamp, this is the case if
    /// queried from subgraph using this lib functionalities
    pub fn measure(
        order: &Order,
        trades: &[Trade],
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, OrderbookSubgraphClientError> {
        if trades.is_empty() {
            return Ok(OrderPerformance {
                order_id: order.id.0.clone(),
                order_hash: order.order_hash.0.clone(),
                orderbook: order.orderbook.id.0.clone(),
                start_time: start_timestamp.unwrap_or(0),
                end_time: end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64),
                inputs_vaults: vec![],
                outputs_vaults: vec![],
                denominated_performance: None,
            });
        }
        let vols = get_vaults_vol(trades)?;
        let vaults_apy = get_vaults_apy(trades, &vols, start_timestamp, end_timestamp)?;

        // build an OrderPerformance struct
        let mut start_time = u64::MAX;
        let mut end_time = 0_u64;
        let mut inputs: Vec<VaultPerformance> = vec![];
        let mut outputs: Vec<VaultPerformance> = vec![];
        for (vault_apy, vault_vol) in vaults_apy.iter().zip(vols) {
            if vault_apy.start_time < start_time {
                start_time = vault_apy.start_time;
            }
            if vault_apy.end_time > end_time {
                end_time = vault_apy.end_time;
            }
            if order
                .inputs
                .iter()
                .any(|v| v.vault_id.0 == vault_apy.id && v.token == vault_apy.token)
            {
                inputs.push(VaultPerformance {
                    id: vault_apy.id.clone(),
                    token: vault_apy.token.clone(),
                    total_in_vol: vault_vol.total_in,
                    total_out_vol: vault_vol.total_out,
                    total_vol: vault_vol.total_vol,
                    net_vol: vault_vol.net_vol,
                    start_time: vault_apy.start_time,
                    end_time: vault_apy.end_time,
                    starting_capital: vault_apy.capital,
                    apy: vault_apy.apy,
                });
            }
            if order
                .outputs
                .iter()
                .any(|v| v.vault_id.0 == vault_apy.id && v.token == vault_apy.token)
            {
                outputs.push(VaultPerformance {
                    id: vault_apy.id.clone(),
                    token: vault_apy.token.clone(),
                    total_in_vol: vault_vol.total_in,
                    total_out_vol: vault_vol.total_out,
                    total_vol: vault_vol.total_vol,
                    net_vol: vault_vol.net_vol,
                    start_time: vault_apy.start_time,
                    end_time: vault_apy.end_time,
                    starting_capital: vault_apy.capital,
                    apy: vault_apy.apy,
                });
            }
        }
        let mut order_performance = OrderPerformance {
            order_id: order.id.0.clone(),
            order_hash: order.order_hash.0.clone(),
            orderbook: order.orderbook.id.0.clone(),
            start_time,
            end_time,
            inputs_vaults: inputs,
            outputs_vaults: outputs,
            denominated_performance: None,
        };

        // get pairs ratios
        let pair_ratio_map = get_order_pairs_ratio(order, trades);

        // try to calculate all vaults capital and volume denominated into each of
        // the order's tokens by checking if there is direct ratio between the tokens,
        // multi path ratios are ignored currently and results in None for the APY.
        // if there is a success for any of the denomination tokens, gather it in order
        // of its net vol and pick the one with highest net vol.
        // if there was no success with any of the order's tokens, simply return None
        // for the APY.
        let mut ordered_token_net_vol_map = BTreeMap::new();
        let mut full_apy_in_distinct_token_denominations = vec![];
        for token in &vaults_apy {
            let mut noway = false;
            let mut combined_capital = I256::ZERO;
            let mut combined_net_vol = I256::ZERO;
            let mut combined_annual_rate_vol = I256::ZERO;
            let mut token_net_vol_map_converted_in_current_denomination = BTreeMap::new();
            for token_vault in &vaults_apy {
                // time to year ratio
                let annual_rate = annual_rate(token_vault.start_time, token_vault.end_time);

                // sum up all token vaults' capitals and vols in the current's iteration
                // token denomination by using the direct ratio between the tokens
                if token_vault.token == token.token {
                    combined_capital += token_vault.capital;
                    combined_net_vol += token_vault.net_vol;
                    combined_annual_rate_vol += token_vault
                        .net_vol
                        .saturating_mul(one_18().get_signed())
                        .saturating_div(annual_rate);
                    token_net_vol_map_converted_in_current_denomination
                        .insert(token_vault.net_vol, &token.token);
                } else {
                    let pair = TokenPair {
                        input: token.token.clone(),
                        output: token_vault.token.clone(),
                    };
                    // convert to current denomination by the direct pair ratio if exists
                    if let Some(Some(ratio)) = pair_ratio_map.get(&pair) {
                        combined_capital += token_vault
                            .capital
                            .saturating_mul(*ratio)
                            .saturating_div(one_18().get_signed());
                        combined_net_vol += token_vault
                            .net_vol
                            .saturating_mul(*ratio)
                            .saturating_div(one_18().get_signed());
                        combined_annual_rate_vol += token_vault
                            .net_vol
                            .saturating_mul(*ratio)
                            .saturating_div(one_18().get_signed())
                            .saturating_mul(one_18().get_signed())
                            .saturating_div(annual_rate);
                        token_net_vol_map_converted_in_current_denomination.insert(
                            token_vault
                                .net_vol
                                .saturating_mul(*ratio)
                                .saturating_div(one_18().get_signed()),
                            &token_vault.token,
                        );
                    } else {
                        noway = true;
                        break;
                    }
                }
            }

            // for every success apy calc in a token denomination, gather them in BTreeMap
            // this means at the end we have all the successful apy calculated in each of
            // the order's io tokens in order from highest to lowest.
            if !noway {
                if let Some(apy) = combined_annual_rate_vol
                    .saturating_mul(one_18().get_signed())
                    .checked_div(combined_capital)
                {
                    full_apy_in_distinct_token_denominations.push(DenominatedPerformance {
                        apy,
                        token: token.token.clone(),
                        starting_capital: combined_capital,
                        net_vol: combined_net_vol,
                    });
                }
            } else {
                token_net_vol_map_converted_in_current_denomination.clear();
            }

            // if we already have ordered token net vol in a denomination
            // we dont need them in other denominations in order to pick
            // the highest vol token as settelement denomination
            if ordered_token_net_vol_map.is_empty() {
                ordered_token_net_vol_map
                    .extend(token_net_vol_map_converted_in_current_denomination);
            }
        }

        // pick the denomination with highest net vol by iterating over tokens with
        // highest vol to lowest and pick the first matching matching one
        for (_, &token) in ordered_token_net_vol_map.iter().rev() {
            if let Some(denominated_apy) = full_apy_in_distinct_token_denominations
                .iter()
                .find(|&v| &v.token == token)
            {
                order_performance.denominated_performance = Some(denominated_apy.clone());
                // return early as soon as a match is found
                return Ok(order_performance);
            }
        }

        Ok(order_performance)
    }
}

/// Calculates an order's pairs' ratios from their last trades in a given list of trades
/// Trades must be sorted indesc order by timestamp, this is the case if queried from subgraph
/// using this lib functionalities
pub fn get_order_pairs_ratio(order: &Order, trades: &[Trade]) -> HashMap<TokenPair, Option<I256>> {
    let mut pair_ratio_map: HashMap<TokenPair, Option<I256>> = HashMap::new();
    for input in &order.inputs {
        for output in &order.outputs {
            let pair_as_key = TokenPair {
                input: input.token.clone(),
                output: output.token.clone(),
            };
            let inverse_pair_as_key = TokenPair {
                input: output.token.clone(),
                output: input.token.clone(),
            };
            // if not same io token and ratio map doesnt already include them
            if input.token != output.token
                && !(pair_ratio_map.contains_key(&pair_as_key)
                    || pair_ratio_map.contains_key(&inverse_pair_as_key))
            {
                // find this pairs(io or oi) latest tradetrades from list of order's
                // trades, the calculate the pair ratio (in amount/out amount) and
                // its inverse from the latest trade that involes these 2 tokens.
                let ratio = trades
                    .iter()
                    .find(|v| {
                        (v.input_vault_balance_change.vault.token == input.token
                            && v.output_vault_balance_change.vault.token == output.token)
                            || (v.output_vault_balance_change.vault.token == input.token
                                && v.input_vault_balance_change.vault.token == output.token)
                    })
                    .and_then(|latest_trade| {
                        // convert input and output amounts to 18 decimals point
                        // and then calculate the pair ratio
                        latest_trade
                            .ratio()
                            .ok()
                            .zip(latest_trade.inverse_ratio().ok())
                            .map(|(ratio, inverse_ratio)| {
                                [I256::from_raw(ratio), I256::from_raw(inverse_ratio)]
                            })
                    });

                // io
                pair_ratio_map.insert(pair_as_key, ratio.map(|v| v[0]));
                // oi
                pair_ratio_map.insert(inverse_pair_as_key, ratio.map(|v| v[1]));
            }
        }
    }

    pair_ratio_map
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Order, Orderbook, TradeEvent, TradeStructPartialOrder,
        TradeVaultBalanceChange, Transaction, Vault, VaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};
    use std::str::FromStr;

    #[test]
    fn test_get_pairs_ratio() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let result = get_order_pairs_ratio(&get_order(), &trades);
        let mut expected = HashMap::new();
        expected.insert(
            TokenPair {
                input: token2.clone(),
                output: token1.clone(),
            },
            Some(I256::from_str("285714285714285714").unwrap()),
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
    fn test_get_order_performance() {
        let order = get_order();
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token1_perf = VaultPerformance {
            id: vault1.to_string(),
            token: token1.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("5000000000000000000").unwrap(),
            starting_capital: I256::from_str("5000000000000000000").unwrap(),
            apy: Some(I256::from_str("3153600000000000000").unwrap()),
            total_in_vol: U256::from_str("7000000000000000000").unwrap(),
            total_out_vol: U256::from_str("2000000000000000000").unwrap(),
            total_vol: U256::from_str("9000000000000000000").unwrap(),
        };
        let token2_perf = VaultPerformance {
            id: vault2.to_string(),
            token: token2.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("3000000000000000000").unwrap(),
            starting_capital: I256::from_str("5000000000000000000").unwrap(),
            apy: Some(I256::from_str("1892160000000000000").unwrap()),
            total_in_vol: U256::from_str("5000000000000000000").unwrap(),
            total_out_vol: U256::from_str("2000000000000000000").unwrap(),
            total_vol: U256::from_str("7000000000000000000").unwrap(),
        };
        let result = OrderPerformance::measure(&order, &trades, Some(1), Some(10000001)).unwrap();
        let expected = OrderPerformance {
            order_id: "order-id".to_string(),
            order_hash: "".to_string(),
            orderbook: "".to_string(),
            start_time: 1,
            end_time: 10000001,
            inputs_vaults: vec![token1_perf.clone(), token2_perf.clone()],
            outputs_vaults: vec![token1_perf.clone(), token2_perf.clone()],
            denominated_performance: Some(DenominatedPerformance {
                apy: I256::from_str("2172479999999999999").unwrap(),
                token: token2,
                net_vol: I256::from_str("4428571428571428570").unwrap(),
                starting_capital: I256::from_str("6428571428571428570").unwrap(),
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
