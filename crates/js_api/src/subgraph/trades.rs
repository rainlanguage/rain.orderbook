use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{
    types::common::SgTrade, OrderbookSubgraphClient, SgPaginationArgs,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetOrderTradesListResult(#[tsify(type = "SgTrade[]")] Vec<SgTrade>);
impl_wasm_traits!(GetOrderTradesListResult);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct GetOrderTradesCountResult(#[tsify(type = "number")] u64);
impl_wasm_traits!(GetOrderTradesCountResult);

/// Fetches trade history for a specific order with optional time filtering.
///
/// Retrieves a chronological list of trades executed by an order within
/// an optional time range.
///
/// # Parameters
///
/// * `url` - Subgraph endpoint URL for the target network
/// * `order_id` - Unique order identifier (order hash)
/// * `pagination_args` - Pagination configuration:
///   - `page`: Page number (1-based)
///   - `page_size`: Number of trades per page
/// * `start_timestamp` - Optional start time filter (Unix timestamp in seconds)
/// * `end_timestamp` - Optional end time filter (Unix timestamp in seconds)
///
/// # Returns
///
/// * `Ok(GetOrderTradesListResult)` - Array of trade records with complete details
/// * `Err(SubgraphError)` - Network errors, invalid parameters, or query failures
///
/// # Examples
///
/// ```javascript
/// const result = await getOrderTradesList(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "0x1234567890abcdef1234567890abcdef12345678",
///   { page: 1, page_size: 50 }
/// );
/// if (result.error) {
///   console.error("Cannot fetch trades:", result.error.readableMsg);
///   return;
/// }
/// const trades = result.value;
/// // Do something with the trades
/// ```
#[wasm_export(
    js_name = "getOrderTradesList",
    unchecked_return_type = "GetOrderTradesListResult"
)]
pub async fn get_order_trades_list(
    url: &str,
    order_id: &str,
    pagination_args: SgPaginationArgs,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<GetOrderTradesListResult, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trades = client
        .order_trades_list(
            Id::new(order_id),
            pagination_args,
            start_timestamp,
            end_timestamp,
        )
        .await?;
    Ok(GetOrderTradesListResult(trades))
}

/// Fetches detailed information for a specific trade.
///
/// Retrieves complete information about a single trade including vault changes
/// and transaction details.
///
/// # Parameters
///
/// * `url` - Subgraph endpoint URL
/// * `trade_id` - Unique trade identifier
///
/// # Returns
///
/// * `Ok(SgTrade)` - Complete trade information
/// * `Err(SubgraphError)` - Trade not found or network errors
///
/// # Examples
///
/// ```javascript
/// const result = await getOrderTradeDetail(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "trade_123456"
/// );
/// if (result.error) {
///   console.error("Trade not found:", result.error.readableMsg);
///   return;
/// }
/// const trade = result.value;
/// // Do something with the trade
/// ```
#[wasm_export(js_name = "getOrderTradeDetail", unchecked_return_type = "SgTrade")]
pub async fn get_order_trade_detail(url: &str, trade_id: &str) -> Result<SgTrade, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    let trade = client.order_trade_detail(Id::new(trade_id)).await?;
    Ok(trade)
}

