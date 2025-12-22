use rain_orderbook_common::local_db::{pipeline::StatusBus, LocalDbError, OrderbookIdentifier};
use tracing::info;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugStatus {
    #[default]
    Disabled,
    Enabled,
}

impl From<bool> for DebugStatus {
    fn from(value: bool) -> Self {
        if value {
            DebugStatus::Enabled
        } else {
            DebugStatus::Disabled
        }
    }
}

#[derive(Clone)]
pub struct ProducerStatusBus {
    debug: DebugStatus,
    orderbook_key: String,
    ob_id: OrderbookIdentifier,
}

impl ProducerStatusBus {
    pub fn new(debug: DebugStatus, orderbook_key: String, ob_id: OrderbookIdentifier) -> Self {
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
            debug: DebugStatus::Disabled,
            orderbook_key: "<unknown>".to_string(),
            ob_id: OrderbookIdentifier::new(0, Default::default()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ProducerStatusBus {
    async fn send(&self, message: &str) -> Result<(), LocalDbError> {
        if self.debug == DebugStatus::Enabled {
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
