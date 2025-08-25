use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use thiserror::Error;

static RPC_URLS: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    let api_token = env!(
        "HYPER_API_TOKEN",
        "HYPER_API_TOKEN environment variable must be set at compile time"
    );
    let mut map = HashMap::new();
    map.insert(
        8453,
        format!("https://base.rpc.hypersync.xyz/{}", api_token),
    );
    map
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperRpcClient {
    chain_id: u32,
    rpc_url: String,
}

impl HyperRpcClient {
    pub fn new(chain_id: u32) -> Result<Self, HyperRpcError> {
        if !RPC_URLS.contains_key(&chain_id) {
            return Err(HyperRpcError::UnsupportedChainId { chain_id });
        }
        Ok(Self {
            chain_id,
            rpc_url: RPC_URLS.get(&chain_id).unwrap().clone(),
        })
    }

    pub fn get_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, HyperRpcError> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_url())
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
            .post(self.get_url())
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
            .post(self.get_url())
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

    #[cfg(test)]
    fn update_rpc_url(&mut self, new_url: String) {
        self.rpc_url = new_url;
    }
}

#[derive(Debug, Error)]
pub enum HyperRpcError {
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u32 },

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
    use httpmock::MockServer;

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

    #[tokio::test]
    async fn test_get_latest_block_number_valid_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_blockNumber"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": "0x1b4"}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_latest_block_number().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 436);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_latest_block_number_rpc_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "Invalid params"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_latest_block_number().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::RpcError { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_latest_block_number_missing_result() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_latest_block_number().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::MissingField { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_logs_valid_request_all_params() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getLogs"}"#)
                .json_body_partial(r#"{"params": [{"fromBlock": "0x1", "toBlock": "0x2", "address": "0x123", "topics": [["0xabc"]]}]}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": [{"blockNumber": "0x1", "logIndex": "0x0"}]}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let topics = Some(vec![Some(vec!["0xabc".to_string()])]);
        let result = client.get_logs("0x1", "0x2", "0x123", topics).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("blockNumber"));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_logs_valid_request_none_topics() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getLogs"}"#)
                .json_body_partial(r#"{"params": [{"fromBlock": "0x1", "toBlock": "0x2", "address": "0x123", "topics": null}]}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": []}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_logs("0x1", "0x2", "0x123", None).await;
        assert!(result.is_ok());

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_logs_rpc_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32000, "message": "Query returned more than 10000 results"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_logs("0x1", "0x1000", "0x123", None).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::RpcError { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_valid_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getBlockByNumber"}"#)
                .json_body_partial(r#"{"params": ["0x1b4", false]}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": {"number": "0x1b4", "timestamp": "0x6234567"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(436).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("0x1b4"));
        assert!(response.contains("timestamp"));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_hex_conversion() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .json_body_partial(r#"{"params": ["0xff", false]}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": {"number": "0xff"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(255).await;
        assert!(result.is_ok());

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_rpc_error() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "Invalid block number"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(999999999).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::RpcError { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_latest_block_number_invalid_hex() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_blockNumber"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": "0xGGG"}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_latest_block_number().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::HexParseError(_)
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_latest_block_number_non_string_result() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_blockNumber"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": 436}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_latest_block_number().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HyperRpcError::MissingField { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_zero() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getBlockByNumber"}"#)
                .json_body_partial(r#"{"params": ["0x0", false]}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": {"number": "0x0", "timestamp": "0x0"}}"#);
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(0).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("0x0"));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_large_number() {
        let server = MockServer::start();
        let large_block = u64::MAX - 1; // 18446744073709551614
        let expected_hex = "0xfffffffffffffffe";

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getBlockByNumber"}"#)
                .json_body_partial(format!(r#"{{"params": ["{}", false]}}"#, expected_hex));
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"jsonrpc": "2.0", "id": 1, "result": {{"number": "{}", "timestamp": "0x123456"}}}}"#, expected_hex));
        });

        let mut client = HyperRpcClient::new(8453).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(large_block).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains(expected_hex));

        mock.assert();
    }
}
