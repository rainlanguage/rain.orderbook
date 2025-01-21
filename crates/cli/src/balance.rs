use anyhow::Result;
use rain_orderbook_subgraph_client::types::common::BigInt;
use rain_orderbook_subgraph_client::OrderbookSubgraphClient;

/// Fetches the order details and returns the balances
pub async fn get_balances_single_order(
    subgraph_client: &OrderbookSubgraphClient,
    order_id: String,
) -> Result<Vec<BigInt>> {
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

    let combined_balances = input_balances
        .into_iter()
        .chain(output_balances.into_iter())
        .collect();

    Ok(combined_balances)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_subgraph_client::types::common::BigInt;
    use rain_orderbook_subgraph_client::OrderbookSubgraphClient;
    use reqwest::Url;

    #[tokio::test]
    async fn test_get_balances_single_order() {
        let default_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-12-13-7f22/gn";

        let subgraph_url = std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL")
            .expect("Environment variable ORDERBOOK_MAINNET_SUBGRAPH_URL must be set.");
        let subgraph_url =
            std::env::var("ORDERBOOK_MAINNET_SUBGRAPH_URL").unwrap_or_else(|_| default_url.to_string());

        let subgraph_url = Url::parse(&subgraph_url).expect("Invalid URL format.");

        let subgraph_client = OrderbookSubgraphClient::new(subgraph_url);

        // Test order ID
        let order_id: String =
            "0x389d61c749f571e2da90a56385600ec421b487f8679ec7a98e2dcbd888a3c1ed".to_string();

        // Call the function to get balances
        let result = get_balances_single_order(&subgraph_client, order_id).await;

        // Assert result is Ok and check the balances
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(combined_balances) = result {
            // Ensure there are balances in the result
            assert!(
                !combined_balances.is_empty(),
                "Combined balances should not be empty."
            );

            for balance in combined_balances {
                let expected_balance = BigInt("0".into());
                assert_eq!(balance, expected_balance, "Balance should be zero.");
            }
        }
    }
}
