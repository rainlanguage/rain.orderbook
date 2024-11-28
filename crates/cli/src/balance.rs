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

pub async fn get_balances_list(
    subgraph_url: &str,
    filters: OrdersListQueryFilters,
) -> Result<Value> {
    let mut all_orders = Vec::new();
    let mut page_skip = 0;
    let page_limit = 100; // Number of orders to fetch per page

    // Loop to fetch the orders in pages until all are fetched
    loop {
        let variables = OrdersListQueryVariables {
            skip: Some(page_skip),
            first: Some(page_limit),
            filters: Some(filters.clone()),
        };

        let res = get_data(subgraph_url, variables).await?;

        // Extract orders from the response
        if let Some(data) = res.get("data").and_then(|d| d.get("orders")) {
            let orders = data.as_array().unwrap();
            if orders.is_empty() {
                println!("No more orders, breaking the loop.");
                break; // Exit the loop if no more orders are returned
            }

            // Collect the orders in the all_orders vector
            all_orders.extend(orders.clone());

            // Increment skip for the next page of results
            page_skip += page_limit;
        } else {
            println!("No orders found, breaking the loop.");
            break; // Exit if there's no "orders" in the response
        }
    }

    // Return all collected orders
    Ok(serde_json::json!({ "data": { "orders": all_orders }}))
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

    #[tokio::test]
    async fn test_get_balances_response_is_ok() {
        let orderbook_mainnet_subgraph_url = std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_MAINNET_SUBGRAPH_URL must be set for tests.");

        let filters = OrdersListQueryFilters {
            owner_in: Vec::new(),
            active: None,
            order_hash: None,
        };

        let result = get_balances_list(&orderbook_mainnet_subgraph_url, filters).await;
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_balances_list_mainnet() {
        // Load the subgraph URL from the environment variable
        let orderbook_mainnet_subgraph_url = std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_MAINNET_SUBGRAPH_URL must be set for tests.");

        let order_hash = "0x12863c37d7dd314984b237619f569f6f6f645383bb39aec4cb219abd52f8eff2";

        let hex_order_hash = &order_hash[2..];

        // Create the filters struct
        let filters = OrdersListQueryFilters {
            owner_in: Vec::new(), // No owner filter for this test
            active: None,         // No active filter for this test
            order_hash: Some(Bytes(hex_order_hash.to_string())), // Filter by order hash
        };

        // Call the function and pass the filters
        let result = get_balances_list(&orderbook_mainnet_subgraph_url, filters).await;

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
