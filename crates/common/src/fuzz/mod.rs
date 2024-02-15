use crate::{
    dotrain_add_order_lsp::LANG_SERVICES,
    frontmatter::{try_parse_frontmatter, FrontmatterError},
};
use dotrain::{error::ComposeError, RainDocument, Rebind};
use proptest::prelude::RngCore;
use proptest::test_runner::{RngAlgorithm, TestRng};
use rain_interpreter_bindings::IInterpreterStoreV1::FullyQualifiedNamespace;
use rain_interpreter_eval::{
    eval::ForkEvalArgs,
    fork::{ForkCallError, ForkedEvm, NewForkedEvm},
    trace::RainEvalResult,
};
use strict_yaml_rust::StrictYamlLoader;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct RunnerScenario {
    pub name: String,
    pub entrypoint: String,
    pub runs: u64,
    pub rebinds: Vec<String>,
}

pub struct FuzzRunner {
    pub forked_evm: ForkedEvm,
    pub document: RainDocument,
    pub rng: TestRng,
}

#[derive(Debug)]
pub struct FuzzResult {
    pub scenario: String,
    pub runs: Vec<RainEvalResult>,
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error(transparent)]
    ForkCallError(#[from] ForkCallError),
    #[error("Empty Front Matter")]
    EmptyFrontmatter,
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error("Front Matter: {0}")]
    FrontmatterError(#[from] FrontmatterError),
    #[error("Scenario Parsing Error: {0}")]
    ScenarioParsingError(#[from] ScenarioParsingError),
}

#[derive(Error, Debug)]
pub enum ScenarioParsingError {
    #[error("No scenarios")]
    NoScenarios,
    #[error("No name")]
    NoName,
    #[error("Invalid runs - must be an integer")]
    InvalidRuns,
    #[error("No runs specified")]
    NoRuns,
    #[error("No rebinds")]
    NoRebinds,
    #[error("No entrypoint")]
    NoEntrypoint,
}

impl FuzzRunner {
    pub async fn new(fork_cfg: NewForkedEvm, dotrain: &str, seed: Option<[u8; 32]>) -> Self {
        Self {
            forked_evm: ForkedEvm::new(fork_cfg).await,
            document: RainDocument::create(dotrain.into(), None, None, None),
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
        }
    }

    pub fn parse_scenarios(&self) -> Result<Vec<RunnerScenario>, FuzzRunnerError> {
        let frontmatter = self.document.front_matter();
        let frontmatter_yaml_vec = StrictYamlLoader::load_from_str(frontmatter)
            .map_err(FrontmatterError::FrontmatterInvalidYaml)?;
        let frontmatter_yaml = frontmatter_yaml_vec
            .first()
            .ok_or(FuzzRunnerError::EmptyFrontmatter)?;

        let scenarios_yaml = frontmatter_yaml["scenarios"]
            .as_vec()
            .ok_or(FuzzRunnerError::EmptyFrontmatter)?;

        let mut scenarios: Vec<RunnerScenario> = Vec::new();

        for scenario_yaml in scenarios_yaml {
            let name = scenario_yaml["name"]
                .as_str()
                .ok_or(ScenarioParsingError::NoName)?
                .to_string();
            let entrypoint = scenario_yaml["entrypoint"]
                .as_str()
                .ok_or(ScenarioParsingError::NoName)?
                .to_string();
            let runs = scenario_yaml["runs"]
                .as_str()
                .ok_or(ScenarioParsingError::NoRuns)?
                .parse()
                .map_err(|_| ScenarioParsingError::InvalidRuns)?;
            let rebinds = scenario_yaml["bind"]
                .as_vec()
                .map(|rebinds| {
                    rebinds
                        .iter()
                        .filter_map(|binding| binding.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .ok_or(ScenarioParsingError::NoRebinds)?;

            scenarios.push(RunnerScenario {
                name,
                entrypoint,
                runs,
                rebinds,
            });
        }

        Ok(scenarios)
    }

    pub async fn run_scenario(
        &mut self,
        scenario: RunnerScenario,
    ) -> Result<FuzzResult, FuzzRunnerError> {
        let mut runs: Vec<RainEvalResult> = Vec::new();

        let (deployer, _valid_inputs, _valid_outputs, rebinds) =
            try_parse_frontmatter(self.document.front_matter())?;

        let rebinds = rebinds.unwrap_or_default();

        for _ in 0..scenario.runs {
            let mut rebinds = rebinds.clone();
            // for each scenario.rebinds, add a random value
            for rebind in &scenario.rebinds {
                let mut val: [u8; 32] = [0; 32];
                self.rng.fill_bytes(&mut val);
                let hex = format!("0x{}", alloy_primitives::hex::encode(val));
                rebinds.push(Rebind(rebind.to_string(), hex));
            }

            let document = RainDocument::create(
                self.document.text().into(),
                Some(LANG_SERVICES.meta_store()),
                None,
                Some(rebinds),
            );

            let rainlang_string = document.compose(&[&scenario.entrypoint])?;

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                deployer,
                namespace: FullyQualifiedNamespace::default(),
                context: vec![],
            };
            let result = self.forked_evm.fork_eval(args).await?;
            runs.push(result.into());
        }

        Ok(FuzzResult {
            scenario: scenario.name,
            runs,
        })
    }

    pub async fn run_all_scenarios(&mut self) -> Result<Vec<FuzzResult>, FuzzRunnerError> {
        let scenarios = self.parse_scenarios()?;
        let mut results: Vec<FuzzResult> = Vec::new();

        for scenario in scenarios {
            let result = self.run_scenario(scenario.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    pub async fn run_scenario_by_name(
        &mut self,
        name: &str,
    ) -> Result<FuzzResult, FuzzRunnerError> {
        let scenarios = self.parse_scenarios()?;
        let scenario = scenarios
            .iter()
            .find(|s| s.name == name)
            .ok_or(FuzzRunnerError::ScenarioNotFound(name.into()))?;

        self.run_scenario(scenario.clone()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_interpreter_eval::fork::NewForkedEvm;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_fuzz_runner() {
        let fork_cfg = NewForkedEvm {
            fork_url: "https://rpc.ankr.com/polygon_mumbai".into(),
            fork_block_number: Some(45658085),
        };

        let dotrain = r#"
orderbook:
    order:
        deployer: 0x0754030e91F316B2d0b992fe7867291E18200A77
        valid-inputs:
        - token: 0x2222222222222222222222222222222222222222
          decimals: 18
          vault-id: 0x1234
        valid-outputs:
        - token: 0x5555555555555555555555555555555555555555
          decimals: 8
          vault-id: 0x5678 
bind:
    - some-binding: 12345
scenarios:
    - name: scenario 1
      entrypoint: main
      runs: 3
      bind:
        - to-be-fuzzed
    - name: scenario 2
      entrypoint: other
      runs: 3
      bind:
        - some-binding
---
#some-binding 3
#to-be-fuzzed 2
#main
a: 1,
b: to-be-fuzzed,
_: int-add(a b);
#other
_: some-binding,
_: 999;
    "#;

        let mut runner = FuzzRunner::new(fork_cfg, dotrain, None).await;
        let runs = runner
            .run_all_scenarios()
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        println!("{:#?}", runs);

        let single_scenario = runner
            .run_scenario_by_name("scenario 2".into())
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();

        println!("{:#?}", single_scenario);
    }
}
