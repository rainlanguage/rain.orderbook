use async_trait::async_trait;
use url::Url;

use alloy::primitives::Address;

use crate::local_db::decode::{DecodedEvent, DecodedEventData};
use crate::local_db::{FetchConfig, LocalDb, LocalDbError};
use crate::rpc_client::{BlockRange, LogEntryResponse};

use crate::local_db::pipeline::EventsPipeline;

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

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use crate::rpc_client::RpcClientError;

    use super::*;
    use alloy::{
        hex,
        primitives::{Address, Bytes, FixedBytes},
        sol_types::SolEvent,
    };
    use rain_orderbook_bindings::OrderBook::MetaV1_2;

    fn test_url() -> Url {
        Url::parse("http://localhost:8545").expect("valid test url")
    }

    fn make_meta_log() -> LogEntryResponse {
        // Construct a minimal, valid MetaV1_2 event and encode it to bytes.
        let sender = Address::from([24u8; 20]);
        let subject_addr = Address::from([25u8; 20]);
        let mut subject_bytes = [0u8; 32];
        subject_bytes[12..32].copy_from_slice(&subject_addr[..]);
        let event = MetaV1_2 {
            sender,
            subject: FixedBytes::<32>::from(subject_bytes),
            meta: Bytes::from(vec![0x09, 0x0a, 0x0b, 0x0c, 0x0d]),
        };

        let encoded = event.encode_data();

        LogEntryResponse {
            address: format!("0x{:040x}", 0),
            topics: vec![format!("0x{}", hex::encode(MetaV1_2::SIGNATURE_HASH))],
            data: format!("0x{}", hex::encode(encoded)),
            block_number: "0x1".to_string(),
            block_timestamp: Some("0x2".to_string()),
            transaction_hash: "0xaabbccddeeff00112233445566778899aabbccddeeff00112233445566778899"
                .to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0xbbccddeeff00112233445566778899aabbccddeeff00112233445566778899aa"
                .to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        }
    }

    #[test]
    fn constructors_build_ok() {
        // with_regular_rpcs
        let pipe = DefaultEventsPipeline::with_regular_rpcs(vec![test_url()])
            .expect("build with regular rpcs");
        // from_local_db
        let db = LocalDb::new_with_url(test_url());
        let _pipe2 = DefaultEventsPipeline::from_local_db(db);

        // with_hyperrpc (uses supported chain id; token string is arbitrary)
        let _pipe3 = DefaultEventsPipeline::with_hyperrpc(42161, "token".to_string())
            .expect("build with hyperrpc");
        drop(pipe);
    }

    #[test]
    fn decode_delegates_to_localdb() {
        let pipe = DefaultEventsPipeline::from_local_db(LocalDb::new_with_url(test_url()));

        let log = make_meta_log();
        let via_db = LocalDb::new_with_url(test_url())
            .decode_events(&[log.clone()])
            .expect("db decode ok");
        let via_pipe = pipe.decode(&[log]).expect("pipeline decode ok");

        assert_eq!(via_db.len(), 1);
        assert_eq!(via_pipe.len(), 1);
        assert_eq!(via_db[0].event_type, via_pipe[0].event_type);

        // Spot‑check decoded variant and fields are as expected
        assert!(matches!(
            via_pipe[0].decoded_data,
            DecodedEvent::MetaV1_2(_)
        ));
        if let DecodedEvent::MetaV1_2(ev) = &via_pipe[0].decoded_data {
            assert_eq!(ev.sender, Address::from([24u8; 20]));
            // Subject is the 20‑byte address left‑padded to 32 bytes
            let mut expected = [0u8; 32];
            expected[12..32].copy_from_slice(&Address::from([25u8; 20])[..]);
            assert_eq!(ev.subject, FixedBytes::<32>::from(expected));
            assert_eq!(ev.meta.as_ref(), &[0x09, 0x0a, 0x0b, 0x0c, 0x0d]);
        }
    }

    #[test]
    fn decode_propagates_decode_errors() {
        let db = LocalDb::new_with_url(test_url());
        let pipe = DefaultEventsPipeline::from_local_db(db);

        // Valid topic but empty data triggers a decode error path.
        let bad_log = LogEntryResponse {
            address: format!("0x{:040x}", 0),
            topics: vec![format!("0x{}", hex::encode(MetaV1_2::SIGNATURE_HASH))],
            data: "0x".to_string(),
            block_number: "0x1".to_string(),
            block_timestamp: Some("0x2".to_string()),
            transaction_hash: "0xaabbccddeeff00112233445566778899aabbccddeeff00112233445566778899"
                .to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0xbbccddeeff00112233445566778899aabbccddeeff00112233445566778899aa"
                .to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        };

        let err = pipe.decode(&[bad_log]).expect_err("expected decode error");
        match err {
            LocalDbError::DecodeError { .. } => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn constructor_error_paths() {
        // Empty RPC list should error via RpcClient config mapping
        let err = DefaultEventsPipeline::with_regular_rpcs(vec![]).expect_err("expected error");
        match err {
            LocalDbError::Rpc(RpcClientError::Config { .. }) => {}
            other => panic!("unexpected error variant: {other:?}"),
        }

        // Unsupported chain id surfaces as Rpc -> UnsupportedChainId
        let err = DefaultEventsPipeline::with_hyperrpc(9999, "token".to_string())
            .expect_err("expected unsupported chain id error");
        match err {
            LocalDbError::Rpc(RpcClientError::UnsupportedChainId { chain_id }) => {
                assert_eq!(chain_id, 9999);
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
