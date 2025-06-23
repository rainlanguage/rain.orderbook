use rain_orderbook_common::fuzz::FuzzRunner;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::CommandError;

pub struct SharedState {
    pub debug_runner: Arc<Mutex<FuzzRunner>>,
}

impl SharedState {
    pub fn new() -> Result<Self, CommandError> {
        let debug_runner = Arc::new(Mutex::new(FuzzRunner::new(None)?));
        Ok(Self { debug_runner })
    }
}
