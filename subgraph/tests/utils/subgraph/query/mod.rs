use anyhow::Result;
use ethers::types::Address;

mod orderbook;

pub static SG_URL: &str = "http://localhost:8000/subgraphs/name/test/test";

pub struct Query;
impl Query {
    pub async fn orderbook(id: &Address) -> Result<orderbook::QueryResponse> {
        orderbook::get_query(id).await
    }
}

async fn send_request<T: serde::Serialize>(request_body: T) -> Result<reqwest::Response> {
    let response = reqwest::Client::new()
        .post(SG_URL)
        .json(&request_body)
        .send()
        .await?;
    Ok(response)
}
