use super::*;
use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use alloy::primitives::private::rand;
use alloy::primitives::U256;
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use dotrain::{error::ComposeError, RainDocument, Rebind};
use futures::TryFutureExt;
use proptest::prelude::RngCore;
use proptest::test_runner::{RngAlgorithm, TestRng};
use rain_interpreter_bindings::IInterpreterStoreV1::FullyQualifiedNamespace;
use rain_interpreter_eval::fork::NewForkedEvm;
pub use rain_interpreter_eval::trace::{RainEvalResultError, RainEvalResults, TraceSearchError};
use rain_interpreter_eval::{
    error::ForkCallError, eval::ForkEvalArgs, fork::Forker, trace::RainEvalResult,
};
use rain_orderbook_app_settings::blocks::BlockError;
use rain_orderbook_app_settings::chart::ChartCfg;
use rain_orderbook_app_settings::config::*;
use rain_orderbook_app_settings::scenario::ScenarioCfg;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct FuzzResult {
    pub scenario: String,
    pub runs: RainEvalResults,
}

impl FuzzResult {
    pub fn flatten_traces(&self) -> Result<FuzzResultFlat, FuzzRunnerError> {
        let result_table = self.runs.into_flattened_table()?;

        Ok(FuzzResultFlat {
            scenario: self.scenario.clone(),
            data: result_table,
        })
    }
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("Scenario not found: {0}")]
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
    #[error(transparent)]
    RainEvalResultError(#[from] RainEvalResultError),
}

#[derive(Clone)]
pub struct FuzzRunner {
    pub forker: Forker,
    pub dotrain: String,
    pub rng: TestRng,
    pub settings: Config,
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
        scenario: &Arc<ScenarioCfg>,
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

        let dotrain = Arc::new(self.dotrain.clone());
        let mut handles = vec![];

        for block_number in blocks {
            self.forker.roll_fork(Some(block_number), None)?;
            let fork = Arc::new(self.forker.clone()); // Wrap in Arc for shared ownership

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
                    let hex = alloy::primitives::hex::encode_prefixed(val);
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
                    let mut context = vec![vec![U256::from(0); 5]; 5];
                    // set random hash for context order hash cell
                    context[1][0] = rand::random();

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
            scenario: scenario.key.clone(),
            runs: runs.into(),
        })
    }

    pub async fn make_chart_data(&self) -> Result<ChartData, FuzzRunnerError> {
        let charts = self.settings.charts.clone();
        let mut scenarios_data: HashMap<String, FuzzResultFlat> = HashMap::new();

        for (_, chart) in charts.clone() {
            let scenario_name = chart.scenario.key.clone();
            let mut runner = self.clone();
            scenarios_data.entry(scenario_name.clone()).or_insert(
                runner
                    .run_scenario_by_name(&scenario_name)
                    .await?
                    .flatten_traces()?,
            );
        }

        let charts: HashMap<String, ChartCfg> = charts
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
    use alloy::{
        primitives::utils::parse_ether,
        providers::{ext::AnvilApi, Provider},
    };
    use rain_orderbook_app_settings::config_source::ConfigSource;
    use rain_orderbook_test_fixtures::LocalEvm;

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_fuzz_runner() {
        let local_evm = LocalEvm::new().await;
        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        runs: 50
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
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        assert!(res.runs.len() == 50);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_block_range() {
        let local_evm = LocalEvm::new().await;

        let start_block_number = local_evm.provider.get_block_number().await.unwrap();
        let last_block_number = start_block_number + 10;
        local_evm
            .provider
            .anvil_mine(Some(U256::from(10)), None)
            .await
            .unwrap();

        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        blocks:
            range: [{start_block}..{end_block}]
            interval: 2
---
#calculate-io
_: block-number();
#handle-io
:;
    "#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address(),
            start_block = start_block_number,
            end_block = last_block_number
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        assert_eq!(res.runs.len(), 6);

        res.runs.iter().enumerate().for_each(|(i, run)| {
            assert_eq!(
                run.traces[0].stack[0],
                parse_ether(&(start_block_number + (i as u64 * 2)).to_string()).unwrap()
            );
        });
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_nested_flattened_fuzz() {
        let local_evm = LocalEvm::new().await;
        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        runs: 50
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
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let flattened = res.flatten_traces().unwrap();

        // find the column index of 0.2.3.0
        let column_index = flattened
            .data
            .column_names
            .iter()
            .position(|x| x == "0.2.3.0");
        // get that from the first row of data
        let value = flattened
            .data
            .rows
            .first()
            .unwrap()
            .get(column_index.unwrap())
            .unwrap();
        assert_eq!(value, &parse_ether("6").unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_context_happy() {
        let local_evm = LocalEvm::new().await;
        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        runs: 50
---
#calculate-io
_: context<0 0>(),
_: context<4 4>();
#handle-io
:;
    "#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let flattened = res.flatten_traces().unwrap();

        for row in flattened.data.rows.iter() {
            for col in row.iter() {
                assert!(col == &U256::from(0));
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_context_unhappy() {
        // if we try to access a context value that is out of bounds, we should get an error
        let local_evm = LocalEvm::new().await;
        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        runs: 50
---
#calculate-io
_: context<50 50>();
#handle-io
:;
    "#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e));

        assert!(res.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_context_random_order_hash() {
        let local_evm = LocalEvm::new().await;

        // random order hash is at <1 0> context cell, ie column1 row0
        let dotrain = format!(
            r#"
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        runs: 20
---
#calculate-io
_: context<1 0>();
#handle-io
:;
    "#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let frontmatter = RainDocument::get_front_matter(&dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        let config = settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        let mut runner = FuzzRunner::new(&dotrain, config, None).await;

        let res = runner
            .run_scenario_by_name("some-key")
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let flattened = res.flatten_traces().unwrap();

        // flatten the result and check all context order id hashes
        // dont match each oher, ie ensuring their randomness
        let result = flattened
            .data
            .rows
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for (i, cell_value) in result.iter().enumerate() {
            for (j, other_cell_value) in result.iter().enumerate() {
                if i != j {
                    assert!(cell_value != other_cell_value);
                }
            }
        }
    }
}
