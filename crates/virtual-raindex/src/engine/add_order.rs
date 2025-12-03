//! Helpers for executing post-order tasks inside the Virtual Raindex.

use alloy::primitives::U256;
use rain_interpreter_bindings::IInterpreterV4::EvalV4;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, TaskV2};

use crate::{
    cache::CodeCache,
    error::{BytecodeKind, RaindexError, Result},
    host,
    state::{self, StoreKey},
};

use super::{mutations::ensure_vault_entries, VirtualRaindex};
use crate::store::namespace_for_order;

/// Adds an order to state and executes associated post tasks within the same transaction.
pub(super) fn add_order<C, H>(
    raindex: &mut VirtualRaindex<C, H>,
    order: OrderV4,
    post_tasks: Vec<TaskV2>,
) -> Result<()>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    raindex.code_cache.ensure_artifacts(&order)?;
    for task in &post_tasks {
        ensure_task_bytecode(raindex.code_cache.as_ref(), task)?;
    }

    let mut draft = raindex.state.clone();
    ensure_vault_entries(&mut draft, &order);
    draft.apply_mutations(&[state::RaindexMutation::SetOrders {
        orders: vec![order.clone()],
    }])?;

    run_post_tasks(raindex, &mut draft, &order, &post_tasks)?;

    raindex.state = draft;
    Ok(())
}

/// Validates that all bytecode needed to execute a post task is cached.
fn ensure_task_bytecode<C: CodeCache>(code_cache: &C, task: &TaskV2) -> Result<()> {
    let interpreter = task.evaluable.interpreter;
    if code_cache.interpreter(interpreter).is_none() {
        return Err(RaindexError::MissingBytecode {
            address: interpreter,
            kind: BytecodeKind::Interpreter,
        });
    }

    let store = task.evaluable.store;
    if code_cache.store(store).is_none() {
        return Err(RaindexError::MissingBytecode {
            address: store,
            kind: BytecodeKind::Store,
        });
    }

    Ok(())
}

