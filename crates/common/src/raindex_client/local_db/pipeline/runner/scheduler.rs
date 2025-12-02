use super::leadership::DefaultLeadership;
use super::ClientRunner;
use crate::local_db::pipeline::adapters::{
    apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
    window::DefaultWindowPipeline,
};
use crate::local_db::LocalDbError;
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;
use crate::raindex_client::local_db::{LocalDb, LocalDbStatusSnapshot};
use futures::channel::oneshot;
use gloo_timers::future::TimeoutFuture;
use js_sys::Function;
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::spawn_local;
use wasm_bindgen_utils::prelude::*;

const DEFAULT_INTERVAL_MS: u32 = 10_000;

type DefaultClientRunner = ClientRunner<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ClientStatusBus,
    DefaultLeadership,
>;

trait SchedulerRunner {
    fn run_once<'a>(
        &'a mut self,
        db_executor: &'a LocalDb,
    ) -> Pin<Box<dyn Future<Output = Result<(), LocalDbError>> + 'a>>;
}

impl SchedulerRunner for DefaultClientRunner {
    fn run_once<'a>(
        &'a mut self,
        db_executor: &'a LocalDb,
    ) -> Pin<Box<dyn Future<Output = Result<(), LocalDbError>> + 'a>> {
        Box::pin(async move { self.run(db_executor).await.map(|_| ()) })
    }
}

#[derive(Debug)]
pub struct SchedulerHandle {
    stop_flag: Rc<Cell<bool>>,
    done: Option<oneshot::Receiver<()>>,
}

impl SchedulerHandle {
    pub async fn stop(mut self) {
        self.stop_flag.set(true);
        if let Some(done) = self.done.take() {
            let _ = done.await;
        }
    }
}

pub(crate) fn start(
    settings_yaml: String,
    db: LocalDb,
    status_callback: Option<Function>,
) -> Result<SchedulerHandle, LocalDbError> {
    let runner = ClientRunner::new(settings_yaml)?;
    Ok(start_with_runner(
        runner,
        db,
        status_callback,
        DEFAULT_INTERVAL_MS,
    ))
}

fn start_with_runner<R>(
    runner: R,
    db: LocalDb,
    status_callback: Option<Function>,
    interval_ms: u32,
) -> SchedulerHandle
where
    R: SchedulerRunner + 'static,
{
    let stop_flag = Rc::new(Cell::new(false));
    let (done_tx, done_rx) = oneshot::channel();

    let stop_flag_task = Rc::clone(&stop_flag);
    let status_callback = status_callback.map(Rc::new);

    spawn_local(async move {
        let mut runner = runner;
        emit_status(status_callback.as_deref(), LocalDbStatusSnapshot::active());
        loop {
            if stop_flag_task.get() {
                break;
            }

            emit_status(status_callback.as_deref(), LocalDbStatusSnapshot::syncing());
            match runner.run_once(&db).await {
                Ok(_) => emit_status(status_callback.as_deref(), LocalDbStatusSnapshot::active()),
                Err(err) => emit_status(
                    status_callback.as_deref(),
                    LocalDbStatusSnapshot::failure(err.to_readable_msg()),
                ),
            }

            if stop_flag_task.get() {
                break;
            }

            TimeoutFuture::new(interval_ms).await;
        }

        let _ = done_tx.send(());
    });

    SchedulerHandle {
        stop_flag,
        done: Some(done_rx),
    }
}

