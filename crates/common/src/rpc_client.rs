use std::str::FromStr;
use thiserror::Error;
use url::Url;

#[derive(Clone, Default, Debug)]
pub struct RpcClient;

/// Shared RPC envelope used by JSON-RPC responses
#[derive(Debug, serde::Deserialize)]
pub struct RpcEnvelope<T> {
    pub result: Option<T>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub struct BlockResponse {
    pub timestamp: Option<String>,
}

impl RpcClient {
    /// Build a HyperRPC URL from chain id and API token.
    pub fn build_hyper_url(chain_id: u32, api_token: &str) -> Result<Url, RpcClientError> {
        let base = match chain_id {
            8453 => "https://base.rpc.hypersync.xyz".to_string(),
            42161 => "https://arbitrum.rpc.hypersync.xyz".to_string(),
            _ => return Err(RpcClientError::UnsupportedChainId { chain_id }),
        };

        Ok(Url::from_str(&format!("{}/{}", base, api_token))?)
    }

    /// Fetch the latest block number by trying each `rpc_urls` sequentially until one succeeds.
    pub async fn get_latest_block_number(&self, rpc_urls: &[Url]) -> Result<u64, RpcClientError> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": []
        });

        let mut last_err: Option<RpcClientError> = None;
        for rpc_url in rpc_urls {
            let resp = client
                .post(rpc_url.to_string())
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let resp = match resp.error_for_status() {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let json: serde_json::Value = match resp.json().await {
                Ok(j) => j,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            if let Some(error) = json.get("error") {
                last_err = Some(RpcClientError::RpcError {
                    message: format!("Getting latest block: {}", error),
                });
                continue;
            }

            if let Some(result) = json.get("result") {
                if let Some(block_hex) = result.as_str() {
                    let block_hex = block_hex.strip_prefix("0x").unwrap_or(block_hex);
                    match u64::from_str_radix(block_hex, 16) {
                        Ok(n) => return Ok(n),
                        Err(e) => {
                            last_err = Some(RpcClientError::HexParseError(e));
                            continue;
                        }
                    }
                }
            }

            last_err = Some(RpcClientError::MissingField {
                field: "result".to_string(),
            });
        }

        Err(last_err.unwrap_or(RpcClientError::RpcError {
            message: "All RPC URLs failed".to_string(),
        }))
    }

    /// Fetch logs from the given `rpc_url`.
    pub async fn get_logs(
        &self,
        rpc_urls: &[Url],
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> Result<String, RpcClientError> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getLogs",
            "params": [{
                "fromBlock": from_block,
                "toBlock": to_block,
                "address": address,
                "topics": topics
            }]
        });

        let mut last_err: Option<RpcClientError> = None;
        for rpc_url in rpc_urls {
            let resp = client
                .post(rpc_url.to_string())
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let resp = match resp.error_for_status() {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(error) = json.get("error") {
                    last_err = Some(RpcClientError::RpcError {
                        message: error.to_string(),
                    });
                    continue;
                }
            }

            return Ok(text);
        }

        Err(last_err.unwrap_or(RpcClientError::RpcError {
            message: "All RPC URLs failed".to_string(),
        }))
    }

    /// Fetch a block by number from the given `rpc_url`.
    pub async fn get_block_by_number(
        &self,
        rpc_urls: &[Url],
        block_number: u64,
    ) -> Result<String, RpcClientError> {
        let client = reqwest::Client::new();
        let block_hex = format!("0x{:x}", block_number);
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBlockByNumber",
            "params": [block_hex, false]
        });

        let mut last_err: Option<RpcClientError> = None;
        for rpc_url in rpc_urls {
            let resp = client
                .post(rpc_url.to_string())
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let resp = match resp.error_for_status() {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            let text = match resp.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_err = Some(RpcClientError::NetworkError(e));
                    continue;
                }
            };

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(error) = json.get("error") {
                    last_err = Some(RpcClientError::RpcError {
                        message: error.to_string(),
                    });
                    continue;
                }
            }

            return Ok(text);
        }

        Err(last_err.unwrap_or(RpcClientError::RpcError {
            message: "All RPC URLs failed".to_string(),
        }))
    }
}

