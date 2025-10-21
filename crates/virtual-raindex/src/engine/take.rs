//! Take order execution flow for the Virtual Raindex.

use std::{
    collections::HashMap,
    ops::{Mul, Neg},
};

use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1, IOV2};
use rain_orderbook_common::utils::order_hash;

use crate::{
    cache::CodeCache,
    error::{RaindexError, Result},
    host,
    state::{self, RaindexMutation, StoreKey, StoreKeyValue, StoreSet, VaultDelta, VaultKey},
};

use super::{
    context::{
        namespace_for_order, IOContext, CONTEXT_CALCULATIONS_COLUMN, CONTEXT_VAULT_INPUTS_COLUMN,
        CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, CONTEXT_VAULT_OUTPUTS_COLUMN, HANDLE_IO_ENTRYPOINT,
    },
    VirtualRaindex,
};

use super::OrderRef;

/// Configuration for taking a specific order instance.
#[derive(Clone, Debug)]
pub struct TakeOrder {
    pub order: OrderRef,
    pub input_io_index: usize,
    pub output_io_index: usize,
    pub signed_context: Vec<SignedContextV1>,
}

/// Payload describing a take orders batch execution.
#[derive(Clone, Debug)]
pub struct TakeOrdersConfig {
    pub orders: Vec<TakeOrder>,
    pub minimum_input: Float,
    pub maximum_input: Float,
    /// From the perspective of the taker.
    pub maximum_io_ratio: Float,
    pub taker: Address,
    pub data: Vec<u8>,
}

/// Non-fatal issues encountered while attempting to take orders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TakeOrderWarning {
    OrderNotFound { order_hash: B256 },
    RatioExceeded { order_hash: B256 },
    ZeroAmount { order_hash: B256 },
}

/// Individual order execution result.
#[derive(Clone, Debug)]
pub struct TakenOrder {
    pub order_hash: B256,
    pub input: Float,
    pub output: Float,
}

/// Aggregate result returned by `take_orders` operations.
#[derive(Clone, Debug)]
pub struct TakeOrdersOutcome {
    pub total_input: Float,
    pub total_output: Float,
    pub taken: Vec<TakenOrder>,
    pub warnings: Vec<TakeOrderWarning>,
    pub mutations: Vec<RaindexMutation>,
}

/// Internal representation of calculate-io results for an order.
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

/// Simulates taking orders against the virtual state and returns the computed outcome.
pub(super) fn take_orders<C, H>(
    raindex: &VirtualRaindex<C, H>,
    config: TakeOrdersConfig,
) -> Result<TakeOrdersOutcome>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let mut processor = TakeOrdersProcessor::new(raindex, config)?;
    processor.process_orders()?;
    processor.ensure_minimum_input()?;
    Ok(processor.into_outcome())
}

/// Runs [`take_orders`] and applies the resulting mutations to the live state.
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

struct TakeOrdersProcessor<'a, C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    raindex: &'a VirtualRaindex<C, H>,
    config: TakeOrdersConfig,
    working_state: state::RaindexState,
    first_order: OrderV4,
    expected_input_token: Address,
    expected_output_token: Address,
    total_input: Float,
    total_output: Float,
    remaining_input: Float,
    taken: Vec<TakenOrder>,
    warnings: Vec<TakeOrderWarning>,
    vault_deltas: Vec<VaultDelta>,
    store_sets: Vec<StoreSet>,
}

