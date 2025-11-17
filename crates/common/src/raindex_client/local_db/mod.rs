use super::{RaindexClient, RaindexError};
use crate::local_db::LocalDbError;
use pipeline::runner::scheduler;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

pub mod executor;
pub mod orders;
pub mod pipeline;
pub mod query;
pub mod vaults;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbStatusSnapshot {
    pub status: LocalDbStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(LocalDbStatusSnapshot);

impl LocalDbStatusSnapshot {
    pub fn new(status: LocalDbStatus, error: Option<String>) -> Self {
        Self { status, error }
    }

    pub fn active() -> Self {
        Self::new(LocalDbStatus::Active, None)
    }

    pub fn syncing() -> Self {
        Self::new(LocalDbStatus::Syncing, None)
    }

    pub fn failure(error: String) -> Self {
        Self::new(LocalDbStatus::Failure, Some(error))
    }
}

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

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use gloo_timers::future::TimeoutFuture;
    use rain_orderbook_app_settings::yaml::{
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlParsable,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::{
        prelude::{serde_wasm_bindgen, JsValue},
        result::WasmEncodedResult,
    };

    wasm_bindgen_test_configure!(run_in_browser);

    const SINGLE_ORDERBOOK_SETTINGS_YAML: &str = r#"
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
local-db-remotes:
  remote-a: https://manifests.example/a.yaml
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
    bootstrap-block-threshold: 10000
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    local-db-remote: remote-a
    deployment-block: 123
"#;

    fn build_client() -> RaindexClient {
        let orderbook_yaml = OrderbookYaml::new(
            vec![SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned()],
            OrderbookYamlValidation::default(),
        )
        .expect("valid orderbook yaml");

        RaindexClient {
            orderbook_yaml,
            local_db_callback: Rc::new(RefCell::new(None)),
            local_db_scheduler: Rc::new(RefCell::new(None)),
        }
    }

    fn success_callback() -> js_sys::Function {
        let result = WasmEncodedResult::Success::<String> {
            value: "[]".to_string(),
            error: None,
        };
        let js_value = serde_wasm_bindgen::to_value(&result).unwrap();
        js_sys::Function::new_no_args(&format!(
            "return Promise.resolve({})",
            js_sys::JSON::stringify(&js_value)
                .unwrap()
                .as_string()
                .unwrap()
        ))
    }

    fn recording_status_callback(
        store: Rc<RefCell<Vec<LocalDbStatusSnapshot>>>,
    ) -> js_sys::Function {
        let closure = Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(snapshot) = serde_wasm_bindgen::from_value::<LocalDbStatusSnapshot>(value) {
                store.borrow_mut().push(snapshot);
            }
        }) as Box<dyn FnMut(JsValue)>);

        let function: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();
        function
    }

    #[wasm_bindgen_test]
    async fn start_scheduler_without_callback_returns_error() {
        let client = build_client();
        let result = client
            .start_local_db_scheduler(SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned(), None)
            .await;
        assert!(matches!(result, Err(RaindexError::JsError(_))));
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn start_and_stop_scheduler_updates_handle_state() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback())
            .expect("callback set");

        client
            .start_local_db_scheduler(SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned(), None)
            .await
            .expect("scheduler starts");
        assert!(client.local_db_scheduler.borrow().is_some());

        TimeoutFuture::new(0).await;

        client
            .stop_local_db_scheduler()
            .await
            .expect("scheduler stops");
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn restarting_scheduler_replaces_handle() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback())
            .expect("callback set");

        client
            .start_local_db_scheduler(SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned(), None)
            .await
            .expect("first scheduler starts");

        TimeoutFuture::new(0).await;

        let first_handle_ptr = client
            .local_db_scheduler
            .borrow()
            .as_ref()
            .map(|handle| handle.stop_flag_ptr())
            .expect("first handle exists");

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = recording_status_callback(Rc::clone(&statuses));

        client
            .start_local_db_scheduler(
                SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned(),
                Some(status_callback),
            )
            .await
            .expect("second scheduler starts");

        TimeoutFuture::new(0).await;

        let second_handle_ptr = client
            .local_db_scheduler
            .borrow()
            .as_ref()
            .map(|handle| handle.stop_flag_ptr())
            .expect("second handle exists");

        assert_ne!(first_handle_ptr, second_handle_ptr);
        let recorded = statuses.borrow();
        assert!(recorded
            .iter()
            .any(|snapshot| snapshot.status == LocalDbStatus::Active));

        client
            .stop_local_db_scheduler()
            .await
            .expect("scheduler stops");
    }

    #[wasm_bindgen_test]
    async fn start_scheduler_propagates_errors_and_leaves_handle_empty() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback())
            .expect("callback set");

        let result = client
            .start_local_db_scheduler("not yaml".to_owned(), None)
            .await;
        assert!(matches!(result, Err(RaindexError::LocalDbError(_))));
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn stop_scheduler_is_idempotent() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback())
            .expect("callback set");

        client
            .start_local_db_scheduler(SINGLE_ORDERBOOK_SETTINGS_YAML.to_owned(), None)
            .await
            .expect("scheduler starts");

        TimeoutFuture::new(0).await;

        client
            .stop_local_db_scheduler()
            .await
            .expect("first stop succeeds");
        client
            .stop_local_db_scheduler()
            .await
            .expect("second stop succeeds");
        assert!(client.local_db_scheduler.borrow().is_none());
    }
}
