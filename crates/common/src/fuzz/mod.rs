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
    fork::{ForkedEvm, NewForkedEvm},
    trace::RainEvalResult,
};
use thiserror::Error;

pub struct FuzzRunner {
    pub forked_evm: ForkedEvm,
}

pub struct FuzzRunCfg<'a> {
    pub dotrain: &'a str,
    pub number_of_runs: u64,
    pub entrypoint: &'a str,
}

pub struct FuzzResult {
    pub runs: Vec<RainEvalResult>,
}

#[derive(Error, Debug)]
pub enum FuzzRunnerError {
    #[error("ForkCallError")]
    ForkCallError,
    #[error("Empty Front Matter")]
    EmptyFrontmatter,
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error("Front Matter: {0}")]
    FrontmatterError(#[from] FrontmatterError),
}

impl FuzzRunner {
    pub async fn new(fork_cfg: NewForkedEvm) -> Self {
        Self {
            forked_evm: ForkedEvm::new(fork_cfg).await,
        }
    }
    pub async fn run<'a>(
        &mut self,
        run_cfg: FuzzRunCfg<'a>,
    ) -> Result<Vec<RainEvalResult>, FuzzRunnerError> {
        let mut runs: Vec<RainEvalResult> = Vec::new();

        let meta_store = LANG_SERVICES.meta_store();
        let frontmatter = RainDocument::get_front_matter(&run_cfg.dotrain)
            .ok_or(FuzzRunnerError::EmptyFrontmatter)?;

        // Prepare call
        let (deployer, _valid_inputs, _valid_outputs, rebinds) =
            try_parse_frontmatter(frontmatter)?;

        let rebinds = rebinds.unwrap_or_default();

        let mut rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[2; 32]);

        for _ in 0..run_cfg.number_of_runs {
            let mut rebinds = rebinds.clone();
            let mut val: [u8; 32] = [0; 32];
            rng.fill_bytes(&mut val);
            let hex = format!("0x{}", alloy_primitives::hex::encode(val));
            rebinds.push(Rebind("fuzz".into(), hex.into()));

            let dotrain_doc = RainDocument::create(
                run_cfg.dotrain.into(),
                Some(meta_store.clone()),
                None,
                Some(rebinds),
            );
            let rainlang_string = dotrain_doc.compose(&[&run_cfg.entrypoint])?;

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                deployer,
                namespace: FullyQualifiedNamespace::default(),
                context: vec![],
            };
            let result = self
                .forked_evm
                .fork_eval(args)
                .await
                .map_err(|_e| FuzzRunnerError::ForkCallError)?;
            runs.push(result.into());
        }
        Ok(runs)
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
        let mut runner = FuzzRunner::new(fork_cfg).await;

        let dotrain = r#"orderbook:
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
        ---
        #fuzz 3
        #main
        _: fuzz;
            "#;

        let runs = runner
            .run(FuzzRunCfg {
                dotrain,
                number_of_runs: 4,
                entrypoint: "main",
            })
            .await
            .unwrap();
        println!("{:#?}", runs);
    }
}