impl<'a, C, H> TakeOrdersProcessor<'a, C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    fn new(raindex: &'a VirtualRaindex<C, H>, config: TakeOrdersConfig) -> Result<Self> {
        if config.orders.is_empty() {
            return Err(RaindexError::NoOrders);
        }

        if config.maximum_input.is_zero()? {
            return Err(RaindexError::ZeroMaximumInput);
        }

        let working_state = raindex.state.clone();
        let first_order_ref = &config.orders[0];
        let first_order = raindex.resolve_order(first_order_ref.order.clone())?;
        validate_io_indices(first_order_ref, &first_order)?;

        let expected_input_token = first_order.validInputs[first_order_ref.input_io_index].token;
        let expected_output_token = first_order.validOutputs[first_order_ref.output_io_index].token;

        Ok(Self {
            raindex,
            working_state,
            expected_input_token,
            expected_output_token,
            total_input: Float::default(),
            total_output: Float::default(),
            remaining_input: config.maximum_input.clone(),
            taken: Vec::new(),
            warnings: Vec::new(),
            vault_deltas: Vec::new(),
            store_sets: Vec::new(),
            first_order,
            config,
        })
    }

    fn process_orders(&mut self) -> Result<()> {
        for index in 0..self.config.orders.len() {
            if !self.remaining_input.gt(Float::default())? {
                break;
            }
            let take_order = self.config.orders[index].clone();
            self.process_order(index, &take_order)?;
        }
        Ok(())
    }

    fn process_order(&mut self, index: usize, take_order: &TakeOrder) -> Result<()> {
        let Some(resolved_order) = self.resolve_order(index, take_order)? else {
            return Ok(());
        };

        validate_io_indices(take_order, &resolved_order)?;
        self.validate_tokens(&resolved_order, take_order)?;
        self.execute_order(&resolved_order, take_order)
    }

    fn resolve_order(&mut self, index: usize, take_order: &TakeOrder) -> Result<Option<OrderV4>> {
        if index == 0 {
            return Ok(Some(self.first_order.clone()));
        }

        match self.raindex.resolve_order(take_order.order.clone()) {
            Ok(order) => Ok(Some(order)),
            Err(RaindexError::OrderNotFound { order_hash }) => {
                self.warnings
                    .push(TakeOrderWarning::OrderNotFound { order_hash });
                Ok(None)
            }
            Err(other) => Err(other),
        }
    }

    fn validate_tokens(&self, order: &OrderV4, take_order: &TakeOrder) -> Result<()> {
        let input_io = &order.validInputs[take_order.input_io_index];
        let output_io = &order.validOutputs[take_order.output_io_index];

        if input_io.token == output_io.token {
            return Err(RaindexError::TokenSelfTrade);
        }

        if input_io.token != self.expected_input_token
            || output_io.token != self.expected_output_token
        {
            return Err(RaindexError::TokenMismatch);
        }

        Ok(())
    }

    fn execute_order(&mut self, resolved_order: &OrderV4, take_order: &TakeOrder) -> Result<()> {
        let store_snapshot = self.working_state.store.clone();
        let calculation = calculate_order_io_for_take(
            self.raindex,
            &self.working_state,
            &store_snapshot,
            resolved_order,
            take_order.input_io_index,
            take_order.output_io_index,
            self.config.taker,
            &take_order.signed_context,
        )?;

        if calculation
            .io_ratio
            .gt(self.config.maximum_io_ratio.clone())?
        {
            self.warnings.push(TakeOrderWarning::RatioExceeded {
                order_hash: order_hash(resolved_order),
            });
            return Ok(());
        }

        if calculation.output_max.is_zero()? {
            self.warnings.push(TakeOrderWarning::ZeroAmount {
                order_hash: order_hash(resolved_order),
            });
            return Ok(());
        }

        let taker_input = calculation
            .output_max
            .clone()
            .min(self.remaining_input.clone())?;
        if taker_input.is_zero()? {
            return Ok(());
        }

        self.apply_calculation_results(calculation, resolved_order, take_order, taker_input)
    }

    fn apply_calculation_results(
        &mut self,
        mut calculation: OrderCalculation,
        resolved_order: &OrderV4,
        take_order: &TakeOrder,
        taker_input: Float,
    ) -> Result<()> {
        let taker_output = calculation.io_ratio.clone().mul(taker_input.clone())?;

        self.total_input = (self.total_input + taker_input.clone())?;
        self.total_output = (self.total_output + taker_output.clone())?;
        self.remaining_input = (self.remaining_input - taker_input.clone())?;

        let input_io = &resolved_order.validInputs[take_order.input_io_index];
        let output_io = &resolved_order.validOutputs[take_order.output_io_index];

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
            &mut self.working_state,
            resolved_order,
            input_io,
            output_io,
            taker_output.clone(),
            taker_input.clone(),
            &mut self.vault_deltas,
        )?;

        if !calculation.calculate_writes.is_empty() {
            self.record_store_set(
                calculation.store,
                calculation.qualified_namespace,
                &calculation.calculate_writes,
            );
            apply_store_writes(
                &mut self.working_state.store,
                calculation.store,
                calculation.qualified_namespace,
                &calculation.calculate_writes,
            );
        }

        let store_snapshot_after_calc = self.working_state.store.clone();
        let handle_writes =
            self.run_handle_io(&calculation, context, &store_snapshot_after_calc)?;
        if !handle_writes.is_empty() {
            self.record_store_set(
                calculation.store,
                calculation.qualified_namespace,
                &handle_writes,
            );
            apply_store_writes(
                &mut self.working_state.store,
                calculation.store,
                calculation.qualified_namespace,
                &handle_writes,
            );
        }

        self.taken.push(TakenOrder {
            order_hash: order_hash(resolved_order),
            input: taker_input,
            output: taker_output,
        });

        Ok(())
    }

    fn ensure_minimum_input(&self) -> Result<()> {
        if self.total_input.lt(self.config.minimum_input.clone())? {
            return Err(RaindexError::MinimumInputNotMet {
                minimum: self.config.minimum_input.clone(),
                actual: self.total_input.clone(),
            });
        }
        Ok(())
    }

    fn run_handle_io(
        &self,
        calculation: &OrderCalculation,
        context: Vec<Vec<B256>>,
        store_snapshot: &HashMap<StoreKey, B256>,
    ) -> Result<Vec<(B256, B256)>> {
        let handle_outcome = self.raindex.interpreter_host.eval4(
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
            store_snapshot,
            self.raindex.state.env,
        )?;

        writes_to_pairs(&handle_outcome.writes)
    }

    fn record_store_set(
        &mut self,
        store: Address,
        qualified_namespace: B256,
        writes: &[(B256, B256)],
    ) {
        self.store_sets.push(StoreSet {
            store,
            fqn: qualified_namespace,
            kvs: writes
                .iter()
                .map(|(key, value)| StoreKeyValue {
                    key: *key,
                    value: *value,
                })
                .collect(),
        });
    }

    fn into_outcome(self) -> TakeOrdersOutcome {
        let mut mutations = Vec::new();
        if !self.vault_deltas.is_empty() {
            mutations.push(RaindexMutation::VaultDeltas {
                deltas: self.vault_deltas,
            });
        }
        if !self.store_sets.is_empty() {
            mutations.push(RaindexMutation::ApplyStore {
                sets: self.store_sets,
            });
        }

        TakeOrdersOutcome {
            total_input: self.total_input,
            total_output: self.total_output,
            taken: self.taken,
            warnings: self.warnings,
            mutations,
        }
    }
}

#[allow(clippy::too_many_arguments)]
/// Executes calculate-io for a specific IO pair and returns the interpreter result.
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

    run_calculate_io(raindex, store_snapshot, order, context, output_balance)
}

/// Runs the interpreter calculate entrypoint and captures its outputs and writes.
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

/// Applies the taker input/output adjustments to the working state and records deltas.
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

/// Validates that supplied IO indices are within bounds for the order.
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

/// Applies store writes produced by interpreter executions onto the snapshot.
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

/// Converts a flat write buffer into key/value pairs, ensuring even length.
pub(super) fn writes_to_pairs(writes: &[B256]) -> Result<Vec<(B256, B256)>> {
    if writes.len() % 2 != 0 {
        return Err(RaindexError::Unimplemented(
            "unpaired store write from interpreter",
        ));
    }

    Ok(writes.chunks(2).map(|chunk| (chunk[0], chunk[1])).collect())
}

/// Sets the balance diff entry for a context column, resizing if needed.
pub(super) fn set_balance_diff_column(column: &mut Vec<B256>, value: B256) {
    if column.len() < CONTEXT_VAULT_IO_BALANCE_DIFF_ROW {
        column.resize(CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, B256::ZERO);
    }
    column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1] = value;
}
