use alloy::providers::Provider;
use alloy::rpc::json_rpc::{Id, RequestMeta};
use alloy::transports::TransportError;
use rain_orderbook_bindings::provider::{mk_read_provider, ReadProvider, ReadProviderError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::Arc;
use thiserror::Error;
use url::Url;

#[derive(Clone)]
pub struct HyperRpcClient {
    chain_id: u32,
    rpc_url: String,
    provider: Arc<ReadProvider>,
}

/// Typed view of the block payload returned by HyperSync's `eth_getBlockByNumber`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockResponse {
    pub mix_hash: Option<String>,
    pub difficulty: String,
    pub extra_data: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub hash: String,
    pub logs_bloom: String,
    pub miner: String,
    pub nonce: String,
    pub number: String,
    pub parent_hash: String,
    pub receipts_root: String,
    pub sha3_uncles: String,
    pub size: String,
    pub state_root: String,
    pub timestamp: String,
    pub total_difficulty: String,
    pub transactions_root: String,
    #[serde(default)]
    pub uncles: Vec<String>,
    #[serde(default)]
    pub transactions: Vec<String>,
    #[serde(default, flatten)]
    pub extra: Map<String, Value>,
}

/// Typed view of a single log returned by HyperSync's `eth_getLogs`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntryResponse {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: String,
    pub block_timestamp: Option<String>,
    pub transaction_hash: String,
    pub transaction_index: String,
    pub block_hash: String,
    pub log_index: String,
    pub removed: bool,
}

impl std::fmt::Debug for HyperRpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted_url = if let Some(last_slash) = self.rpc_url.rfind('/') {
            format!("{}/***", &self.rpc_url[..last_slash])
        } else {
            "***".to_string()
        };

        f.debug_struct("HyperRpcClient")
            .field("chain_id", &self.chain_id)
            .field("rpc_url", &redacted_url)
            .finish()
    }
}

impl HyperRpcClient {
    pub fn new(chain_id: u32, api_token: String) -> Result<Self, HyperRpcError> {
        let rpc_url = match chain_id {
            8453 => format!("https://base.rpc.hypersync.xyz/{}", api_token),
            _ => return Err(HyperRpcError::UnsupportedChainId { chain_id }),
        };
        let provider = Arc::new(Self::build_provider(&rpc_url)?);
        Ok(Self {
            chain_id,
            rpc_url,
            provider,
        })
    }

    pub fn get_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, HyperRpcError> {
        let response = match self
            .provider
            .client()
            .request_noparams::<Value>("eth_blockNumber")
            .map_meta(set_request_id)
            .await
        {
            Ok(value) => value,
            Err(err) => {
                let err = HyperRpcClient::map_transport_error(err, Some("Getting latest block"));
                return Err(match err {
                    HyperRpcError::JsonSerialization(_) => HyperRpcError::MissingField {
                        field: "result".to_string(),
                    },
                    other => other,
                });
            }
        };

        if let Value::String(block_hex) = response {
            let block_hex = block_hex.strip_prefix("0x").unwrap_or(&block_hex);
            let block_number = u64::from_str_radix(block_hex, 16)?;
            return Ok(block_number);
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
    ) -> Result<Vec<LogEntryResponse>, HyperRpcError> {
        let params = serde_json::json!([{
            "fromBlock": from_block,
            "toBlock": to_block,
            "address": address,
            "topics": topics,
        }]);

        self.provider
            .client()
            .request::<_, Vec<LogEntryResponse>>("eth_getLogs", params)
            .map_meta(set_request_id)
            .await
            .map_err(|err| HyperRpcClient::map_transport_error(err, None))
    }

    pub async fn get_block_by_number(
        &self,
        block_number: u64,
    ) -> Result<Option<BlockResponse>, HyperRpcError> {
        let block_hex = format!("0x{:x}", block_number);
        let params = serde_json::json!([block_hex, false]);

        self.provider
            .client()
            .request::<_, Option<BlockResponse>>("eth_getBlockByNumber", params)
            .map_meta(set_request_id)
            .await
            .map_err(|err| HyperRpcClient::map_transport_error(err, None))
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub(crate) fn update_rpc_url(&mut self, new_url: String) {
        self.rpc_url = new_url;
        let provider = Self::build_provider(&self.rpc_url).expect("failed to update provider");
        self.provider = Arc::new(provider);
    }

    fn build_provider(rpc_url: &str) -> Result<ReadProvider, HyperRpcError> {
        let parsed = Url::parse(rpc_url)?;
        Ok(mk_read_provider(&[parsed])?)
    }

    fn map_transport_error(err: TransportError, context: Option<&str>) -> HyperRpcError {
        match err {
            TransportError::ErrorResp(resp) => {
                let message = if let Some(ctx) = context {
                    format!("{}: {}", ctx, resp)
                } else {
                    resp.to_string()
                };
                HyperRpcError::RpcError { message }
            }
            TransportError::NullResp => HyperRpcError::MissingField {
                field: "result".to_string(),
            },
            TransportError::DeserError { err, text } => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    if let Some(error_value) = value.get("error") {
                        return HyperRpcError::RpcError {
                            message: error_value.to_string(),
                        };
                    }
                }
                HyperRpcError::JsonSerialization(err)
            }
            other => HyperRpcError::Transport(other),
        }
    }
}

