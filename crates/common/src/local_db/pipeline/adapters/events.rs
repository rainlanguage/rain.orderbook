use async_trait::async_trait;
use url::Url;

use alloy::primitives::Address;

use crate::local_db::decode::{DecodedEvent, DecodedEventData};
use crate::local_db::{FetchConfig, LocalDb, LocalDbError};
use crate::rpc_client::{BlockRange, LogEntryResponse};

use crate::local_db::pipeline::traits::EventsPipeline;

/// Shared implementation of the EventsPipeline that delegates to LocalDb.
///
/// Construction determines the backend policy:
/// - `with_regular_rpcs` for browser/public RPCs
/// - `with_hyperrpc` for producer/HyperRPC
/// - `from_local_db` if the runner builds a LocalDb externally
#[derive(Debug, Clone)]
pub struct DefaultEventsPipeline {
    db: LocalDb,
}

impl DefaultEventsPipeline {
    /// Constructs the pipeline using regular/public RPC URLs.
    pub fn with_regular_rpcs(rpcs: Vec<Url>) -> Result<Self, LocalDbError> {
        let db = LocalDb::new_with_regular_rpcs(rpcs)?;
        Ok(Self { db })
    }

    /// Constructs the pipeline using HyperRPC (producer path).
    pub fn with_hyperrpc(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let db = LocalDb::new_with_hyper_rpc(chain_id, api_token)?;
        Ok(Self { db })
    }

    /// Wraps an existing LocalDb instance provided by the runner.
    pub fn from_local_db(db: LocalDb) -> Self {
        Self { db }
    }
}

#[async_trait(?Send)]
impl EventsPipeline for DefaultEventsPipeline {
    async fn latest_block(&self) -> Result<u64, LocalDbError> {
        self.db
            .rpc_client()
            .get_latest_block_number()
            .await
            .map_err(Into::into)
    }

    async fn fetch_orderbook(
        &self,
        orderbook_address: Address,
        range: BlockRange,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        self.db
            .fetch_orderbook_events(orderbook_address, range, cfg)
            .await
    }

    async fn fetch_stores(
        &self,
        store_addresses: &[Address],
        range: BlockRange,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        self.db
            .fetch_store_events(store_addresses, range, cfg)
            .await
    }

    fn decode(
        &self,
        logs: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
        self.db.decode_events(logs)
    }
}