fn emit_status(callback: Option<&Function>, status: LocalDbStatusSnapshot) {
    if let Some(callback) = callback {
        if let Ok(value) = serde_wasm_bindgen::to_value(&status) {
            let _ = callback.call1(&JsValue::NULL, &value);
        }
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::LocalDbStatus;
    use gloo_timers::future::TimeoutFuture;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn noop_callback() -> Function {
        Function::new_no_args("return undefined;")
    }

    fn noop_local_db() -> LocalDb {
        LocalDb::from_js_callback(noop_callback())
    }

    impl SchedulerHandle {
        pub(crate) fn stop_flag_ptr(&self) -> *const Cell<bool> {
            Rc::as_ptr(&self.stop_flag)
        }
    }

    struct RecordingRunner {
        calls: Rc<Cell<usize>>,
        failures: Rc<Cell<usize>>,
        outcomes: Rc<RefCell<VecDeque<bool>>>,
    }

    impl RecordingRunner {
        fn new(calls: Rc<Cell<usize>>, failures: Rc<Cell<usize>>, outcomes: Vec<bool>) -> Self {
            Self {
                calls,
                failures,
                outcomes: Rc::new(RefCell::new(VecDeque::from(outcomes))),
            }
        }
    }

    impl SchedulerRunner for RecordingRunner {
        fn run_once<'a>(
            &'a mut self,
            _db_executor: &'a LocalDb,
        ) -> Pin<Box<dyn Future<Output = Result<(), LocalDbError>> + 'a>> {
            let calls = Rc::clone(&self.calls);
            let failures = Rc::clone(&self.failures);
            let outcomes = Rc::clone(&self.outcomes);

            Box::pin(async move {
                calls.set(calls.get() + 1);
                let should_fail = outcomes.borrow_mut().pop_front().unwrap_or(false);
                if should_fail {
                    failures.set(failures.get() + 1);
                    Err(LocalDbError::CustomError("runner failure".to_string()))
                } else {
                    Ok(())
                }
            })
        }
    }

    #[wasm_bindgen_test]
    async fn start_returns_error_for_invalid_yaml() {
        let result = start("not yaml".to_string(), noop_local_db(), None);
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn scheduler_runs_until_stopped() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(Rc::clone(&calls), Rc::clone(&failures), vec![]);
        let handle = start_with_runner(runner, noop_local_db(), None, 1);

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(3).await;

        handle.stop().await;

        assert!(calls.get() >= 1);
        assert_eq!(failures.get(), 0);
    }

    #[wasm_bindgen_test]
    async fn scheduler_continues_after_runner_error() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner =
            RecordingRunner::new(Rc::clone(&calls), Rc::clone(&failures), vec![true, false]);
        let handle = start_with_runner(runner, noop_local_db(), None, 1);

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        handle.stop().await;

        assert!(failures.get() >= 1);
        assert!(calls.get() >= 2);
    }

    #[wasm_bindgen_test]
    async fn scheduler_stop_before_run_prevents_execution() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(Rc::clone(&calls), Rc::clone(&failures), vec![]);
        let handle = start_with_runner(runner, noop_local_db(), None, 1);

        handle.stop().await;

        assert_eq!(calls.get(), 0);
        assert_eq!(failures.get(), 0);
    }

    #[wasm_bindgen_test]
    async fn scheduler_emits_status_transitions() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner =
            RecordingRunner::new(Rc::clone(&calls), Rc::clone(&failures), vec![true, false]);

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = {
            let statuses = Rc::clone(&statuses);
            let closure = Closure::wrap(Box::new(move |value: JsValue| {
                let snapshot: LocalDbStatusSnapshot =
                    serde_wasm_bindgen::from_value(value).expect("valid status value");
                statuses.borrow_mut().push(snapshot);
            }) as Box<dyn FnMut(JsValue)>);
            let function: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            function
        };

        let handle = start_with_runner(runner, noop_local_db(), Some(status_callback), 1);

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        handle.stop().await;

        let recorded = statuses.borrow();
        assert!(
            recorded
                .iter()
                .any(|snapshot| snapshot.status == LocalDbStatus::Failure),
            "expected failure status"
        );
        assert!(
            recorded
                .iter()
                .filter(|snapshot| snapshot.status == LocalDbStatus::Active)
                .count()
                >= 2,
            "expected Active status at least twice"
        );
        assert!(
            recorded
                .iter()
                .any(|snapshot| snapshot.status == LocalDbStatus::Syncing),
            "expected Syncing status"
        );
    }
}
