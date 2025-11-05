use super::ClientRunner;
use crate::local_db::LocalDbError;
use crate::raindex_client::local_db::executor::JsCallbackExecutor;
use futures::channel::oneshot;
use gloo_timers::future::TimeoutFuture;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_utils::prelude::js_sys::Function;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::spawn_local;

const DEFAULT_INTERVAL_MS: u32 = 10_000;

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

pub fn start(
    settings_yaml: String,
    db_callback: Function,
) -> Result<SchedulerHandle, LocalDbError> {
    let runner = ClientRunner::new(settings_yaml)?;
    let stop_flag = Rc::new(Cell::new(false));
    let (done_tx, done_rx) = oneshot::channel();

    let stop_flag_task = Rc::clone(&stop_flag);
    spawn_local(async move {
        let mut runner = runner;
        let db_executor = JsCallbackExecutor::new(db_callback);
        loop {
            if stop_flag_task.get() {
                break;
            }

            let _ = runner.run(&db_executor).await;

            if stop_flag_task.get() {
                break;
            }

            TimeoutFuture::new(DEFAULT_INTERVAL_MS).await;
        }

        let _ = done_tx.send(());
    });

    Ok(SchedulerHandle {
        stop_flag,
        done: Some(done_rx),
    })
}
