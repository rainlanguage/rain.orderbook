use rain_interpreter_eval::{
    eval::ForkEvalArgs,
    fork::{ForkedEvm, NewForkedEvm},
    trace::RainEvalResult,
};

pub struct FuzzRunner {
    pub forked_evm: ForkedEvm,
}

pub struct FuzzRunCfg {
    pub input_path: String,
    pub runs: u64,
}

pub struct FuzzResult {
    pub runs: Vec<RainEvalResult>,
}

impl FuzzRunner {
    pub async fn new(fork_cfg: NewForkedEvm) -> Self {
        Self {
            forked_evm: ForkedEvm::new(fork_cfg).await,
        }
    }
    pub fn run(&self, input: &[u8]) -> FuzzResult {
        let mut runs: Vec<RainEvalResult> = Vec::new();
    }
}
