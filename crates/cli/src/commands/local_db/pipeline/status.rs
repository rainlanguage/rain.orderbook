use rain_orderbook_common::local_db::{pipeline::StatusBus, LocalDbError, OrderbookIdentifier};
use tracing::info;

#[derive(Clone)]
pub struct ProducerStatusBus {
    debug: bool,
    orderbook_key: String,
    ob_id: OrderbookIdentifier,
}

impl ProducerStatusBus {
    pub fn new(debug: bool, orderbook_key: String, ob_id: OrderbookIdentifier) -> Self {
        Self {
            debug,
            orderbook_key,
            ob_id,
        }
    }
}

impl Default for ProducerStatusBus {
    fn default() -> Self {
        Self {
            debug: false,
            orderbook_key: "<unknown>".to_string(),
            ob_id: OrderbookIdentifier::new(0, Default::default()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ProducerStatusBus {
    async fn send(&self, message: &str) -> Result<(), LocalDbError> {
        if self.debug {
            info!(
                target: "local_db_status",
                chain_id = self.ob_id.chain_id,
                orderbook = %self.ob_id.orderbook_address,
                orderbook_key = %self.orderbook_key,
                "{message}"
            );
        }
        Ok(())
    }
}
