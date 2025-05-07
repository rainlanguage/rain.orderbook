use rain_orderbook_common::fuzz::FuzzRunner;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SharedState {
    pub debug_runner: Arc<Mutex<FuzzRunner>>,
}
impl Default for SharedState {
    fn default() -> Self {
        Self {
            debug_runner: Arc::new(Mutex::new(FuzzRunner::new(None))),
        }
    }
}
