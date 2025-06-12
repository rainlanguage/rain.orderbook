use alloy::primitives::U256;
use alloy_ethers_typecast::transaction::{ReadableClientError, ReadableClientHttp};
use dotrain::{error::ComposeError, RainDocument, Rebind};
use futures::TryFutureExt;
use proptest::{
    prelude::RngCore,
    test_runner::{RngAlgorithm, TestRng},
};
use rain_interpreter_bindings::IInterpreterStoreV1::FullyQualifiedNamespace;
use rain_interpreter_eval::{
    error::ForkCallError,
    eval::ForkEvalArgs,
    fork::{Forker, NewForkedEvm},
    trace::{RainEvalResultError, RainEvalResults},
};
use rain_orderbook_app_settings::{
    blocks::BlockError, config::*, deployer::DeployerCfg, unit_test::TestConfig,
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone)]
pub struct TestRunner {
    pub forker: Forker,
    pub dotrains: Dotrains,
    pub settings: Settings,
    pub rng: TestRng,
    pub test_setup: TestSetup,
}

#[derive(Clone)]
pub struct TestSetup {
    pub block_number: u64,
    pub deployer: Arc<DeployerCfg>,
    pub scenario_name: String,
}

#[derive(Clone)]
pub struct Dotrains {
    pub main_dotrain: String,
    pub test_dotrain: String,
}

#[derive(Clone)]
pub struct Settings {
    pub main_config: Config,
    pub test_config: TestConfig,
}

