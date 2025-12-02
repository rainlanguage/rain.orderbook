use crate::local_db::LocalDbError;
use async_trait::async_trait;

#[cfg(target_family = "wasm")]
use js_sys::{Function, Object, Promise, Reflect};
#[cfg(target_family = "wasm")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;
#[cfg(target_family = "wasm")]
use web_sys::window;

#[cfg(target_family = "wasm")]
const LOCK_NAME: &str = "local-db-sync-engine";

/// Guard that keeps platform-specific leadership state alive for the duration of a run.
pub struct LeadershipGuard {
    #[cfg(target_family = "wasm")]
    release_fn: Option<Function>,
}

impl LeadershipGuard {
    /// Creates a guard that does not perform any release action when dropped.
    pub fn new_noop() -> Self {
        Self {
            #[cfg(target_family = "wasm")]
            release_fn: None,
        }
    }

    #[cfg(target_family = "wasm")]
    fn with_release(release_fn: Function) -> Self {
        Self {
            release_fn: Some(release_fn),
        }
    }
}

impl Drop for LeadershipGuard {
    fn drop(&mut self) {
        #[cfg(target_family = "wasm")]
        if let Some(release_fn) = self.release_fn.take() {
            let _ = release_fn.call0(&JsValue::UNDEFINED);
        }
    }
}

#[async_trait(?Send)]
pub trait Leadership {
    /// Attempts to establish leadership for the current runner invocation.
    ///
    /// Returns `None` when another leader is already active (browser Web Locks only).
    async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError>;
}

#[derive(Clone, Debug)]
pub struct DefaultLeadership;

impl DefaultLeadership {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultLeadership {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl Leadership for DefaultLeadership {
    async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
        #[cfg(target_family = "wasm")]
        {
            match attempt_web_lock().await {
                Ok(Some(guard)) => Ok(Some(guard)),
                Ok(None) => Ok(None),
                Err(_) => Ok(Some(LeadershipGuard::new_noop())),
            }
        }

        #[cfg(not(target_family = "wasm"))]
        {
            Ok(Some(LeadershipGuard::new_noop()))
        }
    }
}

/// Convenience helper for callers that do not yet inject a leadership strategy.
pub async fn acquire() -> Result<Option<LeadershipGuard>, LocalDbError> {
    DefaultLeadership::new().acquire().await
}

#[cfg(target_family = "wasm")]
async fn attempt_web_lock() -> Result<Option<LeadershipGuard>, JsValue> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let window = match window() {
        Some(window) => window,
        None => {
            return Ok(Some(LeadershipGuard::new_noop()));
        }
    };
    let navigator = window.navigator();
    let locks_value = Reflect::get(navigator.as_ref(), &JsValue::from_str("locks"))?;
    if locks_value.is_undefined() || locks_value.is_null() {
        return Ok(Some(LeadershipGuard::new_noop()));
    }

