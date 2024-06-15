use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
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
use rain_orderbook_app_settings::blocks::BlockError;
use rain_orderbook_app_settings::chart::Chart;
use rain_orderbook_app_settings::config::*;
use rain_orderbook_app_settings::scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData {
    scenarios_data: HashMap<String, FuzzResultFlat>,
    charts: HashMap<String, Chart>,
}

#[derive(Debug)]
pub struct FuzzResult {
    pub scenario: String,
    pub runs: Vec<RainEvalResult>,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize)]
pub struct FuzzResultFlat {
    pub scenario: String,
    pub column_names: Vec<String>,
    #[typeshare(serialized_as = "Vec<Vec<String>>")]
    pub data: Vec<Vec<U256>>,
}

impl FuzzResult {
    pub fn collect_data_by_path(&self, path: &str) -> Result<Vec<U256>, TraceSearchError> {
        let mut collection: Vec<U256> = vec![];
        // loop over the runs and search_trace_by_path for each
        for run in self.runs.iter() {
            let stack = run.search_trace_by_path(path)?;
            collection.push(stack);
        }
        Ok(collection)
    }

    pub fn flatten_traces(&self) -> Result<FuzzResultFlat, FuzzRunnerError> {
        let mut column_names: Vec<String> = vec![];
        let mut source_paths: Vec<String> = vec![];

        let first_run_traces = &self
            .runs
            .first()
            .ok_or(FuzzRunnerError::ScenarioNoRuns)?
            .traces;

        for trace in first_run_traces.iter() {
            let current_path = if trace.parent_source_index == trace.source_index {
                format!("{}", trace.source_index)
            } else {
                source_paths
                    .iter()
                    .rev()
                    .find_map(|recent_path| {
                        recent_path.split('.').last().and_then(|last_part| {
                            if last_part == trace.parent_source_index.to_string() {
                                Some(format!("{}.{}", recent_path, trace.source_index))
                            } else {
                                None
                            }
                        })
                    })
                    .ok_or_else(|| FuzzRunnerError::CorruptTraces)?
            };

            for (index, _) in trace.stack.iter().enumerate() {
                column_names.push(format!("{}.{}", current_path, index));
            }

            source_paths.push(current_path);
        }

        let mut data: Vec<Vec<U256>> = vec![];

        for run in self.runs.iter() {
            let mut run_data: Vec<U256> = vec![];
            for trace in run.traces.iter() {
                let mut stack = trace.stack.clone();
                stack.reverse();
                for stack_item in stack.iter() {
                    run_data.push(*stack_item);
                }
            }
            data.push(run_data);
        }
        Ok(FuzzResultFlat {
            scenario: self.scenario.clone(),
            column_names,
            data,
        })
    }
}

