use anyhow::Result;
use rain_orderbook_subgraph_client::{
    OrderbookSubgraphClient
};
use rain_orderbook_subgraph_client::types::common::BigInt;

/// Fetches the order details and returns the balances
pub async fn get_balances_single_order(subgraph_client: &OrderbookSubgraphClient, order_id: String) -> Result<Vec<BigInt>> {
    let order = subgraph_client.order_detail(order_id.into()).await?;

    let input_balances: Vec<BigInt> = order
        .inputs
        .iter()
        .map(|input| input.balance.clone())
        .collect();

    let output_balances: Vec<BigInt> = order
        .outputs
        .iter()
        .map(|output| output.balance.clone())
        .collect();

    let combined_balances = input_balances.into_iter().chain(output_balances.into_iter()).collect();

    Ok(combined_balances)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_subgraph_client::OrderbookSubgraphClient;
    use reqwest::Url;
    use rain_orderbook_subgraph_client::types::common::BigInt;

    #[tokio::test]
    async fn test_get_balances_single_order() {
        let subgraph_url = std::env::var("ORDERBOOK_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_SUBGRAPH_URL must be set.");
        let subgraph_url = Url::parse(&subgraph_url).expect("Invalid URL format.");

        let subgraph_client = OrderbookSubgraphClient::new(subgraph_url);

        // Test order ID
        let order_id: String = "0x12863c37d7dd314984b237619f569f6f6f645383bb39aec4cb219abd52f8eff2".to_string();

        // Call the function to get balances
        let result = get_balances_single_order(&subgraph_client, order_id).await;

        // Assert result is Ok and check the balances
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(combined_balances) = result {
            assert!(!combined_balances.is_empty(), "Combined balances should not be empty.");
        }
    }
}