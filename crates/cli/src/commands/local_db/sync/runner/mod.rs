use anyhow::Result;
use url::Url;

use super::super::sqlite::sqlite_execute;
use super::{
    data_source::{SyncDataSource, TokenMetadataFetcher},
    storage::ensure_schema,
};

use self::{
    apply::{decode_events, fetch_events, prepare_sql},
    window::compute_sync_window,
};

mod apply;
mod window;

pub(crate) struct SyncRunner<'a, D, T> {
    db_path: &'a str,
    data_source: &'a D,
    metadata_rpc_urls: Vec<Url>,
    token_fetcher: &'a T,
}

pub(crate) struct SyncParams<'a> {
    pub(crate) chain_id: u32,
    pub(crate) orderbook_address: &'a str,
    pub(crate) deployment_block: u64,
    pub(crate) start_block: Option<u64>,
    pub(crate) end_block: Option<u64>,
}

impl<'a, D, T> SyncRunner<'a, D, T>
where
    D: SyncDataSource + Send + Sync,
    T: TokenMetadataFetcher + Send + Sync,
{
    pub(crate) fn new(
        db_path: &'a str,
        data_source: &'a D,
        metadata_rpc_urls: Vec<Url>,
        token_fetcher: &'a T,
    ) -> Self {
        Self {
            db_path,
            data_source,
            metadata_rpc_urls,
            token_fetcher,
        }
    }

    pub(crate) async fn run(&self, params: &SyncParams<'_>) -> Result<()> {
        let schema_applied = ensure_schema(self.db_path)?;
        if schema_applied {
            println!("Database schema initialized at {}", self.db_path);
        }

        let window = compute_sync_window(self.db_path, self.data_source, params).await?;
        println!("Current last_synced_block: {}", window.last_synced_block);
        if let Some(adjustment) = &window.start_adjustment {
            println!("{}", adjustment.message(window.last_synced_block));
        }
        println!("Network latest block: {}", window.latest_block);
        if let Some(clamp) = &window.end_clamp {
            println!("{}", clamp.message());
        }
        if window.noop {
            println!(
                "Nothing to do (start block {} exceeds target block {})",
                window.start_block, window.target_block
            );
            return Ok(());
        }

        println!(
            "Fetching events for {} from block {} to {}",
            params.orderbook_address, window.start_block, window.target_block
        );
        let fetch = fetch_events(
            self.data_source,
            params.orderbook_address,
            window.start_block,
            window.target_block,
        )
        .await?;
        println!("Fetched {} raw events", fetch.raw_count);

        println!("Decoding events");
        let decoded = decode_events(self.data_source, fetch.events)?;
        println!("Decoded {} events", decoded.decoded_count);

        println!("Preparing token metadata");
        let sql = prepare_sql(
            self.data_source,
            self.token_fetcher,
            self.db_path,
            self.metadata_rpcs(),
            params.chain_id,
            &decoded.decoded,
            window.target_block,
        )
        .await?;

        println!("Generating SQL for {} events", decoded.decoded_count);
        println!("Applying SQL to {}", self.db_path);
        sqlite_execute(self.db_path, &sql)?;

        println!(
            "Sync complete. last_synced_block is now {}",
            window.target_block
        );
        Ok(())
    }

    fn metadata_rpcs(&self) -> &[Url] {
        if self.metadata_rpc_urls.is_empty() {
            self.data_source.rpc_urls()
        } else {
            &self.metadata_rpc_urls
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use async_trait::async_trait;
    use rain_orderbook_common::erc20::TokenInfo;
    use rain_orderbook_common::raindex_client::local_db::decode::{DecodedEvent, DecodedEventData};
    use rain_orderbook_common::rpc_client::LogEntryResponse;
    use url::Url;

    struct DummyDataSource {
        rpc_urls: Vec<Url>,
    }

    #[async_trait]
    impl SyncDataSource for DummyDataSource {
        async fn latest_block(&self) -> Result<u64> {
            Ok(0)
        }

        async fn fetch_events(
            &self,
            _orderbook_address: &str,
            _start_block: u64,
            _end_block: u64,
        ) -> Result<Vec<LogEntryResponse>> {
            Ok(vec![])
        }

        fn decode_events(
            &self,
            _events: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>> {
            Ok(vec![])
        }

        fn events_to_sql(
            &self,
            _decoded_events: &[DecodedEventData<DecodedEvent>],
            _end_block: u64,
            _decimals_by_token: &std::collections::HashMap<Address, u8>,
            _prefix_sql: &str,
        ) -> Result<String> {
            Ok(String::new())
        }

        fn rpc_urls(&self) -> &[Url] {
            &self.rpc_urls
        }
    }

    struct DummyFetcher;

    #[async_trait]
    impl TokenMetadataFetcher for DummyFetcher {
        async fn fetch(
            &self,
            _rpcs: &[Url],
            _missing: Vec<Address>,
        ) -> Result<Vec<(Address, TokenInfo)>> {
            Ok(vec![])
        }
    }

    #[test]
    fn metadata_rpcs_prefers_override_urls() {
        let source = DummyDataSource {
            rpc_urls: vec![Url::parse("https://source.example").unwrap()],
        };
        let override_url = Url::parse("https://override.example").unwrap();
        let runner = SyncRunner::new(
            "db.sqlite",
            &source,
            vec![override_url.clone()],
            &DummyFetcher,
        );

        let urls = runner.metadata_rpcs();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], override_url);
    }

    #[test]
    fn metadata_rpcs_falls_back_to_source_urls() {
        let source = DummyDataSource {
            rpc_urls: vec![Url::parse("https://fallback.example").unwrap()],
        };
        let runner = SyncRunner::new("db.sqlite", &source, Vec::new(), &DummyFetcher);

        assert_eq!(runner.metadata_rpcs(), source.rpc_urls());
    }
}
