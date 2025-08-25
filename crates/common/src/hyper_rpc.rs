use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const RPC_URL: &str = "https://base.rpc.hypersync.xyz/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    #[serde(rename = "fromBlock")]
    pub from_block: String,
    #[serde(rename = "toBlock")]
    pub to_block: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<Option<Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperRpcClient {}

impl HyperRpcClient {
    fn get_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let api_token = std::env::var("HYPER_API_TOKEN")
            .map_err(|_| "HYPER_API_TOKEN environment variable not set")?;
        Ok(format!("{}/{}", RPC_URL, api_token))
    }

    pub async fn get_latest_block_number(
        &self,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(Self::get_url()?)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_blockNumber",
                "params": []
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        // Check for RPC errors
        if let Some(error) = json.get("error") {
            return Err(format!("RPC error getting latest block: {}", error).into());
        }

        if let Some(result) = json.get("result") {
            if let Some(block_hex) = result.as_str() {
                let block_hex = block_hex.strip_prefix("0x").unwrap_or(block_hex);
                let block_number = u64::from_str_radix(block_hex, 16)?;
                return Ok(block_number);
            }
        }

        Err("No result field in response".into())
    }

    pub async fn get_logs(
        &self,
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(Self::get_url()?)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_getLogs",
                "params": [LogFilter {
                    from_block: from_block.to_string(),
                    to_block: to_block.to_string(),
                    address: address.to_string(),
                    topics: topics.clone(),
                }]
            }))
            .send()
            .await?;

        let text = response.text().await?;

        // Check for RPC errors in the response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error) = json.get("error") {
                return Err(format!("RPC error: {}", error).into());
            }
        }

        Ok(text)
    }

    pub async fn get_block_by_number(
        &self,
        block_number: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let block_hex = format!("0x{:x}", block_number);

        let response = client
            .post(Self::get_url()?)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_getBlockByNumber",
                "params": [block_hex, false]
            }))
            .send()
            .await?;

        let text = response.text().await?;

        // Check for RPC errors in the response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error) = json.get("error") {
                return Err(format!("RPC error: {}", error).into());
            }
        }

        Ok(text)
    }
}

pub async fn fetch_block_timestamps(
    block_numbers: Vec<u64>,
) -> Result<HashMap<u64, String>, Box<dyn std::error::Error + Send + Sync>> {
    if block_numbers.is_empty() {
        return Ok(HashMap::new());
    }

    let client = HyperRpcClient {};
    let results: Vec<Result<(u64, String), Box<dyn std::error::Error + Send + Sync>>> =
        futures::stream::iter(block_numbers)
            .map(|block_number| {
                let client = client.clone();
                async move {
                    // Retry logic for failed requests
                    let mut result = Err("Not attempted".into());
                    for _attempt in 1..=3 {
                        result = client.get_block_by_number(block_number).await;
                        if result.is_ok() {
                            break;
                        }
                    }

                    match result {
                        Ok(response) => {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                                if let Some(result) = json.get("result") {
                                    if let Some(block) = result.as_object() {
                                        if let Some(timestamp) =
                                            block.get("timestamp").and_then(|t| t.as_str())
                                        {
                                            return Ok((block_number, timestamp.to_string()));
                                        }
                                    }
                                }
                            }
                            Err(
                                format!("Failed to parse timestamp for block {}", block_number)
                                    .into(),
                            )
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .buffer_unordered(14) // Same concurrency as event fetching
            .collect()
            .await;

    let mut timestamps = HashMap::new();

    for (block_number, timestamp) in results.into_iter().flatten() {
        timestamps.insert(block_number, timestamp);
    }

    Ok(timestamps)
}
