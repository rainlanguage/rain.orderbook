use anyhow::Result;
use rain_orderbook_subgraph_client::types::order::{OrderDetailQuery, Order};
use rain_orderbook_subgraph_client::client::OrderbookSubgraphClient;
use std::str::FromStr;

/// Fetches the order details and returns the balances
pub async fn get_balances_single_order(subgraph_client: &OrderbookSubgraphClient, order_hash: &str) -> Result<Vec<String>> {
    let order_id = Bytes::from_str(order_hash)?;

    // Use the order_detail function to fetch the order details
    let order = subgraph_client.order_detail(order_id).await?;

    // Combine balances from both inputs and outputs into a single array
    let combined_balances: Vec<String> = order
        .inputs
        .iter()
        .map(|input| input.balance.clone())
        .chain(order.outputs.iter().map(|output| output.balance.clone()))
        .collect();

    Ok(combined_balances)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_subgraph_client::client::OrderbookSubgraphClient;

    #[tokio::test]
    async fn test_get_balances_single_order() {
        let subgraph_url = std::env::var("ORDERBOOK_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_SUBGRAPH_URL must be set.");
        let subgraph_client = OrderbookSubgraphClient::new(subgraph_url);

        let order_hash = "0x12863c37d7dd314984b237619f569f6f6f645383bb39aec4cb219abd52f8eff2";

        // Call the function to get balances
        let result = get_balances_single_order(&subgraph_client, order_hash).await;
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(combined_balances) = result {
            println!("Combined Balances: {:?}", combined_balances);
        }
    }
}
