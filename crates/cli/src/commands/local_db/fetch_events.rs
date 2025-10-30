use alloy::primitives::Address;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::local_db::{FetchConfig, LocalDb, LocalDbError};
use rain_orderbook_common::rpc_client::{LogEntryResponse, RpcClientError};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[async_trait::async_trait]
pub trait EventClient {
    async fn get_latest_block_number(&self) -> Result<u64, RpcClientError>;
    async fn fetch(
        &self,
        address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError>;
}

#[async_trait::async_trait]
impl EventClient for LocalDb {
    async fn get_latest_block_number(&self) -> Result<u64, RpcClientError> {
        self.rpc_client().get_latest_block_number().await
    }

    async fn fetch(
        &self,
        address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        self.fetch_orderbook_events(
            Address::from_str(address)?,
            start_block,
            end_block,
            &FetchConfig::default(),
        )
        .await
    }
}

#[derive(Debug, Clone, Parser)]
#[command(about = "Fetch events from blockchain and save to JSON file")]
pub struct FetchEvents {
    #[clap(long)]
    pub api_token: String,
    #[clap(long)]
    pub chain_id: u32,
    #[clap(long)]
    pub start_block: u64,
    #[clap(long)]
    pub end_block: Option<u64>,
    #[clap(long)]
    pub orderbook_address: String,
    #[clap(long)]
    pub output_file: Option<String>,
}

impl FetchEvents {
    pub async fn execute_with_client<C: EventClient>(self, client: C) -> Result<()> {
        println!("Starting event fetch...");

        let end_block = if let Some(end_block) = self.end_block {
            end_block
        } else {
            client
                .get_latest_block_number()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get latest block number: {}", e))?
        };

        let all_events = client
            .fetch(&self.orderbook_address, self.start_block, end_block)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch events: {}", e))?;

        let output_path = self
            .output_file
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::default_output_path(end_block));
        let mut output_handle = File::create(&output_path)?;
        output_handle.write_all(serde_json::to_string_pretty(&all_events)?.as_bytes())?;

        println!("Events and results saved to: {}", output_path.display());
        Ok(())
    }

    pub async fn execute(self) -> Result<()> {
        let local_db = LocalDb::new_with_hyper_rpc(self.chain_id, self.api_token.clone())?;
        self.execute_with_client(local_db).await
    }

