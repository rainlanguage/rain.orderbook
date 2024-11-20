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

pub async fn get_balances(subgraph_url: &str) -> Result<Value> {
    let mut all_orders = Vec::new();
    let mut page_skip = 0;
    let page_limit = 100; // Set the number of orders to fetch per page

    // Loop to fetch the orders in pages until all are fetched
    loop {
        let variables = OrdersListQueryVariables {
            skip: Some(page_skip),
            first: Some(page_limit),
            filters: None,
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
    async fn test_get_balances_response_is_ok() {
        //let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-10-25-af6a/gn";
        let subgraph_url = std::env::var("SUBGRAPH_URL")
        .expect("Environment variable SUBGRAPH_URL must be set for tests.");

        let result = get_balances(&subgraph_url).await;

        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_balances_data_mainnet() {
        //let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-10-25-af6a/gn";
        let subgraph_url = std::env::var("SUBGRAPH_URL")
        .expect("Environment variable SUBGRAPH_URL must be set for tests.");

        // Call the function under test
        let result = get_balances(&subgraph_url).await;

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
        }
    }
}