#[derive(Clone)]
pub struct FuzzRunner {
    pub forker: Forker,
    pub dotrain: String,
    pub rng: TestRng,
    pub settings: Config,
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error("Scenario has no runs defined")]
    ScenarioNoRuns,
    #[error("Corrupt traces")]
    CorruptTraces,
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
    #[error(transparent)]
    ReadableClientHttpError(#[from] ReadableClientError),
    #[error(transparent)]
    BlockError(#[from] BlockError),
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
        // If the scenario doesn't have runs, default is 1
        let no_of_runs = scenario.runs.unwrap_or(1);

        let deployer = scenario.deployer.clone();

        // Fetch the latest block number
        let block_number = ReadableClientHttp::new_from_url(deployer.network.rpc.to_string())?
            .get_block_number()
            .await?;

        let blocks = scenario
            .blocks
            .as_ref()
            .map_or(Ok(vec![block_number]), |b| {
                b.expand_to_block_numbers(block_number)
            })?;

        // Create a fork with the first block number
        self.forker
            .add_or_select(
                NewForkedEvm {
                    fork_url: deployer.network.rpc.clone().into(),
                    fork_block_number: Some(blocks[0]),
                },
                None,
            )
            .await?;

        // Pull out the bindings from the scenario
        let scenario_bindings: Vec<Rebind> = scenario
            .bindings
            .clone()
            .into_iter()
            .map(|(k, v)| Rebind(k, v))
            .collect();

        // Create a new RainDocument with the dotrain and the bindings
        // The bindings in the dotrain string are ignored by the RainDocument
        let rain_document = RainDocument::create(
            self.dotrain.clone(),
            None,
            None,
            Some(scenario_bindings.clone()),
        );

        // Search the namespace hash map for NamespaceItems that are elided and make a vec of the keys
        let elided_binding_keys = Arc::new(
            rain_document
                .namespace()
                .iter()
                .filter(|(_, v)| v.is_elided_binding())
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>(),
        );

        let mut handles = vec![];

        for block_number in blocks {
            self.forker.roll_fork(Some(block_number), None)?;
            let fork = Arc::new(self.forker.clone()); // Wrap in Arc for shared ownership
            let dotrain = Arc::new(self.dotrain.clone());

            for _ in 0..no_of_runs {
                let fork_clone = Arc::clone(&fork); // Clone the Arc for each thread
                let elided_binding_keys = Arc::clone(&elided_binding_keys);
                let deployer = Arc::clone(&deployer);
                let scenario_bindings = scenario_bindings.clone();
                let dotrain = Arc::clone(&dotrain);

                let mut final_bindings: Vec<Rebind> = vec![];

                // For each scenario.fuzz_binds, add a random value
                for elided_binding in elided_binding_keys.as_slice() {
                    let mut val: [u8; 32] = [0; 32];
                    self.rng.fill_bytes(&mut val);
                    let hex = format!("0x{}", alloy_primitives::hex::encode(val));
                    final_bindings.push(Rebind(elided_binding.to_string(), hex));
                }

                let handle = tokio::spawn(async move {
                    final_bindings.extend(scenario_bindings.clone());

                    let rainlang_string = RainDocument::compose_text(
                        &dotrain,
                        &ORDERBOOK_ORDER_ENTRYPOINTS,
                        None,
                        Some(final_bindings),
                    )?;

                    // Create a 5x5 grid of zero values for context - later we'll
                    // replace these with sane values based on Orderbook context
                    let context = vec![vec![U256::from(0); 5]; 5];

                    let args = ForkEvalArgs {
                        rainlang_string,
                        source_index: 0,
                        deployer: deployer.address,
                        namespace: FullyQualifiedNamespace::default(),
                        context,
                        decode_errors: true,
                    };
                    fork_clone
                        .fork_eval(args)
                        .map_err(FuzzRunnerError::ForkCallError)
                        .await
                });
                handles.push(handle);
            }
        }

        let mut runs: Vec<RainEvalResult> = Vec::new();

        for handle in handles {
            let res = handle.await??;
            runs.push(res.into());
        }

        Ok(FuzzResult {
            scenario: scenario.name.clone(),
            runs,
        })
    }

    pub async fn make_chart_data(&self) -> Result<ChartData, FuzzRunnerError> {
        let charts = self.settings.charts.clone();
        let mut scenarios_data: HashMap<String, FuzzResultFlat> = HashMap::new();

        for (_, chart) in charts.clone() {
            let scenario_name = chart.scenario.name.clone();
            let mut runner = self.clone();
            scenarios_data.entry(scenario_name.clone()).or_insert(
                runner
                    .run_scenario_by_name(&scenario_name)
                    .await?
                    .flatten_traces()?,
            );
        }

        let charts: HashMap<String, Chart> = charts
            .iter()
            .map(|(k, v)| (k.clone(), v.as_ref().clone()))
            .collect();

        Ok(ChartData {
            scenarios_data,
            charts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::config_source::ConfigSource;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_fuzz_runner() {
        let dotrain = format!(
            r#"
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 137
scenarios:
    sepolia:
        runs: 500
        bindings:
            bound: 3
---
#bound !bind it
#fuzzed !fuzz it
#calculate-io
a: bound,
b: fuzzed;
#handle-io
:;
    "#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("sepolia")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        assert!(res.runs.len() == 500);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_block_range() {
        let dotrain = format!(
            r#"
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 137
scenarios:
    sepolia:
        blocks:
            range: [5789000..5789100]
            interval: 10
---
#calculate-io
_: block-number();
#handle-io
:;
    "#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("sepolia")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        println!("{:#?}", res.runs.len());

        assert!(res.runs.len() == 11);

        res.runs.iter().enumerate().for_each(|(i, run)| {
            println!("{:#?}", run.traces[0].stack[0]);
            // assert!(run.traces[0].stack[0] == U256::from(5789000 + i as u64 * 10));
        });
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_nested_flattened_fuzz() {
        let dotrain = format!(
            r#"
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 137
scenarios:
    sepolia:
        runs: 500
        bindings:
            bound: 3
---
#bound !bind it
#fuzzed !fuzz it
#calculate-io
a: 1,
b: 2,
c: call<'nested>(),
d: call<'called-twice>();
#nested
c: 5,
d: call<'called-twice>(),
e: 3;
#called-twice
c: 6,
d: 4;
#handle-io
:;
    "#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("sepolia")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let flattened = res.flatten_traces().unwrap();

        // find the column index of 0.2.3.0
        let column_index = flattened.column_names.iter().position(|x| x == "0.2.3.0");
        // get that from the first row of data
        let value = flattened
            .data
            .first()
            .unwrap()
            .get(column_index.unwrap())
            .unwrap();
        assert!(value == &U256::from(6));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_context_happy() {
        let dotrain = format!(
            r#"
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 137
scenarios:
    sepolia:
        runs: 500
---
#calculate-io
_: context<0 0>(),
_: context<49 49>();
#handle-io
:;
    "#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("sepolia")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let flattened = res.flatten_traces().unwrap();

        for row in flattened.data.iter() {
            for col in row.iter() {
                assert!(col == &U256::from(0));
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_context_unhappy() {
        // if we try to access a context value that is out of bounds, we should get an error

        let dotrain = format!(
            r#"
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 137
scenarios:
    sepolia:
        runs: 500
---
#calculate-io
_: context<50 50>();
#handle-io
:;
    "#,
            rpc_url = rain_orderbook_env::CI_DEPLOY_SEPOLIA_RPC_URL
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("sepolia")
            .await
            .map_err(|e| println!("{:#?}", e));

        assert!(res.is_err());
    }
}
