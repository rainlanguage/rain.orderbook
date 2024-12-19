use alloy::hex;
use anyhow::Result;
use cynic::QueryBuilder;
use rain_orderbook_subgraph_client::types::common::{
    Bytes, OrdersListQueryFilters, OrdersListQueryVariables,
};
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

pub async fn get_balances_single_order(subgraph_url: &str, order_hash: &str) -> Result<Value> {
    let hex_order_hash = &order_hash[2..];

    let variables = OrdersListQueryVariables {
        skip: None,  // No need to skip when querying a specific order
        first: None, // No need to limit since we expect a single result
        filters: Some(OrdersListQueryFilters {
            owner_in: Vec::new(),                                // Not filtering by owner
            active: None,                                        // Not filtering by active
            order_hash: Some(Bytes(hex_order_hash.to_string())), // Pass the hex string to Bytes
        }),
    };

    let res = get_data(subgraph_url, variables).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_balances_is_ok() {
        let orderbook_mainnet_subgraph_url = std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_MAINNET_SUBGRAPH_URL must be set for tests.");

        let order_hash = "0x12863c37d7dd314984b237619f569f6f6f645383bb39aec4cb219abd52f8eff2";

        let result = get_balances_single_order(&orderbook_mainnet_subgraph_url, order_hash).await;
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_balances_data_mainnet() {
        let orderbook_mainnet_subgraph_url = std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_MAINNET_SUBGRAPH_URL must be set for tests.");

        let order_hash = "0x12863c37d7dd314984b237619f569f6f6f645383bb39aec4cb219abd52f8eff2";

        let result = get_balances_single_order(&orderbook_mainnet_subgraph_url, order_hash).await;

        // Assert the function call was successful
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(data) = result {
            // Ensure "data" key exists
            let orders = data.get("data").and_then(|d| d.get("orders"));
            assert!(orders.is_some(), "Orders data missing in response");

            // Validate the returned data structure and values
            if let Some(order_array) = orders.and_then(|o| o.as_array()) {
                // Find the order with the matching `id`
                let target_order_id =
                    "0x389d61c749f571e2da90a56385600ec421b487f8679ec7a98e2dcbd888a3c1ed";
                let target_order = order_array
                    .iter()
                    .find(|order| order.get("id").map_or(false, |id| id == target_order_id));

                // Ensure the target order was found
                assert!(
                    target_order.is_some(),
                    "Order with ID {} not found",
                    target_order_id
                );

                if let Some(order) = target_order {
                    assert_eq!(
                        order.get("owner").unwrap(),
                        "0x5ef02599f44eed91ec7b3be4892b1a0665944a04"
                    );
                    assert_eq!(order.get("active").unwrap(), true);

                    // Validate the `outputs` -> `balance`
                    let outputs = order.get("outputs").unwrap().as_array().unwrap();

                    let first_output = &outputs[0];
                    assert_eq!(
                        first_output.get("balance").unwrap(),
                        "0",
                        "Unexpected balance in first output"
                    );

                    let second_output = &outputs[1];
                    assert_eq!(
                        second_output.get("balance").unwrap(),
                        "0",
                        "Unexpected balance in second output"
                    );
                }
            }
        }
    }
}
