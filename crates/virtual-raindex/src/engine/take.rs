use std::{
    collections::HashMap,
    ops::{Mul, Neg},
};

use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1, IOV2};

use crate::{
    cache::CodeCache,
    error::{RaindexError, Result},
    host,
    state::{self, RaindexMutation, StoreKey, StoreKeyValue, StoreSet, VaultDelta, VaultKey},
    types::{TakeOrder, TakeOrderWarning, TakeOrdersConfig, TakeOrdersOutcome, TakenOrder},
};

use super::{
    context::{
        namespace_for_order, CONTEXT_CALCULATIONS_COLUMN, CONTEXT_VAULT_INPUTS_COLUMN,
        CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, CONTEXT_VAULT_OUTPUTS_COLUMN, HANDLE_IO_ENTRYPOINT,
    },
    VirtualRaindex,
};

pub(super) fn take_orders<C, H>(
    raindex: &VirtualRaindex<C, H>,
    config: TakeOrdersConfig,
) -> Result<TakeOrdersOutcome>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    if config.orders.is_empty() {
        return Err(RaindexError::NoOrders);
    }

    if config.maximum_input.is_zero()? {
        return Err(RaindexError::ZeroMaximumInput);
    }

    let mut working_state = raindex.state.clone();

    let first_order_ref = &config.orders[0];
    let first_order = raindex.resolve_order(first_order_ref.order.clone())?;
    validate_io_indices(first_order_ref, &first_order)?;

    let expected_input_token = first_order.validInputs[first_order_ref.input_io_index].token;
    let expected_output_token = first_order.validOutputs[first_order_ref.output_io_index].token;

    let mut total_input = Float::default();
    let mut total_output = Float::default();
    let mut remaining_input = config.maximum_input;
    let mut taken = Vec::new();
    let mut warnings = Vec::new();
    let mut vault_deltas = Vec::new();
    let mut store_sets = Vec::new();

    for (index, take_order) in config.orders.iter().enumerate() {
        if !remaining_input.gt(Float::default())? {
            break;
        }

        let resolved_order = if index == 0 {
            first_order.clone()
        } else {
            match raindex.resolve_order(take_order.order.clone()) {
                Ok(order) => order,
                Err(RaindexError::OrderNotFound { order_hash }) => {
                    warnings.push(TakeOrderWarning::OrderNotFound { order_hash });
                    continue;
                }
                Err(other) => return Err(other),
            }
        };

        validate_io_indices(take_order, &resolved_order)?;

        let input_io = &resolved_order.validInputs[take_order.input_io_index];
        let output_io = &resolved_order.validOutputs[take_order.output_io_index];

        if input_io.token == output_io.token {
            return Err(RaindexError::TokenSelfTrade);
        }

        if input_io.token != expected_input_token || output_io.token != expected_output_token {
            return Err(RaindexError::TokenMismatch);
        }

        let store_snapshot = working_state.store.clone();
        let mut calculation = calculate_order_io_for_take(
            raindex,
            &working_state,
            &store_snapshot,
            &resolved_order,
            take_order.input_io_index,
            take_order.output_io_index,
            config.taker,
            &take_order.signed_context,
        )?;

        if calculation.io_ratio.gt(config.maximum_io_ratio)? {
            warnings.push(TakeOrderWarning::RatioExceeded {
                order_hash: state::order_hash(&resolved_order),
            });
            continue;
        }

        if calculation.output_max.is_zero()? {
            warnings.push(TakeOrderWarning::ZeroAmount {
                order_hash: state::order_hash(&resolved_order),
            });
            continue;
        }

        let taker_input = calculation.output_max.min(remaining_input)?;
        if taker_input.is_zero()? {
            continue;
        }

        let taker_output = calculation.io_ratio.mul(taker_input)?;

        total_input = (total_input + taker_input)?;
        total_output = (total_output + taker_output)?;
        remaining_input = (remaining_input - taker_input)?;

        let mut context = calculation.context.clone();
        set_balance_diff_column(
            &mut context[CONTEXT_VAULT_INPUTS_COLUMN - 1],
            taker_output.get_inner(),
        );
        set_balance_diff_column(
            &mut context[CONTEXT_VAULT_OUTPUTS_COLUMN - 1],
            taker_input.get_inner(),
        );
        calculation.context = context.clone();

        apply_vault_updates(
            &mut working_state,
            &resolved_order,
            input_io,
            output_io,
            taker_output,
            taker_input,
            &mut vault_deltas,
        )?;

        if !calculation.calculate_writes.is_empty() {
            store_sets.push(StoreSet {
                store: calculation.store,
                fqn: calculation.qualified_namespace,
                kvs: calculation
                    .calculate_writes
                    .iter()
                    .map(|(key, value)| StoreKeyValue {
                        key: *key,
                        value: *value,
                    })
                    .collect(),
            });
            apply_store_writes(
                &mut working_state.store,
                calculation.store,
                calculation.qualified_namespace,
                &calculation.calculate_writes,
            );
        }

        let store_snapshot_after_calc = working_state.store.clone();
        let handle_outcome = raindex.interpreter_host.eval4(
            calculation.order.evaluable.interpreter,
            &rain_interpreter_bindings::IInterpreterV4::EvalV4 {
                store: calculation.store,
                namespace: calculation.namespace,
                bytecode: calculation.order.evaluable.bytecode.clone(),
                sourceIndex: U256::from(HANDLE_IO_ENTRYPOINT),
                context,
                inputs: Vec::new(),
                stateOverlay: Vec::new(),
            },
            &store_snapshot_after_calc,
            raindex.state.env,
        )?;

        let handle_writes = writes_to_pairs(&handle_outcome.writes)?;
        if !handle_writes.is_empty() {
            store_sets.push(StoreSet {
                store: calculation.store,
                fqn: calculation.qualified_namespace,
                kvs: handle_writes
                    .iter()
                    .map(|(key, value)| StoreKeyValue {
                        key: *key,
                        value: *value,
                    })
                    .collect(),
            });
            apply_store_writes(
                &mut working_state.store,
                calculation.store,
                calculation.qualified_namespace,
                &handle_writes,
            );
        }

        taken.push(TakenOrder {
            order_hash: state::order_hash(&resolved_order),
            input: taker_input,
            output: taker_output,
        });
    }

    if total_input.lt(config.minimum_input)? {
        return Err(RaindexError::MinimumInputNotMet {
            minimum: config.minimum_input,
            actual: total_input,
        });
    }

    let mut mutations = Vec::new();
    if !vault_deltas.is_empty() {
        mutations.push(RaindexMutation::VaultDeltas {
            deltas: vault_deltas.clone(),
        });
    }
    if !store_sets.is_empty() {
        mutations.push(RaindexMutation::ApplyStore {
            sets: store_sets.clone(),
        });
    }

    Ok(TakeOrdersOutcome {
        total_input,
        total_output,
        taken,
        warnings,
        mutations,
    })
}

