use crate::local_db::pipeline::StatusBus;
use crate::local_db::LocalDbError;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClientStatusBus;

impl ClientStatusBus {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ClientStatusBus {
    async fn send(&self, _message: &str) -> Result<(), LocalDbError> {
        Ok(())
    }
}
