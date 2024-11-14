use anyhow::Result;

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

pub async fn get_balances() -> Result<Vec<String>> {
    let query = r#"
        query MyQuery() {
            vaults {
                balance
            }
        }
    "#;

    let subgraph_url = arg_matches
        .get_one::<String>("subgraph-url")
        .map(|s| s.to_string())
        .or_else(|| env::var("ORDERBOOK_SUBGRAPH_URL").ok())
        .expect("ORDERBOOK_SUBGRAPH_URL not set");

    let res = get_data(&subgraph_url, query).await?;
    dbg!(res);
    Ok(res)
}