use crate::{
    add_order::ORDERBOOK_ORDER_ENTRYPOINTS,
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
use thiserror::Error;

pub struct FuzzRunner {
    pub forked_evm: ForkedEvm,
    pub dotrain: String,
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
}

impl FuzzRunner {
    pub async fn new(fork_cfg: NewForkedEvm, dotrain: &str, seed: Option<[u8; 32]>) -> Self {
        Self {
            forked_evm: ForkedEvm::new(fork_cfg).await,
            dotrain: dotrain.into(),
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
        }
    }

    pub async fn run_scenario(
        &mut self,
        scenario: RunnerScenario,
    ) -> Result<FuzzResult, FuzzRunnerError> {
        let mut runs: Vec<RainEvalResult> = Vec::new();

        let (deployer, _valid_inputs, _valid_outputs, rebinds) =
            try_parse_frontmatter(&self.dotrain)?;

        let rebinds = rebinds.unwrap_or_default();

        for _ in 0..scenario.runs {
            let mut rebinds = rebinds.clone();

            // for each scenario.rebinds, add a random value
            for rebind in &scenario.binds {
                let mut val: [u8; 32] = [0; 32];
                self.rng.fill_bytes(&mut val);
                let hex = format!("0x{}", alloy_primitives::hex::encode(val));
                rebinds.push(Rebind(rebind.to_string(), hex));
            }

            let rainlang_string = RainDocument::compose_text(
                &*self.dotrain.clone(),
                &ORDERBOOK_ORDER_ENTRYPOINTS,
                None,
                Some(rebinds),
            )?;

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
        let frontmatter = RainDocument::get_front_matter(&self.dotrain.as_str())
            .ok_or(FuzzRunnerError::EmptyFrontmatter)?;

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
        let frontmatter = RainDocument::get_front_matter(&self.dotrain.as_str())
            .ok_or(FuzzRunnerError::EmptyFrontmatter)?;

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

// tokens:
//   dai:
//     address: 0x...
//     decimals: 18
//   usdt:
//     address: 0x...
//     decimals: 6
//   usdc:
//     address: 0x...
//     decimals: 6

// orders:
//   usdt+dai:
//     deployer: 0x...
//     valid-inputs:
//       - token: dai
//         vault-id: 0x...
//       - token: usdt
//         vault-id: 0x...
//     valid-outputs:
//       - token: dai
//         vault-id: 0x...
//       - token: usddt
//         vault-id: 0x...
//   usdt+usdc+dai:
//     deployer: 0x...
//     valid-inputs:
//       - token: dai
//         vault-id: 0x...
//       - token: usdt
//         vault-id: 0x...
//       - token: usdc
//         vault-id: 0x...
//     valid-outputs:
//       - token: dai
//         vault-id: 0x...
//       - token: usddt
//         vault-id: 0x...
//       - token: usdcc
//         vault-id: 0x...

// scenarios:
//   main:
//     bind:
//       - some-binding: 12345
//   test:
//     runs: 100
//     bind:
//       - to-be-fuzzed
//       - some-binding
