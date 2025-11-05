use crate::local_db::pipeline::adapters::{
    apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
    window::DefaultWindowPipeline,
};
use crate::local_db::pipeline::runner::environment::{
    default_dump_downloader, default_manifest_fetcher, EnginePipelines, RunnerEnvironment,
};
use crate::local_db::pipeline::runner::utils::RunnerTarget;
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;
use std::sync::Arc;

pub fn default_environment() -> RunnerEnvironment<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ClientStatusBus,
> {
    RunnerEnvironment::new(
        default_manifest_fetcher(),
        default_dump_downloader(),
        Arc::new(|target: &RunnerTarget| {
            let events =
                DefaultEventsPipeline::with_regular_rpcs(target.inputs.metadata_rpcs.clone())?;

            Ok(EnginePipelines::new(
                ClientBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                DefaultTokensPipeline::new(),
                DefaultApplyPipeline::new(),
                ClientStatusBus::new(),
            ))
        }),
    )
}
