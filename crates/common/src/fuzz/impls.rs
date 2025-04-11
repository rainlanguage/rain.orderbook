use super::*;
use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
use alloy::primitives::private::rand;
use alloy::primitives::U256;
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use dotrain::{error::ComposeError, RainDocument, Rebind};
use futures::TryFutureExt;
use proptest::prelude::RngCore;
use proptest::test_runner::{RngAlgorithm, TestRng};
use rain_interpreter_bindings::IInterpreterStoreV1::FullyQualifiedNamespace;
use rain_interpreter_eval::eval::ForkParseArgs;
use rain_interpreter_eval::fork::NewForkedEvm;
pub use rain_interpreter_eval::trace::{RainEvalResultError, RainEvalResults, TraceSearchError};
use rain_interpreter_eval::{
    error::ForkCallError, eval::ForkEvalArgs, fork::Forker, trace::RainEvalResult,
};
use rain_orderbook_app_settings::blocks::BlockError;
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

#[derive(Clone)]
pub struct FuzzRunner {
    pub forker: Forker,
    pub dotrain: String,
    pub dotrain_yaml: DotrainYaml,
    pub rng: TestRng,
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error("Deployment not found")]
    DeploymentNotFound(String),
    #[error("Order not found")]
    OrderNotFound,
    #[error("Input token not found")]
    InputTokenNotFound,
    #[error("Output token not found")]
    OutputTokenNotFound,
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
    #[error(transparent)]
    AbiDecodedErrorType(#[from] AbiDecodedErrorType),
    #[error(transparent)]
    AbiDecodeFailedErrors(#[from] AbiDecodeFailedErrors),
    #[error(transparent)]
    YamlError(#[from] YamlError),
}

impl FuzzRunner {
    pub async fn new(
        dotrain: &str,
        settings: Option<String>,
        seed: Option<[u8; 32]>,
    ) -> Result<Self, FuzzRunnerError> {
        let frontmatter = RainDocument::get_front_matter(dotrain)
            .unwrap_or("")
            .to_string();

        let source = if let Some(settings) = settings {
            vec![frontmatter.to_string(), settings.to_string()]
        } else {
            vec![frontmatter.to_string()]
        };

        let dotrain_yaml = DotrainYaml::new(source, false)?;

        Ok(Self {
            forker: Forker::new(),
            dotrain: dotrain.into(),
            dotrain_yaml,
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
        })
    }

    pub async fn run_scenario_by_key(&mut self, key: &str) -> Result<FuzzResult, FuzzRunnerError> {
        let scenario = self.dotrain_yaml.get_scenario(key)?;
        self.run_scenario(&scenario).await
    }

    pub async fn run_scenario(
        &mut self,
        scenario: &ScenarioCfg,
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

    pub async fn run_debug(
        &mut self,
        block_number: u64,
        input: OrderIOCfg,
        output: OrderIOCfg,
        scenario: &ScenarioCfg,
    ) -> Result<(String, FuzzResult), FuzzRunnerError> {
        let deployer = scenario.deployer.clone();

        // Create a fork with the first block number
        self.forker
            .add_or_select(
                NewForkedEvm {
                    fork_url: deployer.network.rpc.clone().into(),
                    fork_block_number: Some(block_number),
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
        self.forker.roll_fork(Some(block_number), None)?;
        let fork = Arc::new(self.forker.clone()); // Wrap in Arc for shared ownership
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

        let input_token = input
            .token
            .clone()
            .ok_or(FuzzRunnerError::InputTokenNotFound)?;
        let output_token = output
            .token
            .clone()
            .ok_or(FuzzRunnerError::OutputTokenNotFound)?;

        let input_symbol_res = self
            .forker
            .alloy_call(
                deployer.address,
                input_token.address,
                IERC20::symbolCall {},
                false,
            )
            .await?;
        let output_symbol_res = self
            .forker
            .alloy_call(
                deployer.address,
                output_token.address,
                IERC20::symbolCall {},
                false,
            )
            .await?;
        let pair_symbols = format!(
            "{}/{}",
            input_symbol_res.typed_return._0, output_symbol_res.typed_return._0
        );

        let handle = tokio::spawn(async move {
            final_bindings.extend(scenario_bindings.clone());

            let rainlang_string = RainDocument::compose_text(
                &dotrain,
                &ORDERBOOK_ORDER_ENTRYPOINTS,
                None,
                Some(final_bindings),
            )
            .map_err(FuzzRunnerError::ComposeError)?;

            // Create a 5x5 grid of zero values for context - later we'll
            // replace these with sane values based on Orderbook context
            let mut context = vec![vec![U256::from(0); 5]; 5];
            // set random hash for context order hash cell
            context[1][0] = rand::random();

            // set input values in context
            // input token
            context[3][0] = U256::from_be_slice(input_token.address.0.as_slice());
            // input decimals
            context[3][1] = U256::from(input_token.decimals.unwrap_or(18));
            // input vault id
            context[3][2] = input.vault_id.unwrap_or(U256::from(0));
            // input vault balance before
            context[3][3] = U256::from(0);

            // set output values in context
            // output token
            context[4][0] = U256::from_be_slice(output_token.address.0.as_slice());
            // output decimals
            context[4][1] = U256::from(output_token.decimals.unwrap_or(18));
            // output vault id
            context[4][2] = output.vault_id.unwrap_or(U256::from(0));
            // output vault balance before
            context[4][3] = U256::from(0);

            let parse_result = fork_clone
                .fork_parse(ForkParseArgs {
                    rainlang_string: rainlang_string.clone(),
                    deployer: deployer.address,
                    decode_errors: true,
                })
                .await
                .map_err(FuzzRunnerError::ForkCallError)?;
            let store = fork_clone
                .alloy_call(Address::default(), deployer.address, iStoreCall {}, true)
                .await?
                .typed_return
                ._0;
            let interpreter = fork_clone
                .alloy_call(
                    Address::default(),
                    deployer.address,
                    iInterpreterCall {},
                    true,
                )
                .await?
                .typed_return
                ._0;
            let res = fork_clone.call(
                Address::default().as_slice(),
                interpreter.as_slice(),
                &eval3Call {
                    bytecode: parse_result.typed_return.bytecode,
                    sourceIndex: U256::from(0),
                    store,
                    namespace: FullyQualifiedNamespace::default().into(),
                    context,
                    inputs: vec![],
                }
                .abi_encode(),
            )?;

            let mut error = None;
            if res.exit_reason.is_revert() {
                error = Some(AbiDecodedErrorType::selector_registry_abi_decode(&res.result).await);
            }

            Ok::<
                (
                    RainEvalResult,
                    Option<Result<AbiDecodedErrorType, AbiDecodeFailedErrors>>,
                ),
                FuzzRunnerError,
            >((res.into(), error))
        });

        let (result, _) = handle.await??;

        Ok((
            pair_symbols,
            FuzzResult {
                scenario: scenario.key.clone(),
                runs: vec![result].into(),
            },
        ))
    }

    pub async fn make_chart_data(&self) -> Result<ChartData, FuzzRunnerError> {
        let charts = self.dotrain_yaml.get_charts()?;
        let mut scenarios_data: HashMap<String, FuzzResultFlat> = HashMap::new();

        for (_, chart) in charts.clone() {
            let scenario_key = chart.scenario.key.clone();
            let mut runner = self.clone();
            scenarios_data.entry(scenario_key.clone()).or_insert(
                runner
                    .run_scenario_by_key(&scenario_key)
                    .await?
                    .flatten_traces()?,
            );
        }

        Ok(ChartData {
            scenarios_data,
            charts,
        })
    }

    pub async fn make_debug_data(
        &self,
        block_number: Option<u64>,
    ) -> Result<DeploymentDebugData, FuzzRunnerError> {
        let mut block = block_number.unwrap_or(0);
        let mut pair_datas: HashMap<String, Vec<DeploymentDebugPairData>> = HashMap::new();

        let deployments_keys = self.dotrain_yaml.get_deployment_keys()?;

        for deployment_key in deployments_keys {
            let deployment = self.dotrain_yaml.get_deployment(&deployment_key)?;
            let scenario = deployment.scenario.clone();

            if block_number.is_none() {
                // Fetch the latest block number
                block =
                    ReadableClientHttp::new_from_url(scenario.deployer.network.rpc.to_string())?
                        .get_block_number()
                        .await?;
            }

            for input in &deployment.order.inputs {
                let input_token = input
                    .token
                    .clone()
                    .ok_or(FuzzRunnerError::InputTokenNotFound)?;
                for output in &deployment.order.outputs {
                    let output_token = output
                        .token
                        .clone()
                        .ok_or(FuzzRunnerError::OutputTokenNotFound)?;
                    if input_token.address != output_token.address {
                        let mut pair_data = DeploymentDebugPairData {
                            order: deployment.order.key.clone(),
                            scenario: scenario.key.clone(),
                            pair: "".to_string(),
                            result: None,
                            error: None,
                        };

                        let mut runner = self.clone();
                        match runner
                            .run_debug(block, input.clone(), output.clone(), &scenario)
                            .await
                        {
                            Ok((pair_symbols, fuzz_result)) => match fuzz_result.flatten_traces() {
                                Ok(fuzz_result) => {
                                    pair_data.pair = pair_symbols;
                                    pair_data.result = Some(fuzz_result);
                                }
                                Err(e) => {
                                    pair_data.error = Some(e.to_string());
                                }
                            },
                            Err(e) => {
                                if matches!(e, FuzzRunnerError::ComposeError(_)) {
                                    return Err(e);
                                }
                                pair_data.error = Some(e.to_string());
                            }
                        }

                        pair_datas
                            .entry(deployment_key.clone())
                            .or_default()
                            .push(pair_data);
                    }
                }
            }
        }

        let result = DeploymentDebugData {
            result: pair_datas,
            block_number: U256::from(block),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        primitives::utils::parse_ether,
        providers::{ext::AnvilApi, Provider},
    };
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address(),
            start_block = start_block_number,
            end_block = last_block_number
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address()
        );
        let mut runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .run_scenario_by_key("some-key")
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_debug() {
        let mut local_evm = LocalEvm::new().await;

        let usdce = local_evm
            .deploy_new_token(
                "USDCe",
                "USDCe",
                6,
                U256::from(1_000_000_000_000_000_000u128),
                *local_evm.deployer.address(),
            )
            .await;
        let wflr = local_evm
            .deploy_new_token(
                "WFLR",
                "Wrapped Flare",
                18,
                U256::from(1_000_000_000_000_000_000u128),
                *local_evm.deployer.address(),
            )
            .await;

        let usdce_address = usdce.address();
        let wflr_address = wflr.address();

        let dotrain = format!(
            r#"
deployers:
    flare:
        address: {deployer}
networks:
    flare:
        rpc: {rpc_url}
        chain-id: 123
tokens:
    wflr:
        network: flare
        address: {wflr_address}
        decimals: 18
    usdce:
        network: flare
        address: {usdce_address}
        decimals: 6
scenarios:
    flare:
        deployer: flare
        runs: 1
        bindings:
            orderbook-subparser: {orderbook_subparser} 
orders:
    sell-wflr:
        network: flare
        inputs:
            - token: usdce
              vault-id: 10
        outputs:
            - token: wflr
              vault-id: 20
deployments:
    sell-wflr:
        order: sell-wflr
        scenario: flare
---
#orderbook-subparser !
#calculate-io
using-words-from orderbook-subparser
_: input-token(),
_: input-token-decimals(),
_: input-vault-id(),
_: output-token(),
_: output-token-decimals(),
_: output-vault-id(),
max-output: 30,
io-ratio: mul(0.99 20);
#handle-io
:;
#handle-add-order
:;"#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address(),
            orderbook_subparser = local_evm.orderbook_subparser.address(),
            wflr_address = wflr_address,
            usdce_address = usdce_address,
        );
        let runner = FuzzRunner::new(&dotrain, None, None).await.unwrap();

        let res = runner
            .make_debug_data(None)
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        let result_rows = res.result["sell-wflr"][0]
            .result
            .as_ref()
            .unwrap()
            .data
            .rows[0]
            .clone();
        assert_eq!(
            result_rows[0],
            U256::from_be_slice(usdce_address.as_slice())
        );
        assert_eq!(result_rows[1], U256::from(6));
        assert_eq!(result_rows[2], U256::from(10));
        assert_eq!(result_rows[3], U256::from_be_slice(wflr_address.as_slice()));
        assert_eq!(result_rows[4], U256::from(18));
        assert_eq!(result_rows[5], U256::from(20));
    }
}
