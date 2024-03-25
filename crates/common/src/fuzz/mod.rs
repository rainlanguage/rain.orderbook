use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use alloy_primitives::U256;
use dotrain::{error::ComposeError, RainDocument, Rebind};
use futures::TryFutureExt;
use proptest::prelude::RngCore;
use proptest::test_runner::{RngAlgorithm, TestRng};
use rain_interpreter_bindings::IInterpreterStoreV1::FullyQualifiedNamespace;
use rain_interpreter_eval::fork::NewForkedEvm;
use rain_interpreter_eval::trace::TraceSearchError;
use rain_interpreter_eval::{
    error::ForkCallError, eval::ForkEvalArgs, fork::Forker, trace::RainEvalResult,
};
use rain_orderbook_app_settings::chart::Chart;
use rain_orderbook_app_settings::config::*;
use rain_orderbook_app_settings::scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use thiserror::Error;
use typeshare::typeshare;

#[derive(Debug)]
pub struct FuzzResult {
    pub scenario: String,
    pub runs: Vec<(RainEvalResult, String)>,
}

impl FuzzResult {
    pub fn collect_data_by_path(
        &self,
        path: &str,
    ) -> Result<Vec<(U256, String)>, TraceSearchError> {
        let mut collection: Vec<(U256, String)> = vec![];
        // loop over the runs and search_trace_by_path for each
        for (run, rainlang_string) in self.runs.iter() {
            let stack = run.search_trace_by_path(path)?;
            collection.push((stack, rainlang_string.clone()));
        }
        Ok(collection)
    }
}