fn set_request_id(mut meta: RequestMeta) -> RequestMeta {
    meta.id = Id::from(1_u64);
    meta
}

#[derive(Debug, Error)]
pub enum HyperRpcError {
    #[error("Unsupported chain ID: {chain_id}")]
    UnsupportedChainId { chain_id: u32 },

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Provider construction failed: {0}")]
    ProviderConstruction(#[from] ReadProviderError),

    #[error("Transport error: {0}")]
    Transport(TransportError),

    #[error("RPC error: {message}")]
    RpcError { message: String },

    #[error("Missing expected field: {field}")]
    MissingField { field: String },

    #[error("Invalid hex format: {0}")]
    HexParseError(#[from] std::num::ParseIntError),

    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

impl From<TransportError> for HyperRpcError {
    fn from(err: TransportError) -> Self {
        HyperRpcClient::map_transport_error(err, None)
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use serde_json::json;

    fn sample_block_response(number: &str, timestamp: &str) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "mixHash": "0xmix",
                "difficulty": "0x1",
                "extraData": "0xextra",
                "gasLimit": "0xffff",
                "gasUsed": "0xff",
                "hash": "0xhash",
                "logsBloom": "0x0",
                "miner": "0xminer",
                "nonce": "0xnonce",
                "number": number,
                "parentHash": "0xparent",
                "receiptsRoot": "0xreceipts",
                "sha3Uncles": "0xsha3",
                "size": "0x1",
                "stateRoot": "0xstate",
                "timestamp": timestamp,
                "totalDifficulty": "0x2",
                "transactionsRoot": "0xtransactions",
                "uncles": [],
                "transactions": [],
            }
        })
        .to_string()
    }

    fn logs_response_body(logs: serde_json::Value) -> String {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": logs,
        })
        .to_string()
    }

    fn sample_log_entry(block_number: &str) -> serde_json::Value {
        json!({
            "address": "0x123",
            "topics": ["0xabc"],
            "data": "0xdeadbeef",
            "blockNumber": block_number,
            "blockTimestamp": "0x5",
            "transactionHash": "0xtransaction",
            "transactionIndex": "0x0",
            "blockHash": "0xblock",
            "logIndex": "0x0",
            "removed": false
        })
    }

    #[test]
    fn test_new_with_supported_chain_id() {
        let client = HyperRpcClient::new(8453, "test_token".to_string());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.chain_id, 8453);
    }

    #[test]
    fn test_new_with_unsupported_chain_id() {
        let client = HyperRpcClient::new(9999, "test_token".to_string());
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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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
                .body(logs_response_body(json!([sample_log_entry("0x1")])));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let topics = Some(vec![Some(vec!["0xabc".to_string()])]);
        let result = client.get_logs("0x1", "0x2", "0x123", topics).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].block_number, "0x1");
        assert_eq!(response[0].transaction_hash, "0xtransaction");

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
                .body(logs_response_body(json!([])));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_logs("0x1", "0x2", "0x123", None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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
                .body(sample_block_response("0x1b4", "0x6234567"));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(436).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let block = response.expect("missing block data");
        assert_eq!(block.number, "0x1b4");
        assert_eq!(block.timestamp, "0x6234567");

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
                .body(sample_block_response("0xff", "0x1"));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(255).await;
        assert!(result.is_ok());
        let block = result.unwrap().expect("expected block");
        assert_eq!(block.number, "0xff");

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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
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
                .body(sample_block_response("0x0", "0x0"));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(0).await;
        assert!(result.is_ok());
        let block = result.unwrap().expect("expected block");
        assert_eq!(block.number, "0x0");
        assert_eq!(block.timestamp, "0x0");

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
                .body(sample_block_response(expected_hex, "0x123456"));
        });

        let mut client = HyperRpcClient::new(8453, "test_token".to_string()).unwrap();
        client.update_rpc_url(server.base_url());

        let result = client.get_block_by_number(large_block).await;
        assert!(result.is_ok());
        let block = result.unwrap().expect("expected block");
        assert_eq!(block.number, expected_hex);

        mock.assert();
    }

    #[test]
    fn test_map_transport_error_deser_error_with_rpc_payload() {
        let text = r#"{"error":{"code":-32000,"message":"boom"}}"#.to_string();
        let deser_err = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let mapped = HyperRpcClient::map_transport_error(
            TransportError::DeserError {
                err: deser_err,
                text,
            },
            None,
        );

        match mapped {
            HyperRpcError::RpcError { message } => {
                assert!(message.contains("boom"), "unexpected message: {message}")
            }
            other => panic!("expected RpcError, got {:?}", other),
        }
    }

    #[test]
    fn test_map_transport_error_deser_error_without_rpc_payload() {
        let text = r#"{"unexpected":"value"}"#.to_string();
        let deser_err = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let mapped = HyperRpcClient::map_transport_error(
            TransportError::DeserError {
                err: deser_err,
                text,
            },
            None,
        );

        match mapped {
            HyperRpcError::JsonSerialization(_) => {}
            other => panic!("expected JsonSerialization error, got {:?}", other),
        }
    }

    #[test]
    fn test_map_transport_error_null_response() {
        let mapped = HyperRpcClient::map_transport_error(TransportError::NullResp, None);

        match mapped {
            HyperRpcError::MissingField { ref field } if field == "result" => {}
            other => panic!("expected MissingField for 'result', got {:?}", other),
        }
    }
}
