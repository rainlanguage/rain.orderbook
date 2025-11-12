use alloy::primitives::Address;
use async_trait::async_trait;
use url::Url;

use crate::local_db::decode::{decode_events, DecodedEvent, DecodedEventData};
use crate::local_db::fetch::{fetch_orderbook_events, fetch_store_events};
use crate::local_db::pipeline::EventsPipeline;
use crate::local_db::{FetchConfig, LocalDbError};
use crate::rpc_client::{LogEntryResponse, RpcClient};

/// Shared implementation of the EventsPipeline that delegates to LocalDb.
///
/// Construction determines the backend policy:
/// - `with_regular_rpcs` for browser/public RPCs
/// - `with_hyperrpc` for producer/HyperRPC
/// - `from_local_db` if the runner builds a LocalDb externally
#[derive(Debug, Clone)]
pub struct DefaultEventsPipeline {
    rpc_client: RpcClient,
}

impl DefaultEventsPipeline {
    /// Constructs the pipeline using regular/public RPC URLs.
    pub fn with_regular_rpcs(rpcs: Vec<Url>) -> Result<Self, LocalDbError> {
        let rpc_client = RpcClient::new_with_urls(rpcs)?;
        Ok(Self { rpc_client })
    }

    /// Constructs the pipeline using HyperRPC (producer path).
    pub fn with_hyperrpc(chain_id: u32, api_token: String) -> Result<Self, LocalDbError> {
        let rpc_client = RpcClient::new_with_hyper_rpc(chain_id, &api_token)?;
        Ok(Self { rpc_client })
    }
}

#[async_trait(?Send)]
impl EventsPipeline for DefaultEventsPipeline {
    async fn latest_block(&self) -> Result<u64, LocalDbError> {
        self.rpc_client
            .get_latest_block_number()
            .await
            .map_err(Into::into)
    }

    async fn fetch_orderbook(
        &self,
        orderbook_address: Address,
        from_block: u64,
        to_block: u64,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        fetch_orderbook_events(
            &self.rpc_client,
            orderbook_address,
            from_block,
            to_block,
            cfg,
        )
        .await
    }

    async fn fetch_stores(
        &self,
        store_addresses: &[Address],
        from_block: u64,
        to_block: u64,
        cfg: &FetchConfig,
    ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
        fetch_store_events(&self.rpc_client, store_addresses, from_block, to_block, cfg).await
    }

    fn decode(
        &self,
        logs: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
        decode_events(logs).map_err(LocalDbError::DecodeError)
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::rpc_client::RpcClientError;
    use alloy::{hex, primitives::U256, sol_types::SolEvent};
    use rain_orderbook_bindings::OrderBook::MetaV1_2;

    fn test_url() -> Url {
        Url::parse("http://localhost:8545").expect("valid test url")
    }

    #[test]
    fn constructors_build_ok() {
        // with_regular_rpcs
        let pipe = DefaultEventsPipeline::with_regular_rpcs(vec![test_url()])
            .expect("build with regular rpcs");

        // with_hyperrpc (uses supported chain id; token string is arbitrary)
        let _pipe3 = DefaultEventsPipeline::with_hyperrpc(42161, "token".to_string())
            .expect("build with hyperrpc");
        drop(pipe);
    }

    #[test]
    fn decode_propagates_decode_errors() {
        let pipe = DefaultEventsPipeline::with_regular_rpcs(vec![test_url()]).unwrap();

        // Valid topic but empty data triggers a decode error path.
        let bad_log = LogEntryResponse {
            address: format!("0x{:040x}", 0),
            topics: vec![format!("0x{}", hex::encode(MetaV1_2::SIGNATURE_HASH))],
            data: "0x".to_string(),
            block_number: U256::from(1),
            block_timestamp: Some(U256::from(2)),
            transaction_hash: "0xaabbccddeeff00112233445566778899aabbccddeeff00112233445566778899"
                .to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0xbbccddeeff00112233445566778899aabbccddeeff00112233445566778899aa"
                .to_string(),
            log_index: U256::ZERO,
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
