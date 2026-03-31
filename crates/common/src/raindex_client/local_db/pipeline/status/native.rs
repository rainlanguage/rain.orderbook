use crate::local_db::pipeline::{StatusBus, SyncPhase};
use crate::local_db::{LocalDbError, RaindexIdentifier};

#[derive(Debug, Clone, Default)]
pub struct TracingStatusBus {
    ob_id: Option<RaindexIdentifier>,
    raindex_key: Option<String>,
}

impl TracingStatusBus {
    pub fn new() -> Self {
        Self {
            ob_id: None,
            raindex_key: None,
        }
    }

    pub fn with_ob_id(ob_id: RaindexIdentifier) -> Self {
        Self {
            ob_id: Some(ob_id),
            raindex_key: None,
        }
    }

    pub fn with_ob_id_and_key(ob_id: RaindexIdentifier, key: String) -> Self {
        Self {
            ob_id: Some(ob_id),
            raindex_key: Some(key),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for TracingStatusBus {
    async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
        let chain_id = self.ob_id.as_ref().map(|id| id.chain_id).unwrap_or(0);
        let ob_addr = self
            .ob_id
            .as_ref()
            .map(|id| format!("{:#x}", id.raindex_address))
            .unwrap_or_default();
        let key = self.raindex_key.as_deref().unwrap_or("unknown");

        tracing::debug!(
            chain_id = chain_id,
            raindex = %ob_addr,
            key = key,
            phase = %phase.to_message(),
            "sync phase"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::SyncPhase;
    use crate::local_db::RaindexIdentifier;
    use alloy::primitives::address;

    fn test_ob_id() -> RaindexIdentifier {
        RaindexIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    #[tokio::test]
    async fn tracing_status_bus_send_returns_ok() {
        let ob_id = test_ob_id();
        let bus = TracingStatusBus::with_ob_id(ob_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tracing_status_bus_send_without_ob_id_returns_ok() {
        let bus = TracingStatusBus::new();
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }
}
