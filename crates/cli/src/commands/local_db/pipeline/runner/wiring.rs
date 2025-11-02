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