pub struct FuzzRunner {
    pub forker: Forker,
    pub dotrain: String,
    pub rng: TestRng,
    pub settings: Config,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotDataItem {
    #[typeshare(serialized_as = "Vec<String>")]
    pub data: Vec<U256>,
    pub rainlang: String,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotData {
    pub name: String,
    pub plot_type: String,
    pub data: Vec<PlotDataItem>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartData {
    pub name: String,
    pub plots: Vec<PlotData>,
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error("Scenario has no runs defined")]
    ScenarioNoRuns,
    #[error("{0} is not a testable scenario")]
    ScenarioNotTestable(String),
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
    #[error("Empty Front Matter")]
    EmptyFrontmatter,
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error(transparent)]
    TraceSearchError(#[from] TraceSearchError),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
}

impl FuzzRunner {
    pub async fn new(dotrain: &str, settings: Config, seed: Option<[u8; 32]>) -> Self {
        Self {
            forker: Forker::new(),
            dotrain: dotrain.into(),
            settings,
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
        }
    }

    pub async fn run_scenario_by_name(
        &mut self,
        name: &str,
    ) -> Result<FuzzResult, FuzzRunnerError> {
        // find the scenario by name in the settings
        let scenario = self
            .settings
            .scenarios
            .get(name)
            .ok_or(FuzzRunnerError::ScenarioNotFound(name.into()))
            .cloned()?;

        self.run_scenario(&scenario).await
    }

    pub async fn run_scenario(
        &mut self,
        scenario: &Arc<Scenario>,
    ) -> Result<FuzzResult, FuzzRunnerError> {
        // if the scenario doesn't have runs, return an error
        let no_of_runs = scenario.runs.ok_or(FuzzRunnerError::ScenarioNoRuns)?;

        let deployer = scenario.deployer.clone();

        // create a fork
        self.forker
            .add_or_select(
                NewForkedEvm {
                    fork_url: deployer.network.rpc.clone().into(),
                    fork_block_number: None,
                },
                None,
            )
            .await?;

        // pull out the bindings fom the scenario
        let scenario_bindings: Vec<Rebind> = scenario
            .bindings
            .clone()
            .into_iter()
            .map(|(k, v)| Rebind(k, v))
            .collect();

        // create a new RainDocument with the dotrain and the bindings
        // the bindings in the dotrain string are ignored by the RainDocument
        let rain_document = RainDocument::create(
            self.dotrain.clone(),
            None,
            None,
            Some(scenario_bindings.clone()),
        );

        // search the name space hash map for NamespaceItems that are elided and make a vec of the keys
        let elided_binding_keys = Arc::new(
            rain_document
                .namespace()
                .iter()
                .filter(|(_, v)| v.is_elided_binding())
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>(),
        );

        let fork = Arc::new(self.forker.clone()); // Wrap in Arc for shared ownership
        let dotrain = Arc::new(self.dotrain.clone());
        let mut handles = vec![];
        let mut rainlangs = std::collections::VecDeque::new();

        for _ in 0..no_of_runs {
            let fork_clone = Arc::clone(&fork); // Clone the Arc for each thread
            let elided_binding_keys = Arc::clone(&elided_binding_keys);
            let deployer = Arc::clone(&deployer);
            let scenario_bindings = scenario_bindings.clone();
            let dotrain = Arc::clone(&dotrain);

            let mut final_bindings: Vec<Rebind> = vec![];

            // for each scenario.fuzz_binds, add a random value
            for elided_binding in elided_binding_keys.as_slice() {
                let mut val: [u8; 32] = [0; 32];
                self.rng.fill_bytes(&mut val);
                let hex = format!("0x{}", alloy_primitives::hex::encode(val));
                final_bindings.push(Rebind(elided_binding.to_string(), hex));
            }

            final_bindings.extend(scenario_bindings.clone());
            let rainlang_string = RainDocument::compose_text(
                &dotrain,
                &ORDERBOOK_ORDER_ENTRYPOINTS,
                None,
                Some(final_bindings),
            )?;
            rainlangs.push_back(rainlang_string.clone());

            let handle = tokio::spawn(async move {
                let args = ForkEvalArgs {
                    rainlang_string,
                    source_index: 0,
                    deployer: deployer.address,
                    namespace: FullyQualifiedNamespace::default(),
                    context: vec![],
                    decode_errors: false,
                };
                fork_clone
                    .fork_eval(args)
                    .map_err(FuzzRunnerError::ForkCallError)
                    .await
            });
            handles.push(handle);
        }

        let mut runs: Vec<(RainEvalResult, String)> = Vec::new();

        for handle in handles {
            let res = handle.await??;
            runs.push((res.into(), rainlangs.pop_front().unwrap_or("".to_owned())));
        }

        Ok(FuzzResult {
            scenario: scenario.name.clone(),
            runs,
        })
    }

    pub async fn build_chart_data(
        &mut self,
        name: String,
        chart: Chart,
    ) -> Result<ChartData, FuzzRunnerError> {
        let res = self.run_scenario(&chart.scenario).await?;

        let plot_data_results: Result<Vec<PlotData>, FuzzRunnerError> = chart
            .plots
            .into_iter()
            .map(|(name, plot)| {
                let x_result = res.collect_data_by_path(&plot.data.x);
                let y_result = res.collect_data_by_path(&plot.data.y);

                x_result
                    .and_then(|x| {
                        y_result.map(|y| {
                            // Map each pair (x, y) into a Vec<U256>
                            let merged_data = x
                                .into_iter()
                                .zip(y.into_iter())
                                .map(|(x_val, y_val)| PlotDataItem {
                                    data: vec![x_val.0, y_val.0],
                                    rainlang: x_val.1,
                                })
                                .collect::<Vec<PlotDataItem>>();
                            PlotData {
                                plot_type: plot.plot_type,
                                name,
                                data: merged_data,
                            }
                        })
                    })
                    .map_err(|e| e.into())
            })
            .collect();

        let plots = plot_data_results?;

        Ok(ChartData { name, plots })
    }

    pub async fn build_chart_datas(&mut self) -> Result<Vec<ChartData>, FuzzRunnerError> {
        let charts = self.settings.charts.clone();
        let mut chart_datas = Vec::new();

        for (name, chart) in charts {
            let chart_data = self.build_chart_data(name, chart.deref().clone()).await?;
            chart_datas.push(chart_data);
        }

        Ok(chart_datas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::config_source::ConfigSource;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_fuzz_runner() {
        let dotrain = r#"
deployers:
    mumbai:
        address: 0x122ff0445BaE2a88C6f5F344733029E0d669D624
    some-deployer:
        address: 0x83aA87e8773bBE65DD34c5C5895948ce9f6cd2af
        network: mumbai
networks:
    mumbai:
        rpc: https://polygon-mumbai.g.alchemy.com/v2/_i0186N-488iRU9wUwMQDreCAKy-MEXa
        chain-id: 80001
scenarios:
    mumbai:
        runs: 500
        bindings:
            bound: 3
    mainnet:
        deployer: some-deployer
        runs: 1
---
#bound !bind it
#fuzzed !fuzz it
#calculate-io
a: bound,
b: fuzzed;
#handle-io
:;
    "#;
        let frontmatter = RainDocument::get_front_matter(dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("mumbai")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        println!("{:#?}", res);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_build_chart_data() {
        let dotrain = r#"
    deployers:
        mumbai:
            address: 0x122ff0445BaE2a88C6f5F344733029E0d669D624
        some-deployer:
            address: 0x83aA87e8773bBE65DD34c5C5895948ce9f6cd2af
            network: mumbai
    networks:
        mumbai:
            rpc: https://polygon-mumbai.g.alchemy.com/v2/_i0186N-488iRU9wUwMQDreCAKy-MEXa
            chain-id: 80001
    scenarios:
        mumbai:
            runs: 500
            bindings:
                bound: 3
        mainnet:
            deployer: some-deployer
            runs: 1
    charts:
        mainChart:
            scenario: mumbai
            plots:
                plot1:
                    data:
                        x: 0.0
                        y: 0.1
                    plot-type: line
                plot2:
                    data:
                        x: 0.0
                        y: 0.2
                    plot-type: bar
    ---
    #bound !bind it
    #fuzzed !fuzz it
    #calculate-io
    a: bound,
    b: fuzzed,
    c: 1;
    #handle-io
    :;
        "#;
        let frontmatter = RainDocument::get_front_matter(dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();

        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(dotrain, config, None).await;

        let chart_data = runner.build_chart_datas().await.unwrap();

        println!("{:#?}", chart_data);

        assert_eq!(chart_data.len(), 1);
        assert_eq!(chart_data[0].name, "mainChart");
        assert_eq!(chart_data[0].plots.len(), 2);

        // Collect plot names from the result
        let plot_names: Vec<String> = chart_data[0]
            .plots
            .iter()
            .map(|plot| plot.name.clone())
            .collect();

        // Check for the presence of expected plot names
        assert!(plot_names.contains(&"plot1".to_string()));
        assert!(plot_names.contains(&"plot2".to_string()));
    }
}