/// Counts total trades for an order within a time range.
///
/// Efficiently counts the total number of trades executed by an order without
/// fetching all trade details.
///
/// # Parameters
///
/// * `url` - Subgraph endpoint URL
/// * `order_id` - Order identifier
/// * `start_timestamp` - Optional start time filter (Unix timestamp in seconds)
/// * `end_timestamp` - Optional end time filter (Unix timestamp in seconds)
///
/// # Returns
///
/// * `Ok(GetOrderTradesCountResult)` - Total trade count as number
/// * `Err(SubgraphError)` - Network or query errors
///
/// # Examples
///
/// ```javascript
/// const result = await getOrderTradesCount(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "0x1234567890abcdef1234567890abcdef12345678"
/// );
/// if (result.error) {
///   console.error("Cannot count trades:", result.error.readableMsg);
///   return;
/// }
/// const count = result.value;
/// // Do something with the count
/// ```
#[wasm_export(
    js_name = "getOrderTradesCount",
    unchecked_return_type = "GetOrderTradesCountResult"
)]
pub async fn get_order_trades_count(
    url: &str,
    order_id: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<GetOrderTradesCountResult, SubgraphError> {
    // Create the subgraph client using the provided URL
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);

    // Fetch all trades for the specific order and calculate the count
    let trades_count = client
        .order_trades_list_all(Id::new(order_id), start_timestamp, end_timestamp)
        .await?
        .len();

    // Convert the count to a JavaScript-compatible value and return
    Ok(GetOrderTradesCountResult(trades_count as u64))
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use httpmock::MockServer;
        use rain_orderbook_subgraph_client::types::common::SgBigInt;
        use serde_json::{json, Value};

        fn get_single_trade_json() -> Value {
            json!(              {
              "id": "trade1",
              "tradeEvent": {
                "transaction": {
                  "id": "tx1",
                  "from": "from1",
                  "blockNumber": "0",
                  "timestamp": "0"
                },
                "sender": "sender1"
              },
              "outputVaultBalanceChange": {
                "id": "ovbc1",
                "__typename": "TradeVaultBalanceChange",
                "amount": "-2",
                "newVaultBalance": "0",
                "oldVaultBalance": "0",
                "vault": {
                  "id": "vault1",
                  "vaultId": "1",
                  "token": {
                    "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Staked FLR",
                    "symbol": "sFLR",
                    "decimals": "18"
                  }
                },
                "timestamp": "1700000000",
                "transaction": {
                  "id": "tx1",
                  "from": "from1",
                  "blockNumber": "0",
                  "timestamp": "1700000000"
                },
                "orderbook": {
                  "id": "ob1"
                }
              },
              "order": {
                "id": "order1",
                "orderHash": "hash1"
              },
              "inputVaultBalanceChange": {
                "id": "ivbc1",
                "__typename": "TradeVaultBalanceChange",
                "amount": "1",
                "newVaultBalance": "0",
                "oldVaultBalance": "0",
                "vault": {
                  "id": "vault1",
                  "vaultId": "1",
                  "token": {
                    "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                    "name": "Wrapped Flare",
                    "symbol": "WFLR",
                    "decimals": "18"
                  }
                },
                "timestamp": "1700000000",
                "transaction": {
                  "id": "tx1",
                  "from": "from1",
                  "blockNumber": "0",
                  "timestamp": "1700000000"
                },
                "orderbook": {
                  "id": "ob1"
                }
              },
              "timestamp": "0",
              "orderbook": {
                "id": "ob1"
              }
            })
        }
        fn get_trades_json() -> Value {
            json!([
                get_single_trade_json(),
              {
                "id": "trade2",
                "tradeEvent": {
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "0"
                  },
                  "sender": "sender2"
                },
                "outputVaultBalanceChange": {
                  "id": "ovbc2",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "-5",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault2",
                    "vaultId": "2",
                    "token": {
                      "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                      "name": "Staked FLR",
                      "symbol": "sFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "1700086400",
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": "ob2"
                  }
                },
                "order": {
                  "id": "order2",
                  "orderHash": "hash2"
                },
                "inputVaultBalanceChange": {
                  "id": "ivbc2",
                  "__typename": "TradeVaultBalanceChange",
                  "amount": "2",
                  "newVaultBalance": "0",
                  "oldVaultBalance": "0",
                  "vault": {
                    "id": "vault2",
                    "vaultId": "2",
                    "token": {
                      "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                      "name": "Wrapped Flare",
                      "symbol": "WFLR",
                      "decimals": "18"
                    }
                  },
                  "timestamp": "0",
                  "transaction": {
                    "id": "tx2",
                    "from": "from2",
                    "blockNumber": "0",
                    "timestamp": "1700086400"
                  },
                  "orderbook": {
                    "id": "ob2"
                  }
                },
                "timestamp": "1700086400",
                "orderbook": {
                  "id": "ob2"
                }
              }
            ])
        }

        #[tokio::test]
        async fn test_get_order_trades_list() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "trades": get_trades_json()
                    }
                }));
            });

            let trades = get_order_trades_list(
                &sg_server.url("/sg"),
                "order1",
                SgPaginationArgs {
                    page: 1,
                    page_size: 200,
                },
                None,
                None,
            )
            .await
            .unwrap();
            assert_eq!(trades.0.len(), 2);

            let trade1 = &trades.0[0].clone();
            assert_eq!(trade1.id.0, "trade1");
            assert_eq!(trade1.trade_event.transaction.id.0, "tx1");
            assert_eq!(trade1.trade_event.transaction.from.0, "from1");
            assert_eq!(trade1.trade_event.transaction.block_number.0, "0");
            assert_eq!(trade1.trade_event.transaction.timestamp.0, "0");
            assert_eq!(trade1.trade_event.sender.0, "sender1");
            assert_eq!(trade1.output_vault_balance_change.amount.0, "-2");
            assert_eq!(trade1.output_vault_balance_change.new_vault_balance.0, "0");
            assert_eq!(trade1.output_vault_balance_change.old_vault_balance.0, "0");
            assert_eq!(trade1.output_vault_balance_change.vault.id.0, "vault1");
            assert_eq!(trade1.output_vault_balance_change.vault.vault_id.0, "1");
            assert_eq!(
                trade1.output_vault_balance_change.vault.token.id.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade1.output_vault_balance_change.vault.token.address.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade1.output_vault_balance_change.vault.token.name,
                Some("Staked FLR".to_string())
            );
            assert_eq!(
                trade1.output_vault_balance_change.vault.token.symbol,
                Some("sFLR".to_string())
            );
            assert_eq!(
                trade1.output_vault_balance_change.vault.token.decimals,
                Some(SgBigInt("18".to_string()))
            );
            assert_eq!(trade1.output_vault_balance_change.timestamp.0, "1700000000");
            assert_eq!(trade1.output_vault_balance_change.transaction.id.0, "tx1");
            assert_eq!(
                trade1.output_vault_balance_change.transaction.from.0,
                "from1"
            );
            assert_eq!(
                trade1
                    .output_vault_balance_change
                    .transaction
                    .block_number
                    .0,
                "0"
            );
            assert_eq!(
                trade1.output_vault_balance_change.transaction.timestamp.0,
                "1700000000"
            );
            assert_eq!(trade1.order.id.0, "order1");
            assert_eq!(trade1.order.order_hash.0, "hash1");
            assert_eq!(trade1.input_vault_balance_change.amount.0, "1");
            assert_eq!(trade1.input_vault_balance_change.new_vault_balance.0, "0");
            assert_eq!(trade1.input_vault_balance_change.old_vault_balance.0, "0");
            assert_eq!(trade1.input_vault_balance_change.vault.id.0, "vault1");
            assert_eq!(trade1.input_vault_balance_change.vault.vault_id.0, "1");
            assert_eq!(
                trade1.input_vault_balance_change.vault.token.id.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade1.input_vault_balance_change.vault.token.address.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade1.input_vault_balance_change.vault.token.name,
                Some("Wrapped Flare".to_string())
            );
            assert_eq!(
                trade1.input_vault_balance_change.vault.token.symbol,
                Some("WFLR".to_string())
            );
            assert_eq!(
                trade1.input_vault_balance_change.vault.token.decimals,
                Some(SgBigInt("18".to_string()))
            );
            assert_eq!(trade1.input_vault_balance_change.timestamp.0, "1700000000");
            assert_eq!(trade1.input_vault_balance_change.transaction.id.0, "tx1");
            assert_eq!(
                trade1.input_vault_balance_change.transaction.from.0,
                "from1"
            );
            assert_eq!(
                trade1.input_vault_balance_change.transaction.block_number.0,
                "0"
            );
            assert_eq!(
                trade1.input_vault_balance_change.transaction.timestamp.0,
                "1700000000"
            );
            assert_eq!(trade1.timestamp.0, "0");
            assert_eq!(trade1.orderbook.id.0, "ob1");
            assert_eq!(trade1.order.id.0, "order1");
            assert_eq!(trade1.order.order_hash.0, "hash1");

            let trade2 = trades.0[1].clone();
            assert_eq!(trade2.id.0, "trade2");
        }

        #[tokio::test]
        async fn test_get_order_trade_detail() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "trade": get_single_trade_json()
                    }
                }));
            });

            let trade = get_order_trade_detail(&sg_server.url("/sg"), "trade1")
                .await
                .unwrap();
            assert_eq!(trade.id.0, "trade1");
            assert_eq!(trade.trade_event.transaction.id.0, "tx1");
            assert_eq!(trade.trade_event.transaction.from.0, "from1");
            assert_eq!(trade.trade_event.transaction.block_number.0, "0");
            assert_eq!(trade.trade_event.transaction.timestamp.0, "0");
            assert_eq!(trade.trade_event.sender.0, "sender1");
            assert_eq!(trade.output_vault_balance_change.amount.0, "-2");
            assert_eq!(trade.output_vault_balance_change.new_vault_balance.0, "0");
            assert_eq!(trade.output_vault_balance_change.old_vault_balance.0, "0");
            assert_eq!(trade.output_vault_balance_change.vault.id.0, "vault1");
            assert_eq!(trade.output_vault_balance_change.vault.vault_id.0, "1");
            assert_eq!(
                trade.output_vault_balance_change.vault.token.id.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade.output_vault_balance_change.vault.token.address.0,
                "0x12e605bc104e93b45e1ad99f9e555f659051c2bb"
            );
            assert_eq!(
                trade.output_vault_balance_change.vault.token.name,
                Some("Staked FLR".to_string())
            );
            assert_eq!(
                trade.output_vault_balance_change.vault.token.symbol,
                Some("sFLR".to_string())
            );
            assert_eq!(
                trade.output_vault_balance_change.vault.token.decimals,
                Some(SgBigInt("18".to_string()))
            );
            assert_eq!(trade.output_vault_balance_change.timestamp.0, "1700000000");
            assert_eq!(trade.output_vault_balance_change.transaction.id.0, "tx1");
            assert_eq!(
                trade.output_vault_balance_change.transaction.from.0,
                "from1"
            );
            assert_eq!(
                trade.output_vault_balance_change.transaction.block_number.0,
                "0"
            );
            assert_eq!(
                trade.output_vault_balance_change.transaction.timestamp.0,
                "1700000000"
            );
            assert_eq!(trade.order.id.0, "order1");
            assert_eq!(trade.order.order_hash.0, "hash1");
            assert_eq!(trade.input_vault_balance_change.amount.0, "1");
            assert_eq!(trade.input_vault_balance_change.new_vault_balance.0, "0");
            assert_eq!(trade.input_vault_balance_change.old_vault_balance.0, "0");
            assert_eq!(trade.input_vault_balance_change.vault.id.0, "vault1");
            assert_eq!(trade.input_vault_balance_change.vault.vault_id.0, "1");
            assert_eq!(
                trade.input_vault_balance_change.vault.token.id.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade.input_vault_balance_change.vault.token.address.0,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                trade.input_vault_balance_change.vault.token.name,
                Some("Wrapped Flare".to_string())
            );
            assert_eq!(
                trade.input_vault_balance_change.vault.token.symbol,
                Some("WFLR".to_string())
            );
            assert_eq!(
                trade.input_vault_balance_change.vault.token.decimals,
                Some(SgBigInt("18".to_string()))
            );
            assert_eq!(trade.input_vault_balance_change.timestamp.0, "1700000000");
            assert_eq!(trade.input_vault_balance_change.transaction.id.0, "tx1");
            assert_eq!(trade.input_vault_balance_change.transaction.from.0, "from1");
            assert_eq!(
                trade.input_vault_balance_change.transaction.block_number.0,
                "0"
            );
            assert_eq!(
                trade.input_vault_balance_change.transaction.timestamp.0,
                "1700000000"
            );
            assert_eq!(trade.timestamp.0, "0");
            assert_eq!(trade.orderbook.id.0, "ob1");
            assert_eq!(trade.order.id.0, "order1");
            assert_eq!(trade.order.order_hash.0, "hash1");
        }

        #[tokio::test]
        async fn test_get_order_trades_count() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                  "data": {
                    "trades": get_trades_json()
                  }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg")
                    .body_contains("\"first\":200")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });

            let count = get_order_trades_count(&sg_server.url("/sg"), "order1", None, None)
                .await
                .unwrap();
            assert_eq!(count.0, 2);
        }
    }
}
