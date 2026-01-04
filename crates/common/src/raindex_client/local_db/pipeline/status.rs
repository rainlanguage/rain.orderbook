use crate::local_db::pipeline::{StatusBus, SyncPhase};
use crate::local_db::{LocalDbError, OrderbookIdentifier};
use crate::raindex_client::local_db::{OrderbookSyncStatus, SchedulerState};
use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_utils::prelude::*;

thread_local! {
    static STATUS_CALLBACK: RefCell<Option<Rc<Function>>> = const { RefCell::new(None) };
    static SCHEDULER_STATE: RefCell<SchedulerState> = const { RefCell::new(SchedulerState::Leader) };
}

pub fn set_status_callback(callback: Option<Rc<Function>>) {
    STATUS_CALLBACK.with(|c| {
        *c.borrow_mut() = callback;
    });
}

pub fn set_scheduler_state(state: SchedulerState) {
    SCHEDULER_STATE.with(|s| {
        *s.borrow_mut() = state;
    });
}

pub fn get_scheduler_state() -> SchedulerState {
    SCHEDULER_STATE.with(|s| *s.borrow())
}

fn emit_to_callback(status: OrderbookSyncStatus) {
    STATUS_CALLBACK.with(|c| {
        if let Some(callback) = c.borrow().as_ref() {
            if let Ok(value) = serde_wasm_bindgen::to_value(&status) {
                let _ = callback.call1(&JsValue::NULL, &value);
            }
        }
    });
}

#[derive(Debug, Clone, Default)]
pub struct ClientStatusBus {
    ob_id: Option<OrderbookIdentifier>,
}

impl ClientStatusBus {
    pub fn new() -> Self {
        Self { ob_id: None }
    }

    pub fn with_ob_id(ob_id: OrderbookIdentifier) -> Self {
        Self { ob_id: Some(ob_id) }
    }

    fn emit(&self, status: OrderbookSyncStatus) {
        emit_to_callback(status);
    }

    pub fn emit_active(&self) {
        let Some(ob_id) = &self.ob_id else {
            return;
        };

        let scheduler_state = get_scheduler_state();
        self.emit(OrderbookSyncStatus::active(ob_id.clone(), scheduler_state));
    }

    pub fn emit_failure(&self, error: String) {
        let Some(ob_id) = &self.ob_id else {
            return;
        };

        self.emit(OrderbookSyncStatus::failure(ob_id.clone(), error));
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ClientStatusBus {
    async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
        let scheduler_state = get_scheduler_state();
        if scheduler_state == SchedulerState::NotLeader {
            return Ok(());
        }

        let Some(ob_id) = &self.ob_id else {
            return Ok(());
        };

        let status = OrderbookSyncStatus::syncing(ob_id.clone(), phase);
        self.emit(status);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::local_db::LocalDbStatus;
    use alloy::primitives::address;

    fn test_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    #[test]
    fn sync_phase_to_message_returns_correct_strings() {
        assert_eq!(
            SyncPhase::FetchingLatestBlock.to_message(),
            "Fetching latest block"
        );
        assert_eq!(
            SyncPhase::RunningBootstrap.to_message(),
            "Running bootstrap"
        );
        assert_eq!(
            SyncPhase::ComputingSyncWindow.to_message(),
            "Computing sync window"
        );
        assert_eq!(
            SyncPhase::FetchingOrderbookLogs.to_message(),
            "Fetching orderbook logs"
        );
        assert_eq!(
            SyncPhase::DecodingOrderbookLogs.to_message(),
            "Decoding orderbook logs"
        );
        assert_eq!(
            SyncPhase::FetchingStoreLogs.to_message(),
            "Fetching interpreter store logs"
        );
        assert_eq!(
            SyncPhase::DecodingStoreLogs.to_message(),
            "Decoding interpreter store logs"
        );
        assert_eq!(
            SyncPhase::FetchingTokenMetadata.to_message(),
            "Fetching missing token metadata"
        );
        assert_eq!(
            SyncPhase::BuildingSqlBatch.to_message(),
            "Building SQL batch"
        );
        assert_eq!(
            SyncPhase::PersistingToDatabase.to_message(),
            "Persisting to database"
        );
        assert_eq!(
            SyncPhase::RunningPostSyncExport.to_message(),
            "Running post-sync export"
        );
        assert_eq!(SyncPhase::Idle.to_message(), "No work for current window");
    }

    #[test]
    fn client_status_bus_default_has_no_ob_id() {
        let bus = ClientStatusBus::new();
        assert!(bus.ob_id.is_none());
    }

    #[test]
    fn client_status_bus_with_ob_id_stores_identifier() {
        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id.clone());
        assert_eq!(bus.ob_id, Some(ob_id));
    }

    #[tokio::test]
    async fn send_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_skips_when_not_leader() {
        let ob_id = test_ob_id();
        set_scheduler_state(SchedulerState::NotLeader);
        let bus = ClientStatusBus::with_ob_id(ob_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
        set_scheduler_state(SchedulerState::Leader);
    }

    #[tokio::test]
    async fn send_returns_ok_when_leader_with_ob_id() {
        set_scheduler_state(SchedulerState::Leader);
        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[test]
    fn emit_active_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        bus.emit_active();
    }

    #[test]
    fn emit_failure_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        bus.emit_failure("test error".to_string());
    }

    #[test]
    fn emit_active_with_ob_id_does_not_panic() {
        set_scheduler_state(SchedulerState::Leader);
        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id);
        bus.emit_active();
    }

