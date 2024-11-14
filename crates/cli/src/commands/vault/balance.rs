use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

async fn fetch_vault_balance(url: &str, query: &str) -> Result<Value> {
    let client = Client::new();
    let req_body = serde_json::json!({
        "query": query
    });

    let req = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(req_body.to_string())
        .send()
        .await?;
    let text = req.text().await?;
    Ok(serde_json::from_str(&text)?)
}

async fn get_data(url: &str, query: &str) -> Result<Value> {
    let data = fetch_vault_balance(url, query).await?;
    if let Some(errors) = data.get("errors") {
        return Err(anyhow::anyhow!("Error(s) occurred: {:?}", errors));
    }
    Ok(data)
}

pub async fn get_balances(subgraph_url: &str) -> Result<Value> {
    let query = r#"
        query MyQuery() {
            vaults {
                balance
            }
        }
    "#;

    let res = get_data(subgraph_url, query).await?;
    dbg!(res);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_get_balances() {
        // Replace this with your actual subgraph URL.
        let subgraph_url = "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn";

        let result = get_balances(subgraph_url).await;

        // Check that the result is not an error
        assert!(result.is_ok(), "Failed to fetch balances: {:?}", result);

        // Optionally inspect the result
        if let Ok(data) = result {
            println!("Fetched balance data: {:?}", data);
        }
    }
}