    let request_fn =
        Reflect::get(&locks_value, &JsValue::from_str("request"))?.dyn_into::<Function>()?;

    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("mode"),
        &JsValue::from_str("exclusive"),
    )?;
    Reflect::set(
        &options,
        &JsValue::from_str("ifAvailable"),
        &JsValue::from_bool(true),
    )?;

    let acquired_resolver = Rc::new(RefCell::new(None::<Function>));
    let release_resolver = Rc::new(RefCell::new(None::<Function>));

    let acquired_promise = {
        let acquired_resolver = Rc::clone(&acquired_resolver);
        Promise::new(&mut move |resolve, _| {
            acquired_resolver.borrow_mut().replace(resolve.clone());
        })
    };
    let acquired_future = JsFuture::from(acquired_promise);

    let callback_acquired = Rc::clone(&acquired_resolver);
    let callback_release = Rc::clone(&release_resolver);
    let callback = Closure::wrap(Box::new(move |lock: JsValue| -> JsValue {
        if lock.is_undefined() || lock.is_null() {
            if let Some(resolve) = callback_acquired.borrow_mut().take() {
                let _ = resolve.call1(&JsValue::UNDEFINED, &JsValue::from_bool(false));
            }
            JsValue::UNDEFINED
        } else {
            if let Some(resolve) = callback_acquired.borrow_mut().take() {
                let _ = resolve.call1(&JsValue::UNDEFINED, &JsValue::from_bool(true));
            }
            let release_resolver = Rc::clone(&callback_release);
            Promise::new(&mut move |resolve, _| {
                release_resolver.borrow_mut().replace(resolve.clone());
            })
            .into()
        }
    }) as Box<dyn FnMut(JsValue) -> JsValue>);

    let request_result = request_fn.call3(
        &locks_value,
        &JsValue::from_str(LOCK_NAME),
        &options.into(),
        callback.as_ref().unchecked_ref(),
    );

    if request_result.is_err() {
        // Drop the callback before propagating the error.
        drop(callback);
        return Err(request_result
            .err()
            .unwrap_or_else(|| JsValue::from_str("Web Locks request failed")));
    }

    let acquired_value = acquired_future.await?;
    drop(callback);

    if !acquired_value.as_bool().unwrap_or(false) {
        return Ok(None);
    }

    let release_fn = match release_resolver.borrow_mut().take() {
        Some(function) => function,
        None => {
            return Ok(Some(LeadershipGuard::new_noop()));
        }
    };

    Ok(Some(LeadershipGuard::with_release(release_fn)))
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use futures::executor::block_on;

    #[test]
    fn acquire_returns_guard_in_native_env() {
        let guard = block_on(DefaultLeadership::new().acquire()).expect("acquire succeeds");
        assert!(
            guard.is_some(),
            "guard should be present when Web Locks are unavailable"
        );
    }

    struct DeterministicLeadership {
        grant: bool,
    }

    #[async_trait(?Send)]
    impl Leadership for DeterministicLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            if self.grant {
                Ok(Some(LeadershipGuard::new_noop()))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn stub_leadership_can_simulate_denied_lock() {
        let leadership = DeterministicLeadership { grant: false };
        let guard = block_on(leadership.acquire()).expect("acquire succeeds");
        assert!(
            guard.is_none(),
            "stub leadership should be able to simulate missing guard"
        );
    }

    #[test]
    fn stub_leadership_can_simulate_granted_lock() {
        let leadership = DeterministicLeadership { grant: true };
        let guard = block_on(leadership.acquire()).expect("acquire succeeds");
        assert!(
            guard.is_some(),
            "stub leadership should be able to simulate acquiring leadership"
        );
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use js_sys::{Function, Object, Promise, Reflect};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_test::*;
    use web_sys::window;

    wasm_bindgen_test_configure!(run_in_browser);

    struct LockStub {
        hook: LockHook,
        _request_closure: Closure<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue>,
        release_promise: Rc<RefCell<Option<Promise>>>,
        release_invoked: Rc<Cell<bool>>,
        _release_then: Rc<RefCell<Option<Closure<dyn FnMut(JsValue)>>>>,
    }

    enum LockHook {
        Replace {
            navigator: web_sys::Navigator,
        },
        Patch {
            locks: Object,
            original_request: JsValue,
        },
    }

    impl LockStub {
        fn install(grant_lock: bool) -> Result<Self, JsValue> {
            let window = window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
            let navigator = window.navigator();

            let callback_result = if grant_lock {
                JsValue::from(Object::new())
            } else {
                JsValue::UNDEFINED
            };

            let release_promise = Rc::new(RefCell::new(None::<Promise>));
            let release_invoked = Rc::new(Cell::new(false));
            let release_then = Rc::new(RefCell::new(None::<Closure<dyn FnMut(JsValue)>>));

            let promise_cell = Rc::clone(&release_promise);
            let flag_cell = Rc::clone(&release_invoked);
            let then_cell = Rc::clone(&release_then);

            let request_closure: Closure<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue> =
                Closure::wrap(Box::new(move |_name: JsValue, _options: JsValue, cb: JsValue| -> JsValue {
                    let callback: Function = cb.unchecked_into();
                    let promise_js = match callback.call1(&JsValue::UNDEFINED, &callback_result) {
                        Ok(value) => value,
                        Err(_) => return Promise::resolve(&JsValue::UNDEFINED).into(),
                    };

                    if let Some(promise) = promise_js.dyn_ref::<Promise>() {
                        let promise = promise.clone();
                        promise_cell.borrow_mut().replace(promise.clone());

                        let flag = Rc::clone(&flag_cell);
                        let then_closure: Closure<dyn FnMut(JsValue)> =
                            Closure::wrap(Box::new(move |_value: JsValue| {
                                flag.set(true);
                            })
                                as Box<dyn FnMut(JsValue)>);
                        let _ = promise.then(&then_closure);
                        *then_cell.borrow_mut() = Some(then_closure);

                        promise.into()
                    } else {
                        Promise::resolve(&JsValue::UNDEFINED).into()
                    }
                })
                    as Box<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue>);

            let locks_value = Reflect::get(navigator.as_ref(), &JsValue::from_str("locks"))?;
            let hook = if locks_value.is_undefined() || locks_value.is_null() {
                let locks = Object::new();
                Reflect::set(
                    &locks,
                    &JsValue::from_str("request"),
                    request_closure.as_ref().unchecked_ref(),
                )?;
                Reflect::set(
                    navigator.as_ref(),
                    &JsValue::from_str("locks"),
                    &locks.into(),
                )?;

                LockHook::Replace { navigator }
            } else {
                let locks: Object = locks_value.dyn_into()?;
                let original_request = Reflect::get(&locks, &JsValue::from_str("request"))?;
                Reflect::set(
                    &locks,
                    &JsValue::from_str("request"),
                    request_closure.as_ref().unchecked_ref(),
                )?;

                LockHook::Patch {
                    locks,
                    original_request,
                }
            };

            Ok(Self {
                hook,
                _request_closure: request_closure,
                release_promise,
                release_invoked,
                _release_then: release_then,
            })
        }

        async fn await_release(&self) {
            if let Some(promise) = self.release_promise.borrow().clone() {
                let _ = JsFuture::from(promise).await;
            }
        }

        fn release_called(&self) -> bool {
            self.release_invoked.get()
        }
    }

    impl Drop for LockStub {
        fn drop(&mut self) {
            if let Some(_) = window() {
                match &self.hook {
                    LockHook::Replace { navigator } => {
                        let _ = Reflect::delete_property(
                            navigator.as_ref(),
                            &JsValue::from_str("locks"),
                        );
                    }
                    LockHook::Patch {
                        locks,
                        original_request,
                    } => {
                        if original_request.is_undefined() || original_request.is_null() {
                            let _ = Reflect::delete_property(locks, &JsValue::from_str("request"));
                        } else {
                            let _ = Reflect::set(
                                locks,
                                &JsValue::from_str("request"),
                                original_request,
                            );
                        }
                    }
                }
            }
        }
    }

    #[wasm_bindgen_test(async)]
    async fn acquire_returns_guard_when_lock_granted() {
        let _stub = LockStub::install(true).expect("stub install");
        let guard = DefaultLeadership::new()
            .acquire()
            .await
            .expect("acquire ok");
        assert!(guard.is_some(), "expected guard when lock granted");
    }

    #[wasm_bindgen_test(async)]
    async fn acquire_returns_none_when_lock_denied() {
        let _stub = LockStub::install(false).expect("stub install");
        let guard = DefaultLeadership::new()
            .acquire()
            .await
            .expect("acquire ok");
        assert!(guard.is_none(), "expected None when lock denied");
    }

    #[wasm_bindgen_test(async)]
    async fn guard_drop_invokes_release_callback() {
        let stub = LockStub::install(true).expect("stub install");
        {
            let guard_opt = DefaultLeadership::new()
                .acquire()
                .await
                .expect("acquire ok");
            let guard = guard_opt.expect("expected guard when lock granted");
            drop(guard);
        }
        stub.await_release().await;
        assert!(
            stub.release_called(),
            "expected release callback to run when guard dropped"
        );
    }
}
