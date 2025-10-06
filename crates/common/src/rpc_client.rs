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
pub struct RpcClient {
    chain_id: Option<u32>,
    rpc_urls: Vec<Url>,
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

impl std::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let redacted_urls: Vec<String> = self
            .rpc_urls
            .iter()
            .map(|url| {
                let url_str = url.as_str();
                if let Some(last_slash) = url_str.rfind('/') {
                    format!("{}/***", &url_str[..last_slash])
                } else {
                    "***".to_string()
                }
            })
            .collect();

        f.debug_struct("RpcClient")
            .field("chain_id", &self.chain_id)
            .field("rpc_urls", &redacted_urls)
            .finish()
    }
}

impl RpcClient {
    pub fn new_with_urls(urls: Vec<Url>) -> Result<Self, RpcClientError> {
        if urls.is_empty() {
            return Err(RpcClientError::Config {
                message: "at least one RPC URL is required".to_string(),
            });
        }

        let provider = Arc::new(mk_read_provider(&urls)?);
        Ok(Self {
            chain_id: None,
            rpc_urls: urls,
            provider,
        })
    }

    pub fn new_with_hyper_rpc(chain_id: u32, api_token: &str) -> Result<Self, RpcClientError> {
        let url = Self::build_hyper_url(chain_id, api_token)?;
        let provider = Arc::new(mk_read_provider(&[url.clone()])?);
        Ok(Self {
            chain_id: Some(chain_id),
            rpc_urls: vec![url],
            provider,
        })
    }

    pub fn build_hyper_url(chain_id: u32, api_token: &str) -> Result<Url, RpcClientError> {
        let base = match chain_id {
            8453 => "https://base.rpc.hypersync.xyz",
            42161 => "https://arbitrum.rpc.hypersync.xyz",
            _ => return Err(RpcClientError::UnsupportedChainId { chain_id }),
        };

        let url = format!("{}/{}", base, api_token);
        Ok(Url::parse(&url)?)
    }

    pub fn rpc_urls(&self) -> &[Url] {
        &self.rpc_urls
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, RpcClientError> {
        let response = self
            .provider
            .client()
            .request_noparams::<Value>("eth_blockNumber")
            .map_meta(set_request_id)
            .await
            .map_err(|err| Self::map_transport_error(err, Some("Getting latest block")))?;

        if let Value::String(block_hex) = response {
            let block_hex = block_hex.strip_prefix("0x").unwrap_or(&block_hex);
            let block_number = u64::from_str_radix(block_hex, 16)?;
            return Ok(block_number);
        }

        Err(RpcClientError::MissingField {
            field: "result".to_string(),
        })
    }

    pub async fn get_logs(
        &self,
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> Result<Vec<LogEntryResponse>, RpcClientError> {
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
            .map_err(|err| Self::map_transport_error(err, None))
    }

    pub async fn get_block_by_number(
        &self,
        block_number: u64,
    ) -> Result<Option<BlockResponse>, RpcClientError> {
        let block_hex = format!("0x{:x}", block_number);
        let params = serde_json::json!([block_hex, false]);

        self.provider
            .client()
            .request::<_, Option<BlockResponse>>("eth_getBlockByNumber", params)
            .map_meta(set_request_id)
            .await
            .map_err(|err| Self::map_transport_error(err, None))
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    pub(crate) fn update_rpc_urls(&mut self, urls: Vec<Url>) {
        let provider = mk_read_provider(&urls).expect("failed to update provider");
        self.rpc_urls = urls;
        self.provider = Arc::new(provider);
    }

    fn map_transport_error(err: TransportError, context: Option<&str>) -> RpcClientError {
        match err {
            TransportError::ErrorResp(resp) => {
                let message = if let Some(ctx) = context {
                    format!("{}: {}", ctx, resp)
                } else {
                    resp.to_string()
                };
                RpcClientError::RpcError { message }
            }
            TransportError::NullResp => RpcClientError::MissingField {
                field: "result".to_string(),
            },
            TransportError::DeserError { err, text } => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    if let Some(error_value) = value.get("error") {
                        return RpcClientError::RpcError {
                            message: error_value.to_string(),
                        };
                    }
                }
                RpcClientError::JsonSerialization(err)
            }
            other => RpcClientError::Transport(other),
        }
    }
}

fn set_request_id(mut meta: RequestMeta) -> RequestMeta {
    meta.id = Id::from(1_u64);
    meta
}

#[derive(Debug, Error)]
pub enum RpcClientError {
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

    #[error("Configuration error: {message}")]
    Config { message: String },
}

impl From<TransportError> for RpcClientError {
    fn from(err: TransportError) -> Self {
        RpcClient::map_transport_error(err, None)
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
                .json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "eth_blockNumber",
                    "params": []
                }));
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"jsonrpc": "2.0", "id": 1, "result": "0x1b4"}"#);
        });

        let mut client =
            RpcClient::new_with_urls(vec![Url::parse(&server.base_url()).unwrap()]).unwrap();
        let result = client.get_latest_block_number().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 436);

        mock.assert();

        // Update URLs to ensure debug path works.
        client.update_rpc_urls(vec![Url::parse(&server.base_url()).unwrap()]);
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

        let client =
            RpcClient::new_with_urls(vec![Url::parse(&server.base_url()).unwrap()]).unwrap();
        let result = client.get_latest_block_number().await;
        assert!(matches!(
            result.unwrap_err(),
            RpcClientError::RpcError { .. }
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_block_by_number_ok() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "eth_getBlockByNumber",
                    "params": ["0x64", false]
                }));
            then.status(200)
                .header("content-type", "application/json")
                .body(sample_block_response("0x64", "0x64b8c123"));
        });

        let client =
            RpcClient::new_with_urls(vec![Url::parse(&server.base_url()).unwrap()]).unwrap();
        let response = client.get_block_by_number(100).await.unwrap();
        assert!(response.is_some());
        assert_eq!(response.unwrap().timestamp, "0x64b8c123");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_logs_ok() {
        let server = MockServer::start();
        let log_entry = sample_log_entry("0x64");
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "eth_getLogs",
                    "params": [{
                        "fromBlock": "0x1",
                        "toBlock": "0x2",
                        "address": "0x123",
                        "topics": [["0xabc"]]
                    }]
                }));
            then.status(200)
                .header("content-type", "application/json")
                .body(logs_response_body(json!([log_entry])));
        });

        let client =
            RpcClient::new_with_urls(vec![Url::parse(&server.base_url()).unwrap()]).unwrap();
        let logs = client
            .get_logs(
                "0x1",
                "0x2",
                "0x123",
                Some(vec![Some(vec!["0xabc".to_string()])]),
            )
            .await
            .unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].block_number, "0x64");

        mock.assert();
    }
}