#[derive(Debug, Error)]
pub enum RpcClientError {
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u32 },

    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("RPC error: {message}")]
    RpcError { message: String },

    #[error("Missing expected field: {field}")]
    MissingField { field: String },

    #[error("Invalid hex format: {0}")]
    HexParseError(#[from] std::num::ParseIntError),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use httpmock::MockServer;

    #[test]
    fn test_build_hyper_url_supported_chain_id() {
        let url = RpcClient::build_hyper_url(8453, "test_token");
        assert!(url.is_ok());
        assert!(url.unwrap().to_string().contains("test_token"));
    }

    #[test]
    fn test_build_hyper_url_unsupported_chain_id() {
        let url = RpcClient::build_hyper_url(9999, "test_token");
        assert!(matches!(
            url.unwrap_err(),
            RpcClientError::UnsupportedChainId { chain_id: 9999 }
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_latest_block_number(&urls).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_latest_block_number(&urls).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::RpcError { .. }
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_latest_block_number(&urls).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::MissingField { .. }
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
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": [{
                    "address": "0x123",
                    "topics": ["0xabc"],
                    "data": "0x",
                    "blockNumber": "0x1",
                    "transactionHash": "0xdead",
                    "transactionIndex": "0x0",
                    "blockHash": "0xbeef",
                    "logIndex": "0x0",
                    "removed": false
                }]}"#);
        });

        let client = RpcClient;
        let topics = Some(vec![Some(vec!["0xabc".to_string()])]);
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_logs(&urls, "0x1", "0x2", "0x123", topics).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_logs(&urls, "0x1", "0x2", "0x123", None).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_logs(&urls, "0x1", "0x1000", "0x123", None).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::RpcError { .. }
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_block_by_number(&urls, 436).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_block_by_number(&urls, 255).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_block_by_number(&urls, 999999999).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::RpcError { .. }
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_latest_block_number(&urls).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::HexParseError(_)
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_latest_block_number(&urls).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::MissingField { .. }
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_block_by_number(&urls, 0).await;
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

        let client = RpcClient;
        let urls = vec![Url::from_str(&server.base_url()).unwrap()];
        let result = client.get_block_by_number(&urls, large_block).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains(expected_hex));

        mock.assert();
    }

    #[tokio::test]
    async fn test_multi_url_latest_block_failover_http_status() {
        let server1 = MockServer::start();
        let server2 = MockServer::start();

        let mock1 = server1.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_blockNumber"}"#);
            then.status(429)
                .header("content-type", "application/json")
                .body(
                    r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32005,"message":"rate limited"}}"#,
                );
        });

        let mock2 = server2.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_blockNumber"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": "0x1b4"}"#);
        });

        let client = RpcClient;
        let urls = vec![
            Url::from_str(&server1.base_url()).unwrap(),
            Url::from_str(&server2.base_url()).unwrap(),
        ];
        let result = client.get_latest_block_number(&urls).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 436);

        mock1.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_multi_url_get_logs_failover_json_error() {
        let server1 = MockServer::start();
        let server2 = MockServer::start();

        let mock1 = server1.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getLogs"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Internal error"}}"#);
        });

        let mock2 = server2.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getLogs"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"result":[]}"#);
        });

        let client = RpcClient;
        let urls = vec![
            Url::from_str(&server1.base_url()).unwrap(),
            Url::from_str(&server2.base_url()).unwrap(),
        ];
        let res = client
            .get_logs(&urls, "0x1", "0x2", "0x123", None)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&res).unwrap();
        assert!(json.get("result").is_some());
        assert!(json.get("error").is_none());

        mock1.assert();
        mock2.assert();
    }

    #[tokio::test]
    async fn test_multi_url_block_by_number_all_fail() {
        let server1 = MockServer::start();
        let server2 = MockServer::start();

        let _m1 = server1.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getBlockByNumber"}"#);
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Invalid block"}}"#,
                );
        });
        let _m2 = server2.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body_partial(r#"{"method": "eth_getBlockByNumber"}"#);
            then.status(500)
                .header("content-type", "application/json")
                .body("server error");
        });

        let client = RpcClient;
        let urls = vec![
            Url::from_str(&server1.base_url()).unwrap(),
            Url::from_str(&server2.base_url()).unwrap(),
        ];
        let err = client.get_block_by_number(&urls, 123).await.unwrap_err();
        // Should surface some error after exhausting all URLs
        match err {
            RpcClientError::RpcError { .. }
            | RpcClientError::NetworkError(_)
            | RpcClientError::MissingField { .. }
            | RpcClientError::HexParseError(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