pub(super) fn take_orders_and_apply_state<C, H>(
    raindex: &mut VirtualRaindex<C, H>,
    config: TakeOrdersConfig,
) -> Result<TakeOrdersOutcome>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let outcome = take_orders(raindex, config.clone())?;
    if !outcome.mutations.is_empty() {
        raindex.apply_mutations(&outcome.mutations)?;
    }
    Ok(outcome)
}

#[allow(clippy::too_many_arguments)]
fn calculate_order_io_for_take<C, H>(
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

    let order_hash = state::order_hash(order);
    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        input_io,
        input_decimals,
        input_balance,
        output_io,
        output_decimals,
        output_balance,
        signed_context,
    );

    run_calculate_io(raindex, store_snapshot, order, context, output_balance)
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
    let (namespace, qualified_namespace) = namespace_for_order(order, raindex.orderbook);

    let eval = rain_interpreter_bindings::IInterpreterV4::EvalV4 {
        store: order.evaluable.store,
        namespace,
        bytecode: order.evaluable.bytecode.clone(),
        sourceIndex: U256::ZERO,
        context: context.clone(),
        inputs: Vec::new(),
        stateOverlay: Vec::new(),
    };

    let outcome = raindex.interpreter_host.eval4(
        order.evaluable.interpreter,
        &eval,
        store_snapshot,
        raindex.state.env,
    )?;

    if outcome.stack.len() < 2 {
        return Err(RaindexError::Unimplemented("calculate-io outputs"));
    }

    let io_ratio = Float::from_raw(outcome.stack[0]);
    let mut output_max = Float::from_raw(outcome.stack[1]);
    output_max = output_max.min(output_balance)?;

    let mut context = context;
    context[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![output_max.get_inner(), io_ratio.get_inner()];

    let calculate_writes = writes_to_pairs(&outcome.writes)?;

    Ok(OrderCalculation {
        order: order.clone(),
        io_ratio,
        output_max,
        context,
        namespace,
        qualified_namespace,
        store: order.evaluable.store,
        calculate_writes,
    })
}