    fn default_output_path(end_block: u64) -> PathBuf {
        let filename = format!("events_{}.json", end_block);
        if let Ok(dir) = std::env::var("RAIN_ORDERBOOK_EVENTS_DIR") {
            Path::new(&dir).join(filename)
        } else {
            Path::new("src/commands/local_db").join(filename)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    fn sample_event(block_number: &str) -> LogEntryResponse {
        LogEntryResponse {
            address: "0x123".to_string(),
            topics: vec!["0xabc".to_string()],
            data: "0xdeadbeef".to_string(),
            block_number: block_number.to_string(),
            block_timestamp: Some("0x0".to_string()),
            transaction_hash: "0xtransaction".to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0xblock".to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        }
    }

    struct EnvVarGuard {
        key: String,
    }

    impl EnvVarGuard {
        fn set<K: Into<String>, V: AsRef<str>>(key: K, value: V) -> Self {
            let key = key.into();
            std::env::set_var(&key, value.as_ref());
            Self { key }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            std::env::remove_var(&self.key);
        }
    }

    struct MockEventClient {
        latest_block: Option<u64>,
        latest_block_error: Option<String>,
        events: Option<Vec<LogEntryResponse>>,
        events_error: Option<String>,
    }

    impl MockEventClient {
        fn new() -> Self {
            Self {
                latest_block: None,
                latest_block_error: None,
                events: None,
                events_error: None,
            }
        }

        fn with_latest_block(mut self, block: u64) -> Self {
            self.latest_block = Some(block);
            self
        }

        fn with_latest_block_error(mut self, error: String) -> Self {
            self.latest_block_error = Some(error);
            self
        }

        fn with_events(mut self, events: Vec<LogEntryResponse>) -> Self {
            self.events = Some(events);
            self
        }

        fn with_events_error(mut self, error: String) -> Self {
            self.events_error = Some(error);
            self
        }
    }

    #[async_trait::async_trait]
    impl EventClient for MockEventClient {
        async fn get_latest_block_number(&self) -> Result<u64, RpcClientError> {
            if let Some(error) = &self.latest_block_error {
                Err(RpcClientError::RpcError {
                    message: error.clone(),
                })
            } else {
                Ok(self.latest_block.unwrap_or(1000))
            }
        }

        async fn fetch(
            &self,
            _address: &str,
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
            if let Some(error) = &self.events_error {
                Err(LocalDbError::Config {
                    message: error.clone(),
                })
            } else {
                Ok(self.events.clone().unwrap_or_default())
            }
        }
    }

    #[tokio::test]
    async fn test_execute_with_client_success_with_explicit_end_block() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let fetch_events = FetchEvents {
            api_token: "test_token".to_string(),
            chain_id: 1,
            start_block: 100,
            end_block: Some(200),
            orderbook_address: "0x123".to_string(),
            output_file: Some(temp_path.clone()),
        };

        let mock_client = MockEventClient::new().with_events(vec![sample_event("0x64")]);

        let result = fetch_events.execute_with_client(mock_client).await;
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&temp_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed[0]["blockNumber"], "0x64");
    }

    #[tokio::test]
    async fn test_execute_with_client_success_with_latest_block() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let fetch_events = FetchEvents {
            api_token: "test_token".to_string(),
            chain_id: 1,
            start_block: 100,
            end_block: None,
            orderbook_address: "0x123".to_string(),
            output_file: Some(temp_path.clone()),
        };

        let mock_client = MockEventClient::new()
            .with_latest_block(500)
            .with_events(vec![sample_event("0x1f4")]);

        let result = fetch_events.execute_with_client(mock_client).await;
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&temp_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed[0]["blockNumber"], "0x1f4");
    }

    #[tokio::test]
    async fn test_execute_with_client_latest_block_error() {
        let fetch_events = FetchEvents {
            api_token: "test_token".to_string(),
            chain_id: 1,
            start_block: 100,
            end_block: None,
            orderbook_address: "0x123".to_string(),
            output_file: Some("test_output.json".to_string()),
        };

        let mock_client =
            MockEventClient::new().with_latest_block_error("RPC connection failed".to_string());

        let result = fetch_events.execute_with_client(mock_client).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to get latest block number"));
    }

    #[tokio::test]
    async fn test_execute_with_client_fetch_events_error() {
        let fetch_events = FetchEvents {
            api_token: "test_token".to_string(),
            chain_id: 1,
            start_block: 100,
            end_block: Some(200),
            orderbook_address: "0x123".to_string(),
            output_file: Some("test_output.json".to_string()),
        };

        let mock_client =
            MockEventClient::new().with_events_error("Network connection failed".to_string());

        let result = fetch_events.execute_with_client(mock_client).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to fetch events"));
    }

    #[tokio::test]
    async fn test_execute_with_client_default_output_filename() {
        let temp_dir = TempDir::new().unwrap();
        let _env_guard = EnvVarGuard::set(
            "RAIN_ORDERBOOK_EVENTS_DIR",
            temp_dir.path().to_str().unwrap(),
        );

        let fetch_events = FetchEvents {
            api_token: "test_token".to_string(),
            chain_id: 1,
            start_block: 100,
            end_block: Some(200),
            orderbook_address: "0x123".to_string(),
            output_file: None,
        };

        let mock_client = MockEventClient::new().with_events(vec![]);

        let result = fetch_events.execute_with_client(mock_client).await;
        assert!(result.is_ok());

        let expected_filename = temp_dir.path().join("events_200.json");
        assert!(expected_filename.exists());
    }
}
