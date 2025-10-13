use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rain_orderbook_common::{
    erc20::TokenInfo,
    raindex_client::local_db::{
        decode::{DecodedEvent, DecodedEventData},
        token_fetch::fetch_erc20_metadata_concurrent,
        FetchConfig, LocalDb,
    },
    rpc_client::LogEntryResponse,
};
use std::collections::HashMap;
use url::Url;

#[async_trait]
pub(crate) trait SyncDataSource {
    async fn latest_block(&self) -> Result<u64>;
    async fn fetch_events(
        &self,
        orderbook_address: Address,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>>;
    async fn fetch_store_set_events(
        &self,
        store_addresses: &[Address],
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>>;
    fn decode_events(
        &self,
        events: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>>;
    fn events_to_sql(
        &self,
        decoded_events: &[DecodedEventData<DecodedEvent>],
        end_block: u64,
        decimals_by_token: &HashMap<Address, u8>,
        prefix_sql: &str,
    ) -> Result<String>;
    fn raw_events_to_sql(&self, raw_events: &[LogEntryResponse]) -> Result<String>;
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
            .get_latest_block_number()
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn fetch_events(
        &self,
        orderbook_address: Address,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>> {
        <LocalDb>::fetch_events(self, orderbook_address, start_block, end_block)
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn fetch_store_set_events(
        &self,
        store_addresses: &[Address],
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>> {
        let parsed_addresses: Vec<Address> = store_addresses.to_vec();
        <LocalDb>::fetch_store_set_events(
            self,
            parsed_addresses.as_slice(),
            start_block,
            end_block,
            &FetchConfig::default(),
        )
        .await
        .map_err(|e| anyhow!(e))
    }

    fn decode_events(
        &self,
        events: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
        <LocalDb>::decode_events(self, events).map_err(|e| anyhow!(e))
    }

    fn events_to_sql(
        &self,
        decoded_events: &[DecodedEventData<DecodedEvent>],
        end_block: u64,
        decimals_by_token: &HashMap<Address, u8>,
        prefix_sql: &str,
    ) -> Result<String> {
        let prefix = if prefix_sql.is_empty() {
            None
        } else {
            Some(prefix_sql)
        };

        <LocalDb>::decoded_events_to_sql(self, decoded_events, end_block, decimals_by_token, prefix)
            .map_err(|e| anyhow!("Failed to generate SQL: {}", e))
    }

    fn raw_events_to_sql(&self, raw_events: &[LogEntryResponse]) -> Result<String> {
        <LocalDb>::raw_events_to_sql(self, raw_events)
            .map_err(|e| anyhow!("Failed to generate raw events SQL: {}", e))
    }

    fn rpc_urls(&self) -> &[Url] {
        self.rpc_client().rpc_urls()
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
