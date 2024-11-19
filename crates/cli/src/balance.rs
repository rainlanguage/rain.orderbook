use anyhow::Result;
use cynic::QueryBuilder;
use rain_orderbook_subgraph_client::types::common::OrdersListQueryVariables;
use rain_orderbook_subgraph_client::types::order::OrdersListQuery;
use reqwest::Client;
use serde_json::Value;

async fn fetch_vault_balance(url: &str, variables: OrdersListQueryVariables) -> Result<Value> {
    let client = Client::new();

    let query = OrdersListQuery::build(variables);

    let req = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await?;

    let text = req.text().await?;
    Ok(serde_json::from_str(&text)?)
}

async fn get_data(url: &str, variables: OrdersListQueryVariables) -> Result<Value> {
    let data = fetch_vault_balance(url, variables).await?;
    if let Some(errors) = data.get("errors") {
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_balances(subgraph_url: &str) -> Result<Value> {
    let variables = OrdersListQueryVariables {
        skip: Some(0),
        first: Some(25),
        filters: None,
    };

    let res = get_data(subgraph_url, variables).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_balances() {
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn";

        let result = get_balances(subgraph_url).await;

        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        if let Ok(data) = result {
            println!("Fetched balance data: {:?}", data);
        }
    }
}
