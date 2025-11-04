//! Shared helpers for running calculate-io style interpreter entrypoints.

use std::collections::HashMap;

use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1};
use rain_orderbook_common::utils::order_hash;

use crate::{
    cache::CodeCache,
    error::{RaindexError, Result},
    host,
    state::{self, StoreKey, VaultKey},
    store::{namespace_for_order, StoreNamespace},
};

use super::{
    context::{IOContext, CONTEXT_CALCULATIONS_COLUMN},
    eval::{build_eval_call, EvalEntrypoint},
    VirtualRaindex,
};

/// Output of a calculate-io interpreter call.
#[derive(Clone)]
pub(super) struct OrderCalculation {
    pub(super) order: OrderV4,
    pub(super) io_ratio: Float,
    pub(super) output_max: Float,
    pub(super) context: Vec<Vec<B256>>,
    pub(super) stack: Vec<B256>,
    pub(super) store_writes: Vec<(B256, B256)>,
    pub(super) namespace: U256,
    pub(super) qualified_namespace: B256,
    pub(super) store: Address,
}

#[allow(clippy::too_many_arguments)]
/// Executes calculate-io using the provided state snapshot and returns the interpreter outcome.
pub(super) fn calculate_order_io<C, H>(
    raindex: &VirtualRaindex<C, H>,
    working_state: &state::RaindexState,
    store_snapshot: &HashMap<StoreKey, B256>,
    order: &OrderV4,
    input_io_index: usize,
    output_io_index: usize,
    counterparty: Address,
    signed_context: &[SignedContextV1],
) -> Result<OrderCalculation>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    raindex.code_cache.ensure_artifacts(order)?;

    let input_io = &order.validInputs[input_io_index];
    let output_io = &order.validOutputs[output_io_index];

    let input_decimals = *working_state.token_decimals.get(&input_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: input_io.token,
        },
    )?;
    let output_decimals = *working_state.token_decimals.get(&output_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: output_io.token,
        },
    )?;

    let input_balance = working_state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            input_io.token,
            input_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();
    let output_balance = working_state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            output_io.token,
            output_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();

    let order_hash = order_hash(order);
    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        &IOContext {
            io: input_io.clone(),
            balance: input_balance,
            decimals: input_decimals,
        },
        &IOContext {
            io: output_io.clone(),
            balance: output_balance,
            decimals: output_decimals,
        },
        signed_context,
    );

    run_calculate_io(
        raindex,
        store_snapshot,
        order,
        context,
        output_balance,
    )
}

fn run_calculate_io<C, H>(
    raindex: &VirtualRaindex<C, H>,
    store_snapshot: &HashMap<StoreKey, B256>,
    order: &OrderV4,
    context: Vec<Vec<B256>>,
    output_balance: Float,
) -> Result<OrderCalculation>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let StoreNamespace {
        namespace,
        qualified: qualified_namespace,
    } = namespace_for_order(order, raindex.orderbook);

    let eval = build_eval_call(
        order,
        namespace,
        context.clone(),
        EvalEntrypoint::CalculateIo,
    );

    let outcome = raindex.interpreter_host.eval4(
        order.evaluable.interpreter,
        &eval,
        store_snapshot,
        raindex.state.env,
    )?;

    if outcome.stack.len() < 2 {
        return Err(RaindexError::Unimplemented("calculate-io outputs"));
    }

    let mut stack = outcome.stack;
    let io_ratio = Float::from_raw(stack[0]);
    let mut output_max = Float::from_raw(stack[1]);
    output_max = output_max.min(output_balance)?;
    stack[1] = output_max.get_inner();

    let mut context = context;
    context[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![output_max.get_inner(), io_ratio.get_inner()];

    let store_writes = crate::store::writes_to_pairs(&outcome.writes)?;

    Ok(OrderCalculation {
        order: order.clone(),
        io_ratio,
        output_max,
        context,
        stack,
        store_writes,
        namespace,
        qualified_namespace,
        store: order.evaluable.store,
    })
}
