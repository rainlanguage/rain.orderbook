use rain_orderbook_common::local_db::{pipeline::StatusBus, LocalDbError};

#[derive(Clone, Default)]
pub struct ProducerStatusBus;

impl ProducerStatusBus {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ProducerStatusBus {
    async fn send(&self, _message: &str) -> Result<(), LocalDbError> {
        Ok(())
    }
}
