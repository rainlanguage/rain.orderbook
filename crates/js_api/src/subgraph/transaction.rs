use super::SubgraphError;
use cynic::Id;
use rain_orderbook_subgraph_client::{types::common::SgTransaction, OrderbookSubgraphClient};
use reqwest::Url;
use wasm_bindgen_utils::prelude::*;

/// Fetches transaction details from the subgraph.
///
/// Retrieves basic transaction information including sender, block number,
/// and timestamp.
///
/// ## Examples
///
/// ```javascript
/// const result = await getTransaction(
///   "https://api.thegraph.com/subgraphs/name/rain-protocol/orderbook-polygon",
///   "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
/// );
/// if (result.error) {
///   console.error("Transaction not found:", result.error.readableMsg);
///   return;
/// }
/// const transaction = result.value;
/// // Do something with the transaction
/// ```
#[wasm_export(
    js_name = "getTransaction",
    unchecked_return_type = "SgTransaction",
    return_description = "Transaction details"
)]
pub async fn get_transaction(
    #[wasm_export(param_description = "Subgraph endpoint URL")] url: &str,
    #[wasm_export(param_description = "Transaction hash")] tx_hash: &str,
) -> Result<SgTransaction, SubgraphError> {
    let client = OrderbookSubgraphClient::new(Url::parse(url)?);
    Ok(client.transaction_detail(Id::new(tx_hash)).await?)
}

#[cfg(all(test, not(target_family = "wasm")))]
mod test_helpers {
    use super::*;

    mod non_wasm {
        use super::*;
        use httpmock::MockServer;
        use serde_json::json;

        #[tokio::test]
        async fn test_get_transaction() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "transaction": {
                            "id": "tx1",
                            "from": "0x1",
                            "blockNumber": "1",
                            "timestamp": "1"
                        }
                    }
                }));
            });

            let url = sg_server.url("/sg");
            let tx = get_transaction(&url, "hash").await.unwrap();
            assert_eq!(tx.id.0, "tx1".to_string());
            assert_eq!(tx.from.0, "0x1".to_string());
            assert_eq!(tx.block_number.0, "1".to_string());
            assert_eq!(tx.timestamp.0, "1".to_string());
        }
    }
}