    #[test]
    fn emit_failure_with_ob_id_does_not_panic() {
        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id);
        bus.emit_failure("test error".to_string());
    }

    #[test]
    fn set_and_get_scheduler_state_roundtrips() {
        set_scheduler_state(SchedulerState::Leader);
        assert_eq!(get_scheduler_state(), SchedulerState::Leader);

        set_scheduler_state(SchedulerState::NotLeader);
        assert_eq!(get_scheduler_state(), SchedulerState::NotLeader);

        set_scheduler_state(SchedulerState::Leader);
        assert_eq!(get_scheduler_state(), SchedulerState::Leader);
    }

    #[test]
    fn orderbook_sync_status_syncing_sets_correct_fields() {
        let ob_id = test_ob_id();
        let status = OrderbookSyncStatus::syncing(ob_id.clone(), SyncPhase::FetchingLatestBlock);

        assert_eq!(status.ob_id, ob_id);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(
            status.phase_message,
            Some("Fetching latest block".to_string())
        );
        assert!(status.error.is_none());
    }

    #[test]
    fn orderbook_sync_status_active_with_leader_sets_correct_fields() {
        let ob_id = test_ob_id();
        let status = OrderbookSyncStatus::active(ob_id.clone(), SchedulerState::Leader);

        assert_eq!(status.ob_id, ob_id);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.phase_message.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn orderbook_sync_status_active_with_not_leader_sets_correct_fields() {
        let ob_id = test_ob_id();
        let status = OrderbookSyncStatus::active(ob_id.clone(), SchedulerState::NotLeader);

        assert_eq!(status.ob_id, ob_id);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::NotLeader);
        assert!(status.phase_message.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn orderbook_sync_status_failure_sets_correct_fields() {
        let ob_id = test_ob_id();
        let error_msg = "RPC connection failed".to_string();
        let status = OrderbookSyncStatus::failure(ob_id.clone(), error_msg.clone());

        assert_eq!(status.ob_id, ob_id);
        assert_eq!(status.status, LocalDbStatus::Failure);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.phase_message.is_none());
        assert_eq!(status.error, Some(error_msg));
    }

    #[test]
    fn orderbook_sync_status_new_with_all_fields() {
        let ob_id = test_ob_id();
        let status = OrderbookSyncStatus::new(
            ob_id.clone(),
            LocalDbStatus::Syncing,
            SchedulerState::Leader,
            Some("Custom phase".to_string()),
            Some("Custom error".to_string()),
        );

        assert_eq!(status.ob_id, ob_id);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(status.phase_message, Some("Custom phase".to_string()));
        assert_eq!(status.error, Some("Custom error".to_string()));
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::LocalDbStatus;
    use alloy::primitives::address;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn test_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    fn create_recording_callback(
        recorded: Rc<RefCell<Vec<OrderbookSyncStatus>>>,
    ) -> Rc<js_sys::Function> {
        let closure = Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(status) = serde_wasm_bindgen::from_value::<OrderbookSyncStatus>(value) {
                recorded.borrow_mut().push(status);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let function: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();
        Rc::new(function)
    }

    #[wasm_bindgen_test]
    async fn emit_to_callback_invokes_js_function_with_correct_payload() {
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let callback = create_recording_callback(Rc::clone(&recorded));

        set_status_callback(Some(callback));
        set_scheduler_state(SchedulerState::Leader);

        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id.clone());
        bus.send(SyncPhase::FetchingLatestBlock).await.unwrap();

        set_status_callback(None);

        let emissions = recorded.borrow();
        assert_eq!(emissions.len(), 1, "expected exactly one emission");

        let emitted = &emissions[0];
        assert_eq!(emitted.ob_id, ob_id);
        assert_eq!(emitted.status, LocalDbStatus::Syncing);
        assert_eq!(emitted.scheduler_state, SchedulerState::Leader);
        assert_eq!(
            emitted.phase_message,
            Some("Fetching latest block".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn send_does_not_emit_when_not_leader() {
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let callback = create_recording_callback(Rc::clone(&recorded));

        set_status_callback(Some(callback));
        set_scheduler_state(SchedulerState::NotLeader);

        let ob_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(ob_id);
        bus.send(SyncPhase::FetchingLatestBlock).await.unwrap();

        set_status_callback(None);
        set_scheduler_state(SchedulerState::Leader);

        let emissions = recorded.borrow();
        assert_eq!(
            emissions.len(),
            0,
            "expected no emissions when NotLeader, got {}",
            emissions.len()
        );
    }
}
