//! Take order execution flow for the Virtual Raindex.

use std::{
    collections::HashMap,
    ops::{Mul, Neg},
};

use alloy::primitives::{Address, B256};
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
    calc::{calculate_order_io, OrderCalculation},
    context::{
        CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_DIFF_ROW,
        CONTEXT_VAULT_OUTPUTS_COLUMN,
    },
    eval::{build_eval_call, EvalEntrypoint},
    VirtualRaindex,
};

use crate::store::{apply_store_writes, writes_to_pairs};

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
        let calculation = calculate_order_io(
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

        if !calculation.store_writes.is_empty() {
            self.record_store_set(
                calculation.store,
                calculation.qualified_namespace,
                &calculation.store_writes,
            );
            apply_store_writes(
                &mut self.working_state.store,
                calculation.store,
                calculation.qualified_namespace,
                &calculation.store_writes,
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
        let eval = build_eval_call(
            &calculation.order,
            calculation.namespace,
            context,
            EvalEntrypoint::HandleIo,
        );

        let handle_outcome = self.raindex.interpreter_host.eval4(
            calculation.order.evaluable.interpreter,
            &eval,
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

/// Sets the balance diff entry for a context column, resizing if needed.
pub(super) fn set_balance_diff_column(column: &mut Vec<B256>, value: B256) {
    if column.len() < CONTEXT_VAULT_IO_BALANCE_DIFF_ROW {
        column.resize(CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, B256::ZERO);
    }
    column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1] = value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CodeCache,
        error::{RaindexError, Result},
        host::{self, InterpreterHost},
        state::{self, StoreKey, VaultKey},
    };
    use alloy::primitives::{Address, Bytes, B256, U256};
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
    use std::{collections::HashMap, ops::Neg, sync::Arc};

    #[derive(Default)]
    struct NullCache;

    impl CodeCache for NullCache {
        fn interpreter(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
            None
        }

        fn store(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
            None
        }

        fn ensure_artifacts(&self, _order: &OrderV4) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct NullHost;

    impl InterpreterHost for NullHost {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &rain_interpreter_bindings::IInterpreterV4::EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            Err(RaindexError::Unimplemented("test interpreter host"))
        }
    }

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_string()).expect("float parse")
    }

    fn test_raindex() -> VirtualRaindex<NullCache, NullHost> {
        VirtualRaindex::new(
            Address::repeat_byte(0xAB),
            Arc::new(NullCache::default()),
            Arc::new(NullHost::default()),
        )
    }

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(0x44),
            evaluable: EvaluableV4 {
                interpreter: Address::repeat_byte(0xAA),
                store: Address::repeat_byte(0xBB),
                bytecode: Bytes::from(vec![0u8]),
            },
            validInputs: vec![IOV2 {
                token: Address::repeat_byte(0x10),
                vaultId: B256::from([1u8; 32]),
            }],
            validOutputs: vec![IOV2 {
                token: Address::repeat_byte(0x20),
                vaultId: B256::from([2u8; 32]),
            }],
            nonce: B256::ZERO,
        }
    }

    fn make_take_order(order: OrderRef) -> TakeOrder {
        TakeOrder {
            order,
            input_io_index: 0,
            output_io_index: 0,
            signed_context: Vec::new(),
        }
    }

    #[test]
    fn take_orders_processor_new_errors_without_orders() {
        let raindex = test_raindex();
        let config = TakeOrdersConfig {
            orders: Vec::new(),
            minimum_input: parse_float("0"),
            maximum_input: parse_float("1"),
            maximum_io_ratio: parse_float("1"),
            taker: Address::ZERO,
            data: Vec::new(),
        };

        let result = TakeOrdersProcessor::new(&raindex, config);

        assert!(matches!(result, Err(RaindexError::NoOrders)));
    }

    #[test]
    fn take_orders_processor_new_errors_with_zero_max_input() {
        let raindex = test_raindex();
        let order = sample_order();
        let config = TakeOrdersConfig {
            orders: vec![make_take_order(OrderRef::Inline(order))],
            minimum_input: parse_float("0"),
            maximum_input: Float::default(),
            maximum_io_ratio: parse_float("1"),
            taker: Address::ZERO,
            data: Vec::new(),
        };

        let result = TakeOrdersProcessor::new(&raindex, config);

        assert!(matches!(result, Err(RaindexError::ZeroMaximumInput)));
    }

    #[test]
    fn take_orders_processor_new_initializes_expected_tokens() {
        let raindex = test_raindex();
        let order = sample_order();
        let config = TakeOrdersConfig {
            orders: vec![make_take_order(OrderRef::Inline(order.clone()))],
            minimum_input: parse_float("0"),
            maximum_input: parse_float("10"),
            maximum_io_ratio: parse_float("1"),
            taker: Address::repeat_byte(0x55),
            data: Vec::new(),
        };

        let processor =
            TakeOrdersProcessor::new(&raindex, config).expect("processor should be created");

        assert_eq!(processor.expected_input_token, order.validInputs[0].token);
        assert_eq!(processor.expected_output_token, order.validOutputs[0].token);
    }

    #[test]
    fn validate_tokens_errors_on_self_trade() {
        let raindex = test_raindex();
        let mut order = sample_order();
        order.validOutputs[0].token = order.validInputs[0].token;
        let config = TakeOrdersConfig {
            orders: vec![make_take_order(OrderRef::Inline(order.clone()))],
            minimum_input: parse_float("0"),
            maximum_input: parse_float("10"),
            maximum_io_ratio: parse_float("2"),
            taker: Address::repeat_byte(0x66),
            data: Vec::new(),
        };
        let processor =
            TakeOrdersProcessor::new(&raindex, config).expect("processor should initialize");
        let take_order = make_take_order(OrderRef::Inline(order.clone()));

        let err = processor
            .validate_tokens(&order, &take_order)
            .expect_err("self trade should error");

        assert!(matches!(err, RaindexError::TokenSelfTrade));
    }

    #[test]
    fn validate_tokens_errors_on_mismatched_tokens() {
        let raindex = test_raindex();
        let base_order = sample_order();
        let config = TakeOrdersConfig {
            orders: vec![make_take_order(OrderRef::Inline(base_order.clone()))],
            minimum_input: parse_float("0"),
            maximum_input: parse_float("10"),
            maximum_io_ratio: parse_float("2"),
            taker: Address::repeat_byte(0x77),
            data: Vec::new(),
        };
        let processor =
            TakeOrdersProcessor::new(&raindex, config).expect("processor should initialize");

        let mut other_order = sample_order();
        other_order.validInputs[0].token = Address::repeat_byte(0x99);
        let take_order = make_take_order(OrderRef::Inline(other_order.clone()));

        let err = processor
            .validate_tokens(&other_order, &take_order)
            .expect_err("mismatched tokens should error");

        assert!(matches!(err, RaindexError::TokenMismatch));
    }

    #[test]
    fn ensure_minimum_input_enforces_threshold() {
        let raindex = test_raindex();
        let order = sample_order();
        let config = TakeOrdersConfig {
            orders: vec![make_take_order(OrderRef::Inline(order))],
            minimum_input: parse_float("5"),
            maximum_input: parse_float("10"),
            maximum_io_ratio: parse_float("2"),
            taker: Address::repeat_byte(0x88),
            data: Vec::new(),
        };
        let processor =
            TakeOrdersProcessor::new(&raindex, config).expect("processor should initialize");

        let err = processor
            .ensure_minimum_input()
            .expect_err("total input below minimum should error");

        match err {
            RaindexError::MinimumInputNotMet { minimum, actual } => {
                assert!(minimum.eq(parse_float("5")).expect("minimum comparison"));
                assert!(actual.is_zero().expect("actual zero"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn apply_vault_updates_mutates_state_and_records_deltas() {
        let mut working_state = state::RaindexState::default();
        let order = sample_order();
        let input_io = order.validInputs[0].clone();
        let output_io = order.validOutputs[0].clone();
        let taker_output = parse_float("3");
        let taker_input = parse_float("2");

        let input_key = VaultKey::new(order.owner, input_io.token, input_io.vaultId);
        let output_key = VaultKey::new(order.owner, output_io.token, output_io.vaultId);
        let starting_input_balance = parse_float("10");
        let starting_output_balance = parse_float("5");
        working_state
            .vault_balances
            .insert(input_key, starting_input_balance.clone());
        working_state
            .vault_balances
            .insert(output_key, starting_output_balance.clone());

        let mut deltas = Vec::new();

        apply_vault_updates(
            &mut working_state,
            &order,
            &input_io,
            &output_io,
            taker_output.clone(),
            taker_input.clone(),
            &mut deltas,
        )
        .expect("apply vault updates should succeed");

        let updated_input = working_state
            .vault_balances
            .get(&input_key)
            .expect("input balance stored")
            .clone();
        let expected_input = (starting_input_balance.clone() + taker_output.clone())
            .expect("expected input balance");
        assert!(updated_input.eq(expected_input).expect("input balance eq"));

        let updated_output = working_state
            .vault_balances
            .get(&output_key)
            .expect("output balance stored")
            .clone();
        let expected_negative = taker_input.clone().neg().expect("neg");
        let expected_output = (starting_output_balance.clone() + expected_negative.clone())
            .expect("expected output balance");
        assert!(updated_output
            .eq(expected_output)
            .expect("output balance eq"));

        assert_eq!(deltas.len(), 2);
        assert_eq!(deltas[0].owner, order.owner);
        assert_eq!(deltas[0].token, input_io.token);
        assert_eq!(deltas[0].vault_id, input_io.vaultId);
        assert_eq!(deltas[1].owner, order.owner);
        assert_eq!(deltas[1].token, output_io.token);
        assert_eq!(deltas[1].vault_id, output_io.vaultId);
        assert!(deltas[0]
            .delta
            .clone()
            .eq(taker_output.clone())
            .expect("delta input eq"));
        assert!(deltas[1]
            .delta
            .clone()
            .eq(expected_negative)
            .expect("delta output eq"));
    }

    #[test]
    fn set_balance_diff_column_resizes_and_sets_value() {
        let mut column = Vec::new();
        let value = B256::from(U256::from(99_u64));

        set_balance_diff_column(&mut column, value);

        assert_eq!(column.len(), CONTEXT_VAULT_IO_BALANCE_DIFF_ROW);
        assert_eq!(column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1], value);

        let replacement = B256::from(U256::from(123_u64));
        set_balance_diff_column(&mut column, replacement);

        assert_eq!(column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1], replacement);
    }

    #[test]
    fn validate_io_indices_errors_on_out_of_bounds() {
        let order = sample_order();
        let take_order = TakeOrder {
            order: OrderRef::Inline(order.clone()),
            input_io_index: 2,
            output_io_index: 0,
            signed_context: Vec::new(),
        };

        let err = validate_io_indices(&take_order, &order).expect_err("invalid index should error");

        assert!(matches!(
            err,
            RaindexError::InvalidInputIndex { index: 2, len: 1 }
        ));
    }
}
