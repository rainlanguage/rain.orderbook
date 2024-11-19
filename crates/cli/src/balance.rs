use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use rain_orderbook_subgraph_client::types::order::{
    OrdersListQuery, OrdersListQueryVariables,
};
use cynic::{http::ReqwestExt, QueryBuilder};

async fn fetch_vault_balance(
    client: &Client,
    url: &str,
    variables: OrdersListQueryVariables,
) -> Result<Value> {
    let query = OrdersListQuery::build(variables);

    let response = client
        .post(url)
        .cynic_query(&query)
        .send()
        .await?;

    let text = response.text().await?;
    Ok(serde_json::from_str(&text)?)
}

async fn get_data(client: &Client, url: &str, variables: OrdersListQueryVariables) -> Result<Value> {
    let data = fetch_vault_balance(client, url, variables).await?;
    if let Some(errors) = data.get("errors") {
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_balances(subgraph_url: &str) -> Result<Value> {
    let client = Client::new();

    // Define the variables for the OrdersListQuery.
    let variables = OrdersListQueryVariables {
        skip: Some(0),
        first: Some(25),
    };

    let res = get_data(&client, subgraph_url, variables).await?;
    dbg!(&res);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

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
