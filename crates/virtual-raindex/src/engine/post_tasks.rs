use alloy::primitives::U256;
use rain_interpreter_bindings::IInterpreterV4::EvalV4;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, TaskV2};

use crate::{
    cache::CodeCache,
    error::{BytecodeKind, RaindexError, Result},
    host,
    state::{self, StoreKey},
};

use super::{context::namespace_for_order, mutations::ensure_vault_entries, VirtualRaindex};

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
    let (_, qualified) = namespace_for_order(order, raindex.orderbook);

    for task in post_tasks {
        if task.evaluable.bytecode.is_empty() {
            continue;
        }

        let context = raindex.build_context(base_columns.clone(), &task.signedContext, order.owner);

        let eval = EvalV4 {
            store: task.evaluable.store,
            namespace: U256::from_be_slice(qualified.as_slice()),
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
