use alloy::primitives::Address;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rain_orderbook_common::{
    erc20::TokenInfo,
    local_db::{
        decode::{DecodedEvent, DecodedEventData},
        query::SqlStatementBatch,
        token_fetch::fetch_erc20_metadata_concurrent,
        FetchConfig, LocalDb,
    },
    rpc_client::{BlockRange, LogEntryResponse},
};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

#[async_trait]
pub(crate) trait SyncDataSource {
    async fn latest_block(&self) -> Result<u64>;
    async fn fetch_events(
        &self,
        orderbook_address: &str,
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
        chain_id: u32,
        orderbook_address: Address,
        decoded_events: &[DecodedEventData<DecodedEvent>],
        decimals_by_token: &HashMap<Address, u8>,
    ) -> Result<SqlStatementBatch>;
    fn raw_events_to_statements(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        raw_events: &[LogEntryResponse],
    ) -> Result<SqlStatementBatch>;
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

        let fetched =
            fetch_erc20_metadata_concurrent(rpcs.to_vec(), missing, &FetchConfig::default())
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
        orderbook_address: &str,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>> {
        <LocalDb>::fetch_orderbook_events(
            self,
            Address::from_str(orderbook_address)?,
            BlockRange::inclusive(start_block, end_block)?,
            &FetchConfig::default(),
        )
        .await
        .map_err(|e| anyhow!(e))
    }

    async fn fetch_store_set_events(
        &self,
        store_addresses: &[Address],
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<LogEntryResponse>> {
        let addresses: Vec<Address> = store_addresses.to_vec();
        <LocalDb>::fetch_store_events(
            self,
            &addresses,
            BlockRange::inclusive(start_block, end_block)?,
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
        chain_id: u32,
        orderbook_address: Address,
        decoded_events: &[DecodedEventData<DecodedEvent>],
        decimals_by_token: &HashMap<Address, u8>,
    ) -> Result<SqlStatementBatch> {
        <LocalDb>::decoded_events_to_statements(
            self,
            chain_id,
            orderbook_address,
            decoded_events,
            decimals_by_token,
        )
        .map_err(|e| anyhow!("Failed to generate SQL: {}", e))
    }

    fn raw_events_to_statements(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        raw_events: &[LogEntryResponse],
    ) -> Result<SqlStatementBatch> {
        <LocalDb>::raw_events_to_statements(self, chain_id, orderbook_address, raw_events)
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
