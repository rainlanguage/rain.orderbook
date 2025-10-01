use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rain_orderbook_common::erc20::TokenInfo;
use rain_orderbook_common::raindex_client::local_db::token_fetch::fetch_erc20_metadata_concurrent;
use rain_orderbook_common::raindex_client::local_db::LocalDb;
use serde_json::Value;
use url::Url;

#[async_trait]
pub(crate) trait SyncDataSource {
    async fn latest_block(&self) -> Result<u64>;
    async fn fetch_events(
        &self,
        orderbook_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Value>;
    fn decode_events(&self, events: Value) -> Result<Value>;
    fn events_to_sql(
        &self,
        decoded_events: Value,
        end_block: u64,
        prefix_sql: &str,
    ) -> Result<String>;
    fn rpc_urls(&self) -> &[Url];
}

#[async_trait]
pub(crate) trait TokenMetadataFetcher {
    async fn fetch(&self, rpcs: &[Url], missing: Vec<Address>)
        -> Result<Vec<(Address, TokenInfo)>>;
}

pub(crate) struct DefaultTokenFetcher;

#[async_trait]
impl TokenMetadataFetcher for DefaultTokenFetcher {
    async fn fetch(
        &self,
        rpcs: &[Url],
        missing: Vec<Address>,
    ) -> Result<Vec<(Address, TokenInfo)>> {
        if missing.is_empty() {
            return Ok(vec![]);
        }

        let fetched = fetch_erc20_metadata_concurrent(rpcs.to_vec(), missing)
            .await
            .map_err(|e| anyhow!(e))?;
        Ok(fetched)
    }
}

#[async_trait]
impl SyncDataSource for LocalDb {
    async fn latest_block(&self) -> Result<u64> {
        self.rpc_client()
            .get_latest_block_number(self.rpc_urls())
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn fetch_events(
        &self,
        orderbook_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Value> {
        self.fetch_events(orderbook_address, start_block, end_block)
            .await
            .map_err(|e| anyhow!(e))
    }

    fn decode_events(&self, events: Value) -> Result<Value> {
        self.decode_events(events).map_err(|e| anyhow!(e))
    }

    fn events_to_sql(
        &self,
        decoded_events: Value,
        end_block: u64,
        prefix_sql: &str,
    ) -> Result<String> {
        self.decoded_events_to_sql_with_prefix(decoded_events, end_block, prefix_sql)
            .map_err(|e| anyhow!("Failed to generate SQL: {}", e))
    }

    fn rpc_urls(&self) -> &[Url] {
        self.rpc_urls()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_fetcher_short_circuits_on_empty_missing_set() {
        let fetcher = DefaultTokenFetcher;
        let result = fetcher
            .fetch(&[], Vec::new())
            .await
            .expect("empty fetch should succeed");
        assert!(result.is_empty());
    }
}