#[derive(Error, Debug)]
pub enum TestRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error(transparent)]
    ReadableClientHttpError(#[from] ReadableClientError),
    #[error(transparent)]
    BlockError(#[from] BlockError),
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error(transparent)]
    RainEvalResultError(#[from] RainEvalResultError),
}

impl TestRunner {
    pub async fn new(
        dotrain: &str,
        test_dotrain: &str,
        settings: Config,
        test_settings: TestConfig,
        seed: Option<[u8; 32]>,
    ) -> Self {
        Self {
            forker: Forker::new(),
            dotrains: Dotrains {
                main_dotrain: dotrain.into(),
                test_dotrain: test_dotrain.into(),
            },
            settings: Settings {
                main_config: settings,
                test_config: test_settings,
            },
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
            test_setup: TestSetup {
                block_number: 0,
                deployer: Arc::new(DeployerCfg::dummy()),
                scenario_name: String::new(),
            },
        }
    }

    fn get_final_bindings(&mut self, is_test_dotrain: bool) -> Vec<Rebind> {
        let scenario_bindings: Vec<Rebind> = self
            .settings
            .test_config
            .scenario
            .bindings
            .clone()
            .into_iter()
            .map(|(k, v)| Rebind(k, v))
            .collect();
        let mut final_bindings: Vec<Rebind> = vec![];

        let dotrain = if is_test_dotrain {
            self.dotrains.test_dotrain.clone()
        } else {
            self.dotrains.main_dotrain.clone()
        };

        let rain_document =
            RainDocument::create(dotrain, None, None, Some(scenario_bindings.clone()));
        let elided_binding_keys = Arc::new(
            rain_document
                .namespace()
                .iter()
                .filter(|(_, v)| v.is_elided_binding())
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>(),
        );

        let elided_binding_keys = Arc::clone(&elided_binding_keys);
        let scenario_bindings = scenario_bindings.clone();

        for elided_binding in elided_binding_keys.as_slice() {
            let mut val: [u8; 32] = [0; 32];
            self.rng.fill_bytes(&mut val);
            let hex = alloy::primitives::hex::encode_prefixed(val);
            final_bindings.push(Rebind(elided_binding.to_string(), hex));
        }

        final_bindings.extend(scenario_bindings);
        final_bindings
    }

    async fn run_pre_entrypoint(&mut self) -> Result<RainEvalResults, TestRunnerError> {
        let final_bindings = self.get_final_bindings(true);

        let dotrain = Arc::new(self.dotrains.test_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let deployer = Arc::clone(&self.test_setup.deployer);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["pre"], None, Some(final_bindings))?;

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                deployer: deployer.address,
                namespace: FullyQualifiedNamespace::default(),
                context: vec![vec![U256::from(0); 1]; 1],
                decode_errors: true,
            };
            fork_clone
                .fork_eval(args)
                .map_err(TestRunnerError::ForkCallError)
                .await
        });

        Ok(vec![handle.await??.into()].into())
    }

    async fn run_calculate_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let input_token = pre_stack.results[0].stack[2];
        let output_token = pre_stack.results[0].stack[1];
        let output_cap = pre_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(false);

        let dotrain = Arc::new(self.dotrains.main_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let deployer = Arc::clone(&self.test_setup.deployer);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string = RainDocument::compose_text(
                &dotrain,
                &["calculate-io"],
                None,
                Some(final_bindings),
            )?;

            let mut context = vec![vec![U256::from(0); 5]; 5];
            // output cap
            context[2][0] = output_cap;
            // input token
            context[3][0] = input_token;
            // output token
            context[4][0] = output_token;

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
                .map_err(TestRunnerError::ForkCallError)
                .await
        });

        Ok(vec![handle.await??.into()].into())
    }

    async fn run_handle_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
        calculate_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let output_cap = pre_stack.results[0].stack[0];
        let max_output = calculate_stack.results[0].stack[1];
        let _io_ratio = calculate_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(false);

        let dotrain = Arc::new(self.dotrains.main_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let deployer = Arc::clone(&self.test_setup.deployer);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["handle-io"], None, Some(final_bindings))?;

            let mut context = vec![vec![U256::from(0); 5]; 5];

            // output vault decrease
            context[4][4] = U256::min(max_output, output_cap);

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
                .map_err(TestRunnerError::ForkCallError)
                .await
        });

        Ok(vec![handle.await??.into()].into())
    }

    async fn run_post_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
        calculate_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let input_token = pre_stack.results[0].stack[2];
        let output_token = pre_stack.results[0].stack[1];
        let output_cap = pre_stack.results[0].stack[0];
        let max_output = calculate_stack.results[0].stack[1];
        let io_ratio = calculate_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(true);

        let dotrain = Arc::new(self.dotrains.test_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let deployer = Arc::clone(&self.test_setup.deployer);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["post"], None, Some(final_bindings))?;

            let mut context = vec![vec![U256::from(0); 20]; 20];

            // input token
            context[3][0] = input_token;
            // output token
            context[4][0] = output_token;
            // max output
            context[2][0] = max_output;
            // io ratio
            context[2][1] = io_ratio;
            // output vault decrease
            context[4][4] = U256::min(max_output, output_cap);

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
                .map_err(TestRunnerError::ForkCallError)
                .await
        });

        Ok(vec![handle.await??.into()].into())
    }

    pub async fn run_unit_test(&mut self) -> Result<RainEvalResults, TestRunnerError> {
        self.test_setup.deployer = self
            .settings
            .main_config
            .deployers
            .get(&self.settings.test_config.scenario_name)
            .ok_or(TestRunnerError::ScenarioNotFound(
                self.settings.test_config.scenario_name.clone(),
            ))?
            .clone();

        // Fetch the latest block number
        let block_number = ReadableClientHttp::new_from_urls(vec![self
            .test_setup
            .deployer
            .network
            .rpc
            .to_string()])?
        .get_block_number()
        .await?;
        let blocks = self
            .settings
            .test_config
            .scenario
            .blocks
            .as_ref()
            .map_or(Ok(vec![block_number]), |b| {
                b.expand_to_block_numbers(block_number)
            })?;
        self.test_setup.block_number = blocks[0];

        // Create a fork with the first block number
        self.forker
            .add_or_select(
                NewForkedEvm {
                    fork_url: self.test_setup.deployer.network.rpc.clone().into(),
                    fork_block_number: Some(block_number),
                },
                None,
            )
            .await?;

        let pre_stack = self.run_pre_entrypoint().await?;
        let calculate_stack = self.run_calculate_entrypoint(pre_stack.clone()).await?;
        let _handle_stack = self
            .run_handle_entrypoint(pre_stack.clone(), calculate_stack.clone())
            .await?;
        let results = self.run_post_entrypoint(pre_stack, calculate_stack).await?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::{
        config_source::ConfigSource, spec_version::SpecVersion, unit_test::UnitTestConfigSource,
    };
    use rain_orderbook_test_fixtures::LocalEvm;

    fn get_main_config(dotrain: &str) -> Config {
        let frontmatter = RainDocument::get_front_matter(dotrain).unwrap();
        let settings = serde_yaml::from_str::<ConfigSource>(frontmatter).unwrap();
        settings
            .try_into()
            .map_err(|e| println!("{:?}", e))
            .unwrap()
    }

    fn get_test_config(test_dotrain: &str) -> TestConfig {
        let frontmatter = RainDocument::get_front_matter(test_dotrain).unwrap();
        let source = serde_yaml::from_str::<UnitTestConfigSource>(frontmatter).unwrap();
        source
            .test
            .try_into_test_config()
            .map_err(|e| println!("{:?}", e))
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_test_runner() {
        let local_evm = LocalEvm::new().await;
        let test_dotrain = format!(
            r#"
test:
    # calculate-entrypoint: some-custom-entrypoint
    # handle-entrypoint: some-custom-entrypoint
    scenario-name: some-key
    scenario:
        bindings:
            orderbook-subparser: {orderbook_subparser}
            second-binding: 999
---
#orderbook-subparser !
#second-binding !

#pre
input-token: 0x01,
output-token: 0x02,
output-cap: 10;

#post
using-words-from orderbook-subparser

/* calculate io stack */
:ensure(equal-to(calculated-io-ratio() 999) "io ratio should be 999"),
:ensure(equal-to(calculated-max-output() 10) "max output should be 10"),
:ensure(equal-to(output-token() 0x02) "output token should be 0x02"),
:ensure(equal-to(input-token() 0x01) "input token should be 0x01"),

/* handle io stack */
:ensure(equal-to(output-vault-decrease() 10) "output cap should be 10");
    "#,
            orderbook_subparser = local_evm.orderbook_subparser.address()
        );
        let dotrain = format!(
            r#"
version: {spec_version}
deployers:
    some-key:
        address: {deployer}
networks:
    some-key:
        rpc: {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        bindings:
            orderbook-subparser: {orderbook_subparser}
            second-binding: 20
---
#orderbook-subparser !
#second-binding !

#calculate-io
using-words-from orderbook-subparser

_: input-token(),
_: output-token(),
a: 10,
b: second-binding;
#handle-io
using-words-from orderbook-subparser

_: output-vault-decrease();
    "#,
            rpc_url = local_evm.url(),
            deployer = local_evm.deployer.address(),
            orderbook_subparser = local_evm.orderbook_subparser.address(),
            spec_version = SpecVersion::current()
        );

        let main_config = get_main_config(&dotrain);
        let test_config = get_test_config(&test_dotrain);

        let mut runner =
            TestRunner::new(&dotrain, &test_dotrain, main_config, test_config, None).await;

        runner
            .run_unit_test()
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();
    }
}