/// Executes post tasks against the provided state snapshot, writing any store updates.
pub(super) fn run_post_tasks<C, H>(
    raindex: &VirtualRaindex<C, H>,
    state: &mut state::RaindexState,
    order: &OrderV4,
    post_tasks: &[TaskV2],
) -> Result<()>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    if post_tasks.is_empty() {
        return Ok(());
    }

    let base_columns = raindex.build_post_context(order);
    let env = state.env;
    let store_namespace = namespace_for_order(order, raindex.orderbook);
    let qualified = store_namespace.qualified;

    for task in post_tasks {
        if task.evaluable.bytecode.is_empty() {
            continue;
        }

        let context = raindex.build_context(base_columns.clone(), &task.signedContext, order.owner);

        let eval = EvalV4 {
            store: task.evaluable.store,
            namespace: store_namespace.namespace,
            bytecode: task.evaluable.bytecode.clone(),
            sourceIndex: U256::ZERO,
            context,
            inputs: Vec::new(),
            stateOverlay: Vec::new(),
        };

        let outcome = raindex
            .interpreter_host
            .eval4(task.evaluable.interpreter, &eval, &state.store, env)
            .map_err(|err| RaindexError::RevmExecution(err.to_string()))?;

        for chunk in outcome.writes.chunks(2) {
            if let [key, value] = chunk {
                state
                    .store
                    .insert(StoreKey::new(task.evaluable.store, qualified, *key), *value);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::StaticCodeCache,
        error::{BytecodeKind, RaindexError, Result},
        host::{self, InterpreterHost},
        state::{self, StoreKey},
        store::{address_to_u256, derive_fqn},
    };
    use alloy::primitives::{Address, Bytes, B256, U256};
    use rain_interpreter_bindings::IInterpreterV4::EvalV4;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, TaskV2, IOV2};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    struct RecordingHost {
        outcome: host::EvalOutcome,
        evals: Mutex<Vec<EvalV4>>,
    }

    impl RecordingHost {
        fn new(outcome: host::EvalOutcome) -> Self {
            Self {
                outcome,
                evals: Mutex::new(Vec::new()),
            }
        }

        fn last_eval(&self) -> Option<EvalV4> {
            self.evals.lock().unwrap().last().cloned()
        }
    }

    impl InterpreterHost for RecordingHost {
        fn eval4(
            &self,
            _interpreter: Address,
            eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            self.evals.lock().unwrap().push(eval.clone());
            Ok(self.outcome.clone())
        }
    }

    #[derive(Default)]
    struct FailingHost;

    impl InterpreterHost for FailingHost {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            Err(RaindexError::Unimplemented("failing test host"))
        }
    }

    fn test_order() -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(0x42),
            evaluable: EvaluableV4 {
                interpreter: Address::repeat_byte(0xAA),
                store: Address::repeat_byte(0xBB),
                bytecode: Bytes::from(vec![0x01]),
            },
            validInputs: vec![IOV2 {
                token: Address::repeat_byte(0x10),
                vaultId: B256::from([0x11; 32]),
            }],
            validOutputs: vec![IOV2 {
                token: Address::repeat_byte(0x20),
                vaultId: B256::from([0x22; 32]),
            }],
            nonce: B256::ZERO,
        }
    }

    fn cache_with_code(order: &OrderV4) -> Arc<StaticCodeCache> {
        let cache = Arc::new(StaticCodeCache::default());
        cache.upsert_interpreter(order.evaluable.interpreter, &[0xAB]);
        cache.upsert_store(order.evaluable.store, &[0xCD]);
        cache
    }

    fn order_task(order: &OrderV4) -> TaskV2 {
        TaskV2 {
            evaluable: order.evaluable.clone(),
            signedContext: Vec::new(),
        }
    }

    #[test]
    fn ensure_task_bytecode_succeeds_when_artifacts_present() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let task = order_task(&order);

        ensure_task_bytecode(cache.as_ref(), &task).expect("artifacts available");
    }

    #[test]
    fn ensure_task_bytecode_errors_when_interpreter_missing() {
        let interpreter = Address::repeat_byte(0x01);
        let store = Address::repeat_byte(0x02);
        let task = TaskV2 {
            evaluable: EvaluableV4 {
                interpreter,
                store,
                bytecode: Bytes::from(vec![0x01]),
            },
            signedContext: Vec::new(),
        };

        let cache = StaticCodeCache::default();
        cache.upsert_store(store, &[0xEE]);

        let err = ensure_task_bytecode(&cache, &task).expect_err("missing interpreter");
        match err {
            RaindexError::MissingBytecode { address, kind } => {
                assert_eq!(address, interpreter);
                assert_matches::assert_matches!(kind, BytecodeKind::Interpreter);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn ensure_task_bytecode_errors_when_store_missing() {
        let interpreter = Address::repeat_byte(0x11);
        let store = Address::repeat_byte(0x22);
        let task = TaskV2 {
            evaluable: EvaluableV4 {
                interpreter,
                store,
                bytecode: Bytes::from(vec![0x01]),
            },
            signedContext: Vec::new(),
        };

        let cache = StaticCodeCache::default();
        cache.upsert_interpreter(interpreter, &[0xFF]);

        let err = ensure_task_bytecode(&cache, &task).expect_err("missing store");
        match err {
            RaindexError::MissingBytecode { address, kind } => {
                assert_eq!(address, store);
                assert_matches::assert_matches!(kind, BytecodeKind::Store);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn run_post_tasks_handles_empty_task_list() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let raindex = VirtualRaindex::new(Address::ZERO, Arc::clone(&cache), Arc::clone(&host));
        let mut state = raindex.state.clone();

        run_post_tasks(&raindex, &mut state, &order, &[]).expect("empty tasks succeed");

        assert!(host.last_eval().is_none());
        assert!(state.store.is_empty());
    }

    #[test]
    fn run_post_tasks_skips_empty_bytecode() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let raindex = VirtualRaindex::new(Address::ZERO, Arc::clone(&cache), Arc::clone(&host));
        let mut state = raindex.state.clone();

        let mut task = order_task(&order);
        task.evaluable.bytecode = Bytes::new();

        run_post_tasks(&raindex, &mut state, &order, &[task]).expect("blank bytecode skipped");

        assert!(host.last_eval().is_none());
        assert!(state.store.is_empty());
    }

    #[test]
    fn run_post_tasks_writes_store_updates() {
        let order = test_order();
        let cache = cache_with_code(&order);

        let key = B256::from(U256::from(5_u64));
        let value = B256::from(U256::from(9_u64));
        let outcome = host::EvalOutcome {
            stack: Vec::new(),
            writes: vec![key, value],
        };

        let host = Arc::new(RecordingHost::new(outcome));
        let raindex = VirtualRaindex::new(Address::ZERO, Arc::clone(&cache), Arc::clone(&host));
        let mut state = raindex.state.clone();

        let task = order_task(&order);

        run_post_tasks(&raindex, &mut state, &order, &[task]).expect("writes succeed");

        let namespace = address_to_u256(order.owner);
        let qualified = derive_fqn(namespace, raindex.orderbook_address());
        let store_key = StoreKey::new(order.evaluable.store, qualified, key);

        assert_eq!(state.store.get(&store_key), Some(&value));
        assert!(host.last_eval().is_some());
    }

    #[test]
    fn run_post_tasks_maps_errors_to_revm_execution() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let host = Arc::new(FailingHost);
        let raindex = VirtualRaindex::new(Address::ZERO, Arc::clone(&cache), host);
        let mut state = raindex.state.clone();

        let task = order_task(&order);

        let err = run_post_tasks(&raindex, &mut state, &order, &[task]).expect_err("host failure");
        match err {
            RaindexError::RevmExecution(message) => {
                assert!(message.contains("failing test host"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
