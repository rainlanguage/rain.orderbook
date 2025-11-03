use std::sync::Arc;

use rain_orderbook_common::local_db::pipeline::{
    adapters::{
        apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
        window::DefaultWindowPipeline,
    },
    runner::{
        default_dump_downloader, default_manifest_fetcher, EnginePipelines, RunnerEnvironment,
        RunnerTarget,
    },
};

use crate::commands::local_db::pipeline::{
    bootstrap::ProducerBootstrapAdapter, status::ProducerStatusBus,
};

pub fn default_environment(
    hypersync_token: String,
) -> RunnerEnvironment<
    ProducerBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ProducerStatusBus,
> {
    RunnerEnvironment::new(
        default_manifest_fetcher(),
        default_dump_downloader(),
        Arc::new(move |target: &RunnerTarget| {
            let events = DefaultEventsPipeline::with_hyperrpc(
                target.inputs.target.chain_id,
                hypersync_token.clone(),
            )?;

            Ok(EnginePipelines::new(
                ProducerBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                DefaultTokensPipeline::new(),
                DefaultApplyPipeline::new(),
                ProducerStatusBus::new(),
            ))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;
    use rain_orderbook_common::local_db::fetch::FetchConfig;
    use rain_orderbook_common::local_db::pipeline::engine::SyncInputs;
    use rain_orderbook_common::local_db::pipeline::runner::RunnerTarget;
    use rain_orderbook_common::local_db::pipeline::{
        FinalityConfig, SyncConfig, TargetKey, WindowOverrides,
    };
    use rain_orderbook_common::local_db::LocalDbError;
    use rain_orderbook_common::rpc_client::RpcClientError;
    use url::Url;

    fn sample_target(chain_id: u32) -> RunnerTarget {
        let fetch = FetchConfig::new(1, 1, 1, 1).expect("fetch config");
        RunnerTarget {
            orderbook_key: "test-book".to_string(),
            manifest_url: Url::parse("https://manifests.example/default.yaml").unwrap(),
            network_key: "anvil".to_string(),
            inputs: SyncInputs {
                target: TargetKey {
                    chain_id,
                    orderbook_address: address!("00000000000000000000000000000000000000a1"),
                },
                metadata_rpcs: vec![Url::parse("https://rpc.example/anvil").unwrap()],
                cfg: SyncConfig {
                    deployment_block: 100,
                    fetch,
                    finality: FinalityConfig { depth: 12 },
                    window_overrides: WindowOverrides::default(),
                },
                dump_str: None,
            },
        }
    }

    #[test]
    fn build_engine_configures_hyperrpc_for_supported_chain() {
        let env = default_environment("super-secret-token".to_string());
        let target = sample_target(42161);
        let engine = env.build_engine(&target).expect("engine available");

        let events_debug = format!("{:?}", engine.events);
        assert!(
            events_debug.contains("chain_id: Some(42161)"),
            "expected HyperRPC client for chain 42161: {events_debug}"
        );
        assert!(
            events_debug.contains("https://arbitrum.rpc.hypersync.xyz"),
            "expected HyperRPC base URL in debug repr: {events_debug}"
        );
    }

    #[test]
    fn build_engine_rejects_unsupported_chain() {
        let env = default_environment("token".to_string());
        let target = sample_target(1);
        match env.build_engine(&target) {
            Err(LocalDbError::Rpc(RpcClientError::UnsupportedChainId { chain_id })) => {
                assert_eq!(chain_id, 1);
            }
            Err(other) => panic!("unexpected error variant: {other:?}"),
            Ok(_) => panic!("expected unsupported chain to fail"),
        }
    }
}
