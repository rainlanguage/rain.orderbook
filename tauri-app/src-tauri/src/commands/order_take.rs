use crate::error::CommandResult;
use rain_orderbook_common::{
    csv::TryIntoCsv, subgraph::SubgraphArgs, types::FlattenError, types::OrderTakeFlattened,
};
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn order_trades_list_write_csv(
    path: PathBuf,
    order_id: String,
    subgraph_args: SubgraphArgs,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> CommandResult<()> {
    let order_takes = subgraph_args
        .to_subgraph_client()?
        .order_trades_list_all(order_id.clone().into(), start_timestamp, end_timestamp)
        .await?;
    let order_takes_flattened: Vec<OrderTakeFlattened> = order_takes
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?;
    let csv_text = order_takes_flattened.try_into_csv()?;
    fs::write(path, csv_text)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::CommandError;

    use super::*;
    use httpmock::MockServer;
    use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
    use serde_json::{json, Value};

    fn get_single_trade_json() -> Value {
        json!({
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
    async fn test_order_trades_list_write_csv() {
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

        let path = PathBuf::from("./test.csv");
        let result = order_trades_list_write_csv(
            path.clone(),
            "order1".to_string(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
            None,
            None,
        )
        .await;
        assert!(result.is_ok());

        let expected = "id,timestamp,timestamp_display,transaction,sender,order_id,input,input_display,input_token_id,input_token_symbol,output,output_display,output_token_id,output_token_symbol
trade1,0,1970-01-01 02:00:00 AM,tx1,sender1,hash1,1,0.000000000000000001,0x1d80c49bbbcd1c0911346656b529df9e5c2f783d,WFLR,-2,-0.000000000000000002,0x12e605bc104e93b45e1ad99f9e555f659051c2bb,sFLR
trade2,1700086400,2023-11-16 01:13:20 AM,tx2,sender2,hash2,2,0.000000000000000002,0x1d80c49bbbcd1c0911346656b529df9e5c2f783d,WFLR,-5,-0.000000000000000005,0x12e605bc104e93b45e1ad99f9e555f659051c2bb,sFLR
";
        let csv_text = fs::read_to_string(path.clone()).unwrap();
        assert_eq!(csv_text, expected);

        fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn test_order_trades_list_write_csv_empty_trades() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({
              "data": {
                "trades": []
              }
            }));
        });

        let path = PathBuf::from("test.csv");
        let result = order_trades_list_write_csv(
            path.clone(),
            "order1".to_string(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
            None,
            None,
        )
        .await;
        assert!(result.is_ok());

        fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn test_order_trades_list_write_csv_malformed_response() {
        let sg_server = MockServer::start_async().await;
        sg_server.mock(|when, then| {
            when.path("/sg")
                .body_contains("\"first\":200")
                .body_contains("\"skip\":0");
            then.status(200).json_body_obj(&json!({}));
        });

        let path = PathBuf::from("test.csv");
        let err = order_trades_list_write_csv(
            path.clone(),
            "order1".to_string(),
            SubgraphArgs {
                url: sg_server.url("/sg"),
            },
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            CommandError::OrderbookSubgraphClientError(
                OrderbookSubgraphClientError::CynicClientError(_)
            )
        ));
    }
}
