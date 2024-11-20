use anyhow::Result;
use cynic::QueryBuilder;
use rain_orderbook_subgraph_client::types::common::OrdersListQueryVariables;
use rain_orderbook_subgraph_client::types::order::OrdersListQuery;
use reqwest::Client;
use serde_json::Value;

/// Fetches data from subgraph
async fn fetch_order_details(url: &str, variables: OrdersListQueryVariables) -> Result<Value> {
    let client = Client::new();

    // Build the GraphQL query with the provided variables.
    let query = OrdersListQuery::build(variables);

    let req = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await?;

    let text = req.text().await?;

    // Parse the response JSON.
    let response: Value = serde_json::from_str(&text)?;
    Ok(serde_json::from_str(&text)?)
}

/// Retrieves data from subgraph and checks for errors in the response.
async fn get_data(url: &str, variables: OrdersListQueryVariables) -> Result<Value> {
    let data = fetch_order_details(url, variables).await?;
    if let Some(errors) = data.get("errors") {
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_balances_single_order(subgraph_url: &str) -> Result<Value> {
    let variables = OrdersListQueryVariables {
        skip: Some(0),
        first: Some(1),
        filters: None,
    };

    let res = get_data(subgraph_url, variables).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn test_get_balances() {
        //let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-10-25-af6a/gn";
        let subgraph_url = std::env::var("SUBGRAPH_URL")
            .expect("Environment variable SUBGRAPH_URL must be set for tests.");

        let result = get_balances_single_order(&subgraph_url).await;

        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(data) = result {
            println!("Fetched balance data: {:?}", data);
        }
    }

    #[tokio::test]
    async fn test_get_balances_data_mock() {
        // Define the mock response JSON
        let mock_response = serde_json::json!({
            "data": {
                "orders": [
                    {
                        "orderBytes": "0x1234",
                        "orderHash": "0x5678",
                        "owner": "0xabcdef",
                        "outputs": [
                            {
                                "token": {
                                    "id": "1",
                                    "address": "0xdeadbeef",
                                    "name": "Token A",
                                    "symbol": "TKA",
                                    "decimals": 18
                                },
                                "balance": "1000",
                                "vaultId": "vault-123"
                            }
                        ],
                        "inputs": [],
                        "orderbook": {
                            "id": "orderbook-1"
                        },
                        "active": true,
                        "timestampAdded": "2024-11-19T12:00:00Z",
                        "addEvents": [
                            {
                                "transaction": {
                                    "blockNumber": 12345,
                                    "timestamp": "2024-11-19T12:00:00Z"
                                }
                            }
                        ],
                        "trades": []
                    }
                ]
            }
        });

        // Start a mock server
        let _mock = mock("POST", "/")
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let mock_url = &mockito::server_url();

        // Call the function under test
        let result = get_balances_single_order(mock_url).await;

        // Assert the function call was successful
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        // Validate the returned data structure and values
        if let Ok(data) = result {
            // Ensure "data" key exists
            let orders = data.get("data").and_then(|d| d.get("orders"));
            assert!(orders.is_some(), "Orders data missing in response");

            if let Some(order_array) = orders {
                assert_eq!(
                    order_array.as_array().unwrap().len(),
                    1,
                    "Unexpected number of orders"
                );

                let first_order = &order_array[0];
                assert_eq!(first_order.get("owner").unwrap(), "0xabcdef");
                assert_eq!(first_order.get("active").unwrap(), true);

                // Validate the `outputs` -> `balance`
                let outputs = first_order.get("outputs").unwrap().as_array().unwrap();
                assert_eq!(outputs.len(), 1, "Unexpected number of outputs");

                let first_output = &outputs[0];
                assert_eq!(
                    first_output.get("balance").unwrap(),
                    "1000",
                    "Unexpected balance in output"
                );
            }

            println!("Fetched balance data: {:?}", data);
        }
    }

    #[tokio::test]
    async fn test_get_balances_data_mainnet() {
        //let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-10-25-af6a/gn";
        let subgraph_url = std::env::var("SUBGRAPH_URL")
            .expect("Environment variable SUBGRAPH_URL must be set for tests.");

        // Call the function under test
        let result = get_balances_single_order(&subgraph_url).await;

        // Assert the function call was successful
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        // Validate the returned data structure and values
        if let Ok(data) = result {
            // Ensure "data" key exists
            let orders = data.get("data").and_then(|d| d.get("orders"));
            assert!(orders.is_some(), "Orders data missing in response");

            if let Some(order_array) = orders {
                let first_order = &order_array[0];
                assert_eq!(
                    first_order.get("owner").unwrap(),
                    "0xcabcbea6e523274b269fb7b21665c0b4003bd456"
                );
                assert_eq!(first_order.get("active").unwrap(), false);

                // Validate the `outputs` -> `balance`
                let outputs = first_order.get("outputs").unwrap().as_array().unwrap();
                assert_eq!(outputs.len(), 1, "Unexpected number of outputs");

                let first_output = &outputs[0];
                assert_eq!(
                    first_output.get("balance").unwrap(),
                    "0",
                    "Unexpected balance in output"
                );
            }

            println!("Fetched balance data: {:?}", data);
        }
    }
}
