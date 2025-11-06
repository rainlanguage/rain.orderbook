use super::{RaindexClient, RaindexError};
use crate::local_db::LocalDbError;
use pipeline::runner::scheduler;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

pub mod executor;
pub mod pipeline;
pub mod query;

impl From<LocalDbError> for WasmEncodedError {
    fn from(value: LocalDbError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum LocalDbStatus {
    Active,
    Syncing,
    Failure,
}
impl_wasm_traits!(LocalDbStatus);

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "startLocalDbScheduler", unchecked_return_type = "void")]
    pub async fn start_local_db_scheduler(
        &self,
        #[wasm_export(
            js_name = "settingsYaml",
            param_description = "Full settings YAML string used by the client runner"
        )]
        settings_yaml: String,
        #[wasm_export(
            js_name = "statusCallback",
            param_description = "Optional callback invoked with the current local DB status"
        )]
        status_callback: Option<js_sys::Function>,
    ) -> Result<(), RaindexError> {
        let callback = {
            let slot = self.local_db_callback.borrow();
            slot.clone()
                .ok_or_else(|| RaindexError::JsError("Local DB callback not set".to_string()))?
        };

        let scheduler_cell = Rc::clone(&self.local_db_scheduler);
        let existing = {
            let mut slot = scheduler_cell.borrow_mut();
            slot.take()
        };

        if let Some(handle) = existing {
            handle.stop().await;
        }

        let handle = scheduler::start(settings_yaml, callback, status_callback)?;
        {
            let mut slot = scheduler_cell.borrow_mut();
            *slot = Some(handle);
        }
        Ok(())
    }

    #[wasm_export(js_name = "stopLocalDbScheduler", unchecked_return_type = "void")]
    pub async fn stop_local_db_scheduler(&self) -> Result<(), RaindexError> {
        let scheduler_cell = Rc::clone(&self.local_db_scheduler);
        let handle = {
            let mut slot = scheduler_cell.borrow_mut();
            slot.take()
        };

        if let Some(handle) = handle {
            handle.stop().await;
        }

        Ok(())
    }
}