fn apply_vault_updates(
    working_state: &mut state::RaindexState,
    order: &OrderV4,
    input_io: &IOV2,
    output_io: &IOV2,
    taker_output: Float,
    taker_input: Float,
    vault_deltas: &mut Vec<VaultDelta>,
) -> Result<()> {
    let input_key = VaultKey::new(order.owner, input_io.token, input_io.vaultId);
    let input_balance = *working_state.vault_balances.entry(input_key).or_default();
    let new_input_balance = (input_balance + taker_output)?;
    working_state
        .vault_balances
        .insert(input_key, new_input_balance);

    let negative_taker_input = taker_input.neg()?;
    let output_key = VaultKey::new(order.owner, output_io.token, output_io.vaultId);
    let output_balance = *working_state.vault_balances.entry(output_key).or_default();
    let new_output_balance = (output_balance + negative_taker_input)?;
    working_state
        .vault_balances
        .insert(output_key, new_output_balance);

    vault_deltas.push(VaultDelta {
        owner: order.owner,
        token: input_io.token,
        vault_id: input_io.vaultId,
        delta: taker_output,
    });
    vault_deltas.push(VaultDelta {
        owner: order.owner,
        token: output_io.token,
        vault_id: output_io.vaultId,
        delta: negative_taker_input,
    });

    Ok(())
}

fn validate_io_indices(take_order: &TakeOrder, order: &OrderV4) -> Result<()> {
    if take_order.input_io_index >= order.validInputs.len() {
        return Err(RaindexError::InvalidInputIndex {
            index: take_order.input_io_index,
            len: order.validInputs.len(),
        });
    }
    if take_order.output_io_index >= order.validOutputs.len() {
        return Err(RaindexError::InvalidOutputIndex {
            index: take_order.output_io_index,
            len: order.validOutputs.len(),
        });
    }
    Ok(())
}

pub(super) fn apply_store_writes(
    store: &mut HashMap<StoreKey, B256>,
    store_address: Address,
    qualified: B256,
    writes: &[(B256, B256)],
) {
    for (key, value) in writes {
        store.insert(StoreKey::new(store_address, qualified, *key), *value);
    }
}

pub(super) fn writes_to_pairs(writes: &[B256]) -> Result<Vec<(B256, B256)>> {
    if writes.len() % 2 != 0 {
        return Err(RaindexError::Unimplemented(
            "unpaired store write from interpreter",
        ));
    }

    Ok(writes.chunks(2).map(|chunk| (chunk[0], chunk[1])).collect())
}

pub(super) fn set_balance_diff_column(column: &mut Vec<B256>, value: B256) {
    if column.len() < CONTEXT_VAULT_IO_BALANCE_DIFF_ROW {
        column.resize(CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, B256::ZERO);
    }
    column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1] = value;
}

#[derive(Clone)]
struct OrderCalculation {
    order: OrderV4,
    io_ratio: Float,
    output_max: Float,
    context: Vec<Vec<B256>>,
    namespace: U256,
    qualified_namespace: B256,
    store: Address,
    calculate_writes: Vec<(B256, B256)>,
}
