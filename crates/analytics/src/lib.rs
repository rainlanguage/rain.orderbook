use std::ops::Div;

use async_trait::async_trait;

use rain_orderbook_subgraph_client::types::common::Trade;
use rain_orderbook_subgraph_client::{OrderbookSubgraphClient, OrderbookSubgraphClientError};

#[async_trait]
pub trait OrderbookSubgraphClientTrait {
    async fn all_trades_list(
        &self,
        timestamp_gte: Option<u64>,
        timestamp_lte: Option<u64>,
    ) -> Result<Vec<Trade>, OrderbookSubgraphClientError>;
}

#[async_trait]
impl OrderbookSubgraphClientTrait for OrderbookSubgraphClient {
    async fn all_trades_list(
        &self,
        timestamp_gte: Option<u64>,
        timestamp_lte: Option<u64>,
    ) -> Result<Vec<Trade>, OrderbookSubgraphClientError> {
        self.all_trades_list(timestamp_gte, timestamp_lte).await
    }
}

pub struct Analytics<T: OrderbookSubgraphClientTrait + Send + Sync> {
    client: T,
}

impl<T: OrderbookSubgraphClientTrait + Send + Sync> Analytics<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }

    /// Downtime Metrics based on time between trades
    pub async fn calculate_downtime_between_trades(
        &self,
        period: Option<(u64, u64)>,
        threshold: u64,
    ) -> (f64, f64, f64, usize, u64) {
        let trades: Vec<Trade> = match period {
            Some((start, end)) => self.client.all_trades_list(Some(start), Some(end)).await,
            None => self.client.all_trades_list(None, None).await,
        }
        .unwrap_or_default();

        let mut time_diffs: Vec<u64> = Vec::new();
        for window in trades.windows(2) {
            if let [prev, curr] = window {
                let diff = curr.timestamp.0.parse::<u64>().unwrap()
                    - prev.timestamp.0.parse::<u64>().unwrap();
                if diff >= threshold {
                    time_diffs.push(diff);
                }
            }
        }

        if time_diffs.is_empty() {
            return (0.0, 0.0, 0.0, 0, 0);
        }

        let count: usize = time_diffs.len();
        let total: u64 = time_diffs.iter().sum::<u64>();
        let avg: f64 = (total as f64).div(count as f64);
        let min: f64 = *time_diffs.iter().min().unwrap() as f64;
        let max: f64 = *time_diffs.iter().max().unwrap() as f64;

        (avg, min, max, count, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_subgraph_client::types::common::{
        BigInt, Bytes, Erc20, Orderbook, Trade, TradeEvent, TradeStructPartialOrder,
        TradeVaultBalanceChange, Transaction, VaultBalanceChangeVault,
    };

    struct MockSubgraphClient {
        trades: Vec<Trade>,
    }

    #[async_trait::async_trait]
    impl OrderbookSubgraphClientTrait for MockSubgraphClient {
        async fn all_trades_list(
            &self,
            _timestamp_gte: Option<u64>,
            _timestamp_lte: Option<u64>,
        ) -> Result<Vec<Trade>, OrderbookSubgraphClientError> {
            Ok(self.trades.clone())
        }
    }

    fn create_mock_trade(timestamp: u64) -> Trade {
        Trade {
            id: Bytes("trade_id".to_owned()),
            trade_event: TradeEvent {
                transaction: Transaction {
                    id: Bytes("transaction_id".to_owned()),
                    from: Bytes("from_address".to_owned()),
                    block_number: BigInt("1".to_owned()),
                    timestamp: BigInt(timestamp.to_string()),
                },
                sender: Bytes("sender_address".to_owned()),
            },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: Bytes("output_change_id".to_owned()),
                __typename: "TradeVaultBalanceChange".to_owned(),
                amount: BigInt("100".to_owned()),
                new_vault_balance: BigInt("1000".to_owned()),
                old_vault_balance: BigInt("900".to_owned()),
                vault: VaultBalanceChangeVault {
                    id: Bytes("vault_id".to_owned()),
                    vault_id: BigInt("1".to_owned()),
                    token: Erc20 {
                        id: Bytes("token_id".to_owned()),
                        address: Bytes("token_address".to_owned()),
                        name: Some("TokenName".to_owned()),
                        symbol: Some("TKN".to_owned()),
                        decimals: Some(BigInt("18".to_owned())),
                    },
                },
                timestamp: BigInt(timestamp.to_string()),
                transaction: Transaction {
                    id: Bytes("transaction_id".to_owned()),
                    from: Bytes("from_address".to_owned()),
                    block_number: BigInt("1".to_owned()),
                    timestamp: BigInt(timestamp.to_string()),
                },
                orderbook: Orderbook {
                    id: Bytes("orderbook_id".to_owned()),
                },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: Bytes("input_change_id".to_owned()),
                __typename: "TradeVaultBalanceChange".to_owned(),
                amount: BigInt("50".to_owned()),
                new_vault_balance: BigInt("500".to_owned()),
                old_vault_balance: BigInt("550".to_owned()),
                vault: VaultBalanceChangeVault {
                    id: Bytes("vault_id_2".to_owned()),
                    vault_id: BigInt("2".to_owned()),
                    token: Erc20 {
                        id: Bytes("token_id_2".to_owned()),
                        address: Bytes("token_address_2".to_owned()),
                        name: Some("TokenName2".to_owned()),
                        symbol: Some("TKN2".to_owned()),
                        decimals: Some(BigInt("18".to_owned())),
                    },
                },
                timestamp: BigInt(timestamp.to_string()),
                transaction: Transaction {
                    id: Bytes("transaction_id_2".to_owned()),
                    from: Bytes("from_address_2".to_owned()),
                    block_number: BigInt("1".to_owned()),
                    timestamp: BigInt(timestamp.to_string()),
                },
                orderbook: Orderbook {
                    id: Bytes("orderbook_id".to_owned()),
                },
            },
            order: TradeStructPartialOrder {
                id: Bytes("order_id".to_owned()),
                order_hash: Bytes("order_hash_value".to_owned()),
            },
            timestamp: BigInt(timestamp.to_string()),
            orderbook: Orderbook {
                id: Bytes("orderbook_id".to_owned()),
            },
        }
    }

    #[tokio::test]
    async fn test_empty_trades() {
        let client = MockSubgraphClient { trades: vec![] };
        let analytics = Analytics::new(client);

        let (avg, min, max, count, total) =
            analytics.calculate_downtime_between_trades(None, 0).await;
        assert_eq!(avg, 0.0);
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
        assert_eq!(count, 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_single_trade() {
        let client = MockSubgraphClient {
            trades: vec![create_mock_trade(1000)],
        };
        let analytics = Analytics::new(client);

        let (avg, min, max, count, total) =
            analytics.calculate_downtime_between_trades(None, 0).await;
        assert_eq!(avg, 0.0);
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
        assert_eq!(count, 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_multiple_trades() {
        let client = MockSubgraphClient {
            trades: vec![
                create_mock_trade(1000),
                create_mock_trade(1500),
                create_mock_trade(2500),
            ],
        };
        let analytics = Analytics::new(client);

        let (avg, min, max, count, total) =
            analytics.calculate_downtime_between_trades(None, 600).await;
        assert_eq!(avg, 1000.0);
        assert_eq!(min, 1000.0);
        assert_eq!(max, 1000.0);
        assert_eq!(count, 1); // Only one gap above threshold
        assert_eq!(total, 1000);
    }

    #[tokio::test]
    async fn test_trades_with_period() {
        let client = MockSubgraphClient {
            trades: vec![
                create_mock_trade(1000),
                create_mock_trade(1500),
                create_mock_trade(2500),
            ],
        };
        let analytics = Analytics::new(client);

        let (avg, min, max, count, total) = analytics
            .calculate_downtime_between_trades(Some((1000, 2500)), 400)
            .await;
        assert_eq!(avg, 750.0);
        assert_eq!(min, 500.0);
        assert_eq!(max, 1000.0);
        assert_eq!(count, 2); // Two gaps above threshold
        assert_eq!(total, 1500); // 500 + 1000
    }
}
