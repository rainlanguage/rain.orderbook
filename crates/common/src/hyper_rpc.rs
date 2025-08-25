use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use thiserror::Error;

static RPC_URLS: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(8453, "https://base.rpc.hypersync.xyz/"); // Base
    map
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperRpcClient {
    chain_id: u32,
}

impl HyperRpcClient {
    pub fn new(chain_id: u32) -> Result<Self, HyperRpcError> {
        if !RPC_URLS.contains_key(&chain_id) {
            return Err(HyperRpcError::UnsupportedChainId { chain_id });
        }
        Ok(Self { chain_id })
    }

    pub fn get_url(&self) -> Result<String, HyperRpcError> {
        let api_token =
            std::env::var("HYPER_API_TOKEN").map_err(|_| HyperRpcError::MissingApiToken)?;
        let rpc_url =
            RPC_URLS
                .get(&self.chain_id)
                .ok_or_else(|| HyperRpcError::UnsupportedChainId {
                    chain_id: self.chain_id,
                })?;
        Ok(format!("{}/{}", rpc_url, api_token))
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, HyperRpcError> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_url()?)
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
            return Err(HyperRpcError::RpcError {
                message: format!("Getting latest block: {}", error),
            });
        }

        if let Some(result) = json.get("result") {
            if let Some(block_hex) = result.as_str() {
                let block_hex = block_hex.strip_prefix("0x").unwrap_or(block_hex);
                let block_number = u64::from_str_radix(block_hex, 16)?;
                return Ok(block_number);
            }
        }

        Err(HyperRpcError::MissingField {
            field: "result".to_string(),
        })
    }

    pub async fn get_logs(
        &self,
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> Result<String, HyperRpcError> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_url()?)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_getLogs",
                "params": [{
                    "fromBlock": from_block,
                    "toBlock": to_block,
                    "address": address,
                    "topics": topics
                }]
            }))
            .send()
            .await?;

        let text = response.text().await?;

        // Check for RPC errors in the response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error) = json.get("error") {
                return Err(HyperRpcError::RpcError {
                    message: error.to_string(),
                });
            }
        }

        Ok(text)
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> Result<String, HyperRpcError> {
        let client = reqwest::Client::new();
        let block_hex = format!("0x{:x}", block_number);

        let response = client
            .post(self.get_url()?)
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
                return Err(HyperRpcError::RpcError {
                    message: error.to_string(),
                });
            }
        }

        Ok(text)
    }
}

pub async fn fetch_block_timestamps(
    chain_id: u32,
    block_numbers: Vec<u64>,
) -> Result<HashMap<u64, String>, HyperRpcError> {
    if block_numbers.is_empty() {
        return Ok(HashMap::new());
    }

    let client = HyperRpcClient::new(chain_id)?;
    let results: Vec<Result<(u64, String), HyperRpcError>> = futures::stream::iter(block_numbers)
        .map(|block_number| {
            let client = client.clone();
            async move {
                // Retry logic for failed requests
                let mut result: Result<String, HyperRpcError> = Err(HyperRpcError::MissingField {
                    field: "Not attempted".to_string(),
                });
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
                        Err(HyperRpcError::MissingField {
                            field: "timestamp".to_string(),
                        })
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

#[derive(Debug, Error)]
pub enum HyperRpcError {
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u32 },

    #[error("HYPER_API_TOKEN environment variable not set")]
    MissingApiToken,

    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("RPC error: {message}")]
    RpcError { message: String },

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing expected field: {field}")]
    MissingField { field: String },

    #[error("Invalid hex format: {0}")]
    HexParseError(#[from] std::num::ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_new_with_supported_chain_id() {
        let client = HyperRpcClient::new(8453);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.chain_id, 8453);
    }

    #[test]
    fn test_new_with_unsupported_chain_id() {
        let client = HyperRpcClient::new(9999);
        assert!(client.is_err());
        assert!(matches!(
            client.unwrap_err(),
            HyperRpcError::UnsupportedChainId { chain_id: 9999 }
        ));
    }

    #[test]
    fn test_get_url_with_valid_api_token() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_token = env::var("HYPER_API_TOKEN").ok();
        env::set_var("HYPER_API_TOKEN", "test_token_123");

        let client = HyperRpcClient::new(8453).unwrap();
        let url = client.get_url();
        assert!(url.is_ok());
        assert_eq!(
            url.unwrap(),
            "https://base.rpc.hypersync.xyz//test_token_123"
        );

        if let Some(token) = original_token {
            env::set_var("HYPER_API_TOKEN", token);
        } else {
            env::remove_var("HYPER_API_TOKEN");
        }
    }

    #[test]
    fn test_get_url_with_missing_api_token() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_token = env::var("HYPER_API_TOKEN").ok();
        env::remove_var("HYPER_API_TOKEN");

        let client = HyperRpcClient::new(8453).unwrap();
        let url = client.get_url();
        assert!(url.is_err());
        assert!(matches!(url.unwrap_err(), HyperRpcError::MissingApiToken));

        if let Some(token) = original_token {
            env::set_var("HYPER_API_TOKEN", token);
        }
    }
}
