//! Virtual Raindex core primitives and interpreter host glue.

mod cache;
pub mod host;
mod state;

pub use cache::{CodeCache, StaticCodeCache};
pub use host::RevmInterpreterHost;
pub use state::{
    derive_fqn, Env, RaindexMutation, Snapshot, StoreKey, StoreKeyValue, StoreSet,
    TokenDecimalEntry, VaultDelta, VaultKey,
};

#[cfg(test)]
mod integration_tests;

use std::{
    collections::HashMap,
    fmt,
    ops::{Mul, Neg},
    sync::Arc,
};

use alloy::primitives::{Address, B256, U256};
use rain_interpreter_bindings::IInterpreterV4::EvalV4;
use rain_math_float::{Float, FloatError};
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1, TaskV2, IOV2};

pub type Result<T> = std::result::Result<T, RaindexError>;

/// Errors that can occur while working with the Virtual Raindex.
#[derive(Debug)]
pub enum RaindexError {
    /// Placeholder error used for mutations and features that are not yet implemented.
    Unimplemented(&'static str),
    /// Error bubbling up from Float math helpers.
    Float(FloatError),
    /// Raised when the interpreter host cannot resolve bytecode for a given address.
    MissingBytecode {
        address: Address,
        kind: BytecodeKind,
    },
    /// Wrapper for REVM execution failures.
    RevmExecution(String),
    /// Raised when a referenced order hash cannot be resolved from state.
    OrderNotFound { order_hash: B256 },
    /// Raised when an input IO index is outside the order's valid inputs array.
    InvalidInputIndex { index: usize, len: usize },
    /// Raised when an output IO index is outside the order's valid outputs array.
    InvalidOutputIndex { index: usize, len: usize },
    /// Raised when token decimals are required but missing from virtual state.
    TokenDecimalMissing { token: Address },
    /// Raised when a quote attempts to use the same token for input and output.
    TokenSelfTrade,
    /// Raised when the take orders config contains no orders.
    NoOrders,
    /// Raised when the maximum input for take orders is zero.
    ZeroMaximumInput,
    /// Raised when take orders mix different input or output tokens.
    TokenMismatch,
    /// Raised when the total input from take orders is less than the configured minimum.
    MinimumInputNotMet { minimum: Float, actual: Float },
}

impl From<FloatError> for RaindexError {
    fn from(value: FloatError) -> Self {
        Self::Float(value)
    }
}

impl fmt::Display for RaindexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaindexError::Unimplemented(feature) => write!(f, "{feature} is not implemented"),
            RaindexError::Float(err) => write!(f, "float error: {err}"),
            RaindexError::MissingBytecode { address, kind } => {
                write!(f, "missing {kind} bytecode for address {address}")
            }
            RaindexError::RevmExecution(reason) => write!(f, "revm execution failed: {reason}"),
            RaindexError::OrderNotFound { order_hash } => {
                write!(f, "order {order_hash:?} not found in virtual state")
            }
            RaindexError::InvalidInputIndex { index, len } => {
                write!(f, "input IO index {index} out of bounds (len {len})")
            }
            RaindexError::InvalidOutputIndex { index, len } => {
                write!(f, "output IO index {index} out of bounds (len {len})")
            }
            RaindexError::TokenDecimalMissing { token } => {
                write!(f, "missing token decimals for {token}")
            }
            RaindexError::TokenSelfTrade => write!(f, "token self trade is not allowed"),
            RaindexError::NoOrders => write!(f, "take orders requires at least one order"),
            RaindexError::ZeroMaximumInput => {
                write!(f, "take orders maximum input must be positive")
            }
            RaindexError::TokenMismatch => {
                write!(f, "all take orders must share the same input/output tokens")
            }
            RaindexError::MinimumInputNotMet { minimum, actual } => {
                let min = minimum
                    .format()
                    .unwrap_or_else(|_| "<format error>".to_string());
                let act = actual
                    .format()
                    .unwrap_or_else(|_| "<format error>".to_string());
                write!(
                    f,
                    "take orders minimum input {min} not satisfied (actual {act})"
                )
            }
        }
    }
}

impl std::error::Error for RaindexError {}

/// Identifies the type of bytecode requested from the [CodeCache].
#[derive(Clone, Copy, Debug)]
pub enum BytecodeKind {
    Interpreter,
    Store,
}

impl fmt::Display for BytecodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytecodeKind::Interpreter => write!(f, "interpreter"),
            BytecodeKind::Store => write!(f, "store"),
        }
    }
}

/// Describes how to locate an order for quote/take operations.
#[derive(Clone, Debug)]
pub enum OrderRef {
    /// Reference an order already stored within the virtual raindex by hash.
    ByHash(B256),
    /// Provide an inline order payload without mutating virtual state.
    Inline(OrderV4),
}

/// Temporary overlay applied to interpreter store reads during evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreOverride {
    pub store: Address,
    pub fqn: B256,
    pub key: B256,
    pub value: B256,
}

/// Input parameters for calculating a quote against an order.
#[derive(Clone, Debug)]
pub struct QuoteRequest {
    pub order: OrderRef,
    pub input_io_index: usize,
    pub output_io_index: usize,
    pub counterparty: Address,
    pub signed_context: Vec<SignedContextV1>,
    pub overrides: Vec<StoreOverride>,
}

impl QuoteRequest {
    pub fn new(
        order: OrderRef,
        input_io_index: usize,
        output_io_index: usize,
        counterparty: Address,
    ) -> Self {
        Self {
            order,
            input_io_index,
            output_io_index,
            counterparty,
            signed_context: Vec::new(),
            overrides: Vec::new(),
        }
    }

    pub fn with_signed_context(mut self, signed_context: Vec<SignedContextV1>) -> Self {
        self.signed_context = signed_context;
        self
    }

    pub fn with_overrides(mut self, overrides: Vec<StoreOverride>) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Result of executing calculate-io for an order.
#[derive(Clone, Debug)]
pub struct Quote {
    pub io_ratio: Float,
    pub output_max: Float,
    pub stack: Vec<B256>,
    pub writes: Vec<B256>,
}

const CALLING_CONTEXT_COLUMNS: usize = 4;
const CONTEXT_CALLING_CONTEXT_COLUMN: usize = 1;
const CONTEXT_CALCULATIONS_COLUMN: usize = 2;
const CONTEXT_VAULT_INPUTS_COLUMN: usize = 3;
const CONTEXT_VAULT_OUTPUTS_COLUMN: usize = 4;
const HANDLE_IO_ENTRYPOINT: u64 = 1;

#[derive(Clone, Debug)]
pub struct TakeOrder {
    pub order: OrderRef,
    pub input_io_index: usize,
    pub output_io_index: usize,
    pub signed_context: Vec<SignedContextV1>,
}

#[derive(Clone, Debug)]
pub struct TakeOrdersConfig {
    pub orders: Vec<TakeOrder>,
    pub minimum_input: Float,
    pub maximum_input: Float,
    // From the perspective of the taker
    pub maximum_io_ratio: Float,
    pub taker: Address,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TakeOrderWarning {
    OrderNotFound { order_hash: B256 },
    RatioExceeded { order_hash: B256 },
    ZeroAmount { order_hash: B256 },
}

#[derive(Clone, Debug)]
pub struct TakenOrder {
    pub order_hash: B256,
    pub input: Float,
    pub output: Float,
}

#[derive(Clone, Debug)]
pub struct TakeOrdersOutcome {
    pub total_input: Float,
    pub total_output: Float,
    pub taken: Vec<TakenOrder>,
    pub warnings: Vec<TakeOrderWarning>,
    pub mutations: Vec<RaindexMutation>,
}
const CONTEXT_VAULT_IO_BALANCE_DIFF_ROW: usize = 5;
pub struct VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    state: state::RaindexState,
    code_cache: Arc<C>,
    interpreter_host: Arc<H>,
    orderbook: Address,
}

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    pub fn new(orderbook: Address, code_cache: Arc<C>, interpreter_host: Arc<H>) -> Self {
        Self {
            state: state::RaindexState::default(),
            code_cache,
            interpreter_host,
            orderbook,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.snapshot()
    }

    pub fn apply_mutations(&mut self, mutations: &[RaindexMutation]) -> Result<()> {
        self.prepare_mutations(mutations)?;

        let mut draft = self.state.clone();

        for mutation in mutations {
            if let RaindexMutation::SetOrders { orders } = mutation {
                for order in orders {
                    self.ensure_order_context(&mut draft, order)?;
                }
            }
        }

        draft.apply_mutations(mutations)?;
        self.state = draft;
        Ok(())
    }

    pub fn quote(&self, request: QuoteRequest) -> Result<Quote> {
        let QuoteRequest {
            order,
            input_io_index,
            output_io_index,
            counterparty,
            signed_context,
            overrides,
        } = request;

        let order = self.resolve_order(order)?;
        self.code_cache.ensure_artifacts(&order)?;

        let input_len = order.validInputs.len();
        if input_io_index >= input_len {
            return Err(RaindexError::InvalidInputIndex {
                index: input_io_index,
                len: input_len,
            });
        }

        let output_len = order.validOutputs.len();
        if output_io_index >= output_len {
            return Err(RaindexError::InvalidOutputIndex {
                index: output_io_index,
                len: output_len,
            });
        }

        let input_io = &order.validInputs[input_io_index];
        let output_io = &order.validOutputs[output_io_index];

        if input_io.token == output_io.token {
            return Err(RaindexError::TokenSelfTrade);
        }

        let input_decimals = *self.state.token_decimals.get(&input_io.token).ok_or(
            RaindexError::TokenDecimalMissing {
                token: input_io.token,
            },
        )?;
        let output_decimals = *self.state.token_decimals.get(&output_io.token).ok_or(
            RaindexError::TokenDecimalMissing {
                token: output_io.token,
            },
        )?;

        let order_hash = state::order_hash(&order);

        let input_vault_balance = self
            .state
            .vault_balances
            .get(&VaultKey::new(
                order.owner,
                input_io.token,
                input_io.vaultId,
            ))
            .cloned()
            .unwrap_or_default();
        let output_vault_balance = self
            .state
            .vault_balances
            .get(&VaultKey::new(
                order.owner,
                output_io.token,
                output_io.vaultId,
            ))
            .cloned()
            .unwrap_or_default();

        let context = self.build_quote_context(
            order_hash,
            order.owner,
            counterparty,
            input_io,
            input_decimals,
            input_vault_balance,
            output_io,
            output_decimals,
            output_vault_balance,
            &signed_context,
        );

        let mut store_snapshot = self.state.store.clone();
        for override_entry in overrides {
            let key = StoreKey::new(override_entry.store, override_entry.fqn, override_entry.key);
            store_snapshot.insert(key, override_entry.value);
        }

        let state_namespace = address_to_u256(order.owner);
        let fqn = derive_fqn(state_namespace, self.orderbook);
        let namespace = U256::from_be_slice(fqn.as_slice());

        let eval = EvalV4 {
            store: order.evaluable.store,
            namespace,
            bytecode: order.evaluable.bytecode.clone(),
            sourceIndex: U256::ZERO,
            context,
            inputs: Vec::new(),
            stateOverlay: Vec::new(),
        };

        let mut outcome = self.interpreter_host.eval4(
            order.evaluable.interpreter,
            &eval,
            &store_snapshot,
            self.state.env,
        )?;

        if outcome.stack.len() < 2 {
            return Err(RaindexError::Unimplemented("calculate-io outputs"));
        }

        let io_ratio = Float::from_raw(outcome.stack[0]);
        let mut output_max = Float::from_raw(outcome.stack[1]);
        output_max = output_max.min(output_vault_balance)?;
        outcome.stack[1] = output_max.get_inner();

        Ok(Quote {
            io_ratio,
            output_max,
            stack: outcome.stack,
            writes: outcome.writes,
        })
    }

    pub fn interpreter(&self) -> &Arc<H> {
        &self.interpreter_host
    }

    pub fn code_cache(&self) -> &Arc<C> {
        &self.code_cache
    }

    pub fn orderbook_address(&self) -> Address {
        self.orderbook
    }

    pub fn take_orders(&self, config: TakeOrdersConfig) -> Result<TakeOrdersOutcome> {
        if config.orders.is_empty() {
            return Err(RaindexError::NoOrders);
        }

        if config.maximum_input.is_zero()? {
            return Err(RaindexError::ZeroMaximumInput);
        }

        let mut working_state = self.state.clone();

        let first_order_ref = &config.orders[0];
        let first_order = self.resolve_order(first_order_ref.order.clone())?;
        if first_order_ref.input_io_index >= first_order.validInputs.len() {
            return Err(RaindexError::InvalidInputIndex {
                index: first_order_ref.input_io_index,
                len: first_order.validInputs.len(),
            });
        }
        if first_order_ref.output_io_index >= first_order.validOutputs.len() {
            return Err(RaindexError::InvalidOutputIndex {
                index: first_order_ref.output_io_index,
                len: first_order.validOutputs.len(),
            });
        }

        let expected_input_token = first_order.validInputs[first_order_ref.input_io_index].token;
        let expected_output_token = first_order.validOutputs[first_order_ref.output_io_index].token;

        let mut total_input = Float::default();
        let mut total_output = Float::default();
        let mut remaining_input = config.maximum_input.clone();
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
                match self.resolve_order(take_order.order.clone()) {
                    Ok(order) => order,
                    Err(RaindexError::OrderNotFound { order_hash }) => {
                        warnings.push(TakeOrderWarning::OrderNotFound { order_hash });
                        continue;
                    }
                    Err(other) => return Err(other),
                }
            };

            if take_order.input_io_index >= resolved_order.validInputs.len() {
                return Err(RaindexError::InvalidInputIndex {
                    index: take_order.input_io_index,
                    len: resolved_order.validInputs.len(),
                });
            }
            if take_order.output_io_index >= resolved_order.validOutputs.len() {
                return Err(RaindexError::InvalidOutputIndex {
                    index: take_order.output_io_index,
                    len: resolved_order.validOutputs.len(),
                });
            }

            let input_io = &resolved_order.validInputs[take_order.input_io_index];
            let output_io = &resolved_order.validOutputs[take_order.output_io_index];

            if input_io.token == output_io.token {
                return Err(RaindexError::TokenSelfTrade);
            }

            if input_io.token != expected_input_token || output_io.token != expected_output_token {
                return Err(RaindexError::TokenMismatch);
            }

            let store_snapshot = working_state.store.clone();
            let mut calculation = self.calculate_order_io_for_take(
                &working_state,
                &store_snapshot,
                &resolved_order,
                take_order.input_io_index,
                take_order.output_io_index,
                config.taker,
                &take_order.signed_context,
            )?;

            if calculation.io_ratio.gt(config.maximum_io_ratio.clone())? {
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

            let taker_input = calculation.output_max.min(remaining_input.clone())?;
            if taker_input.is_zero()? {
                continue;
            }

            let taker_output = calculation.io_ratio.clone().mul(taker_input.clone())?;

            total_input = (total_input + taker_input.clone())?;
            total_output = (total_output + taker_output.clone())?;
            remaining_input = (remaining_input - taker_input.clone())?;

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

            let input_key = VaultKey::new(resolved_order.owner, input_io.token, input_io.vaultId);
            let input_balance = working_state
                .vault_balances
                .entry(input_key)
                .or_insert_with(Float::default)
                .clone();
            let new_input_balance = (input_balance + taker_output.clone())?;
            working_state
                .vault_balances
                .insert(input_key, new_input_balance.clone());

            let negative_taker_input = taker_input.clone().neg()?;
            let output_key =
                VaultKey::new(resolved_order.owner, output_io.token, output_io.vaultId);
            let output_balance = working_state
                .vault_balances
                .entry(output_key)
                .or_insert_with(Float::default)
                .clone();
            let new_output_balance = (output_balance + negative_taker_input.clone())?;
            working_state
                .vault_balances
                .insert(output_key, new_output_balance.clone());

            vault_deltas.push(VaultDelta {
                owner: resolved_order.owner,
                token: input_io.token,
                vault_id: input_io.vaultId,
                delta: taker_output.clone(),
            });
            vault_deltas.push(VaultDelta {
                owner: resolved_order.owner,
                token: output_io.token,
                vault_id: output_io.vaultId,
                delta: negative_taker_input.clone(),
            });

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
            let handle_outcome = self.interpreter_host.eval4(
                calculation.order.evaluable.interpreter,
                &EvalV4 {
                    store: calculation.store,
                    namespace: calculation.namespace,
                    bytecode: calculation.order.evaluable.bytecode.clone(),
                    sourceIndex: U256::from(HANDLE_IO_ENTRYPOINT),
                    context: context,
                    inputs: Vec::new(),
                    stateOverlay: Vec::new(),
                },
                &store_snapshot_after_calc,
                self.state.env,
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

        if total_input.lt(config.minimum_input.clone())? {
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

    pub fn take_orders_and_apply_state(
        &mut self,
        config: TakeOrdersConfig,
    ) -> Result<TakeOrdersOutcome> {
        let outcome = self.take_orders(config.clone())?;
        if !outcome.mutations.is_empty() {
            self.apply_mutations(&outcome.mutations)?;
        }
        Ok(outcome)
    }

    fn calculate_order_io_for_take(
        &self,
        working_state: &state::RaindexState,
        store_snapshot: &HashMap<StoreKey, B256>,
        order: &OrderV4,
        input_io_index: usize,
        output_io_index: usize,
        counterparty: Address,
        signed_context: &[SignedContextV1],
    ) -> Result<OrderCalculation> {
        self.code_cache.ensure_artifacts(order)?;

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
        let mut context = self.build_quote_context(
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

        let state_namespace = address_to_u256(order.owner);
        let qualified = derive_fqn(state_namespace, self.orderbook);
        let namespace = U256::from_be_slice(qualified.as_slice());

        let eval = EvalV4 {
            store: order.evaluable.store,
            namespace,
            bytecode: order.evaluable.bytecode.clone(),
            sourceIndex: U256::ZERO,
            context: context.clone(),
            inputs: Vec::new(),
            stateOverlay: Vec::new(),
        };

        let outcome = self.interpreter_host.eval4(
            order.evaluable.interpreter,
            &eval,
            store_snapshot,
            self.state.env,
        )?;

        if outcome.stack.len() < 2 {
            return Err(RaindexError::Unimplemented("calculate-io outputs"));
        }

        let io_ratio = Float::from_raw(outcome.stack[0]);
        let mut output_max = Float::from_raw(outcome.stack[1]);
        output_max = output_max.min(output_balance)?;

        context[CONTEXT_CALCULATIONS_COLUMN - 1] =
            vec![output_max.get_inner(), io_ratio.get_inner()];

        let calculate_writes = writes_to_pairs(&outcome.writes)?;

        Ok(OrderCalculation {
            order: order.clone(),
            io_ratio,
            output_max,
            context,
            namespace,
            qualified_namespace: qualified,
            store: order.evaluable.store,
            calculate_writes,
        })
    }

    pub fn add_order(&mut self, order: OrderV4, post_tasks: Vec<TaskV2>) -> Result<()> {
        self.code_cache.ensure_artifacts(&order)?;
        for task in &post_tasks {
            ensure_task_bytecode(self.code_cache.as_ref(), task)?;
        }

        let mut draft = self.state.clone();
        ensure_vault_entries(&mut draft, &order);
        draft.apply_mutations(&[RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        }])?;

        run_post_tasks(self, &mut draft, &order, &post_tasks)?;

        self.state = draft;
        Ok(())
    }

    fn prepare_mutations(&self, mutations: &[RaindexMutation]) -> Result<()> {
        for mutation in mutations {
            match mutation {
                RaindexMutation::SetOrders { orders } => {
                    for order in orders {
                        self.code_cache.ensure_artifacts(order)?;
                    }
                }
                RaindexMutation::Batch(batch) => self.prepare_mutations(batch)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn resolve_order(&self, reference: OrderRef) -> Result<OrderV4> {
        match reference {
            OrderRef::Inline(order) => Ok(order),
            OrderRef::ByHash(hash) => self
                .state
                .orders
                .get(&hash)
                .cloned()
                .ok_or(RaindexError::OrderNotFound { order_hash: hash }),
        }
    }

    fn ensure_order_context(&self, state: &mut state::RaindexState, order: &OrderV4) -> Result<()> {
        ensure_vault_entries(state, order);
        Ok(())
    }

    fn build_post_context(&self, order: &OrderV4) -> Vec<Vec<B256>> {
        let order_hash = state::order_hash(order);
        vec![vec![order_hash, order.owner.into_word()]]
    }

    fn build_quote_context(
        &self,
        order_hash: B256,
        owner: Address,
        counterparty: Address,
        input: &IOV2,
        input_decimals: u8,
        input_balance: Float,
        output: &IOV2,
        output_decimals: u8,
        output_balance: Float,
        signed_context: &[SignedContextV1],
    ) -> Vec<Vec<B256>> {
        let mut base_columns = vec![Vec::new(); CALLING_CONTEXT_COLUMNS];

        base_columns[CONTEXT_CALLING_CONTEXT_COLUMN - 1] =
            vec![order_hash, owner.into_word(), counterparty.into_word()];

        base_columns[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![B256::ZERO; 2];

        base_columns[CONTEXT_VAULT_INPUTS_COLUMN - 1] = vec![
            input.token.into_word(),
            u8_to_b256(input_decimals),
            input.vaultId,
            input_balance.get_inner(),
            B256::ZERO,
        ];

        base_columns[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = vec![
            output.token.into_word(),
            u8_to_b256(output_decimals),
            output.vaultId,
            output_balance.get_inner(),
            B256::ZERO,
        ];

        self.build_context(base_columns, signed_context, counterparty)
    }

    fn build_context(
        &self,
        base_columns: Vec<Vec<B256>>,
        signed_context: &[SignedContextV1],
        counterparty: Address,
    ) -> Vec<Vec<B256>> {
        let mut context = Vec::with_capacity(
            1 + base_columns.len()
                + if signed_context.is_empty() {
                    0
                } else {
                    signed_context.len() + 1
                },
        );

        context.push(vec![counterparty.into_word(), self.orderbook.into_word()]);
        context.extend(base_columns.into_iter());

        if !signed_context.is_empty() {
            let mut signers = Vec::with_capacity(signed_context.len());
            for sc in signed_context {
                signers.push(sc.signer.into_word());
            }
            context.push(signers);

            for sc in signed_context {
                context.push(sc.context.clone());
            }
        }

        context
    }
}

fn address_to_u256(address: Address) -> U256 {
    U256::from_be_slice(address.into_word().as_slice())
}

fn u8_to_b256(value: u8) -> B256 {
    B256::from(U256::from(value))
}

fn ensure_vault_entries(state: &mut state::RaindexState, order: &OrderV4) {
    for io in &order.validOutputs {
        state
            .vault_balances
            .entry(VaultKey::new(order.owner, io.token, io.vaultId))
            .or_insert_with(Float::default);
    }
}

fn ensure_task_bytecode<C: CodeCache>(code_cache: &C, task: &TaskV2) -> Result<()> {
    let interpreter = Address::from(task.evaluable.interpreter);
    if code_cache.interpreter(interpreter).is_none() {
        return Err(RaindexError::MissingBytecode {
            address: interpreter,
            kind: BytecodeKind::Interpreter,
        });
    }

    let store = Address::from(task.evaluable.store);
    if code_cache.store(store).is_none() {
        return Err(RaindexError::MissingBytecode {
            address: store,
            kind: BytecodeKind::Store,
        });
    }

    Ok(())
}

fn run_post_tasks<C, H>(
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
    let namespace = address_to_u256(order.owner);
    let qualified = derive_fqn(namespace, raindex.orderbook);

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

fn writes_to_pairs(writes: &[B256]) -> Result<Vec<(B256, B256)>> {
    if writes.len() % 2 != 0 {
        return Err(RaindexError::Unimplemented(
            "unpaired store write from interpreter",
        ));
    }

    Ok(writes.chunks(2).map(|chunk| (chunk[0], chunk[1])).collect())
}

fn apply_store_writes(
    store: &mut HashMap<StoreKey, B256>,
    store_address: Address,
    qualified: B256,
    writes: &[(B256, B256)],
) {
    for (key, value) in writes {
        store.insert(StoreKey::new(store_address, qualified, *key), *value);
    }
}

fn set_balance_diff_column(column: &mut Vec<B256>, value: B256) {
    if column.len() < CONTEXT_VAULT_IO_BALANCE_DIFF_ROW {
        column.resize(CONTEXT_VAULT_IO_BALANCE_DIFF_ROW, B256::ZERO);
    }
    column[CONTEXT_VAULT_IO_BALANCE_DIFF_ROW - 1] = value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use crate::host::InterpreterHost;
    use alloy::primitives::{Address, Bytes, B256, U256};
    use rain_interpreter_bindings::IInterpreterV4::EvalV4;
    use rain_interpreter_test_fixtures::{Interpreter, LocalEvm, Store};
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::{OrderV4, TaskV2, IOV2};

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
    struct NullInterpreter;

    impl host::InterpreterHost for NullInterpreter {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: Env,
        ) -> Result<host::EvalOutcome> {
            Err(RaindexError::Unimplemented("interpreter host"))
        }
    }

    fn new_raindex() -> VirtualRaindex<NullCache, NullInterpreter> {
        let cache = Arc::new(NullCache::default());
        let interpreter = Arc::new(NullInterpreter::default());
        VirtualRaindex::new(Address::ZERO, cache, interpreter)
    }

    struct RecordingHost {
        outcome: host::EvalOutcome,
        evals: Mutex<Vec<EvalV4>>,
        snapshots: Mutex<Vec<HashMap<StoreKey, B256>>>,
        scripted: Mutex<Vec<host::EvalOutcome>>,
    }

    impl RecordingHost {
        fn new(outcome: host::EvalOutcome) -> Self {
            Self {
                outcome,
                evals: Mutex::new(Vec::new()),
                snapshots: Mutex::new(Vec::new()),
                scripted: Mutex::new(Vec::new()),
            }
        }

        fn last_eval(&self) -> Option<EvalV4> {
            self.evals.lock().unwrap().last().cloned()
        }

        fn last_snapshot(&self) -> Option<HashMap<StoreKey, B256>> {
            self.snapshots.lock().unwrap().last().cloned()
        }

        fn push_outcome(&self, outcome: host::EvalOutcome) {
            self.scripted.lock().unwrap().push(outcome);
        }
    }

    impl host::InterpreterHost for RecordingHost {
        fn eval4(
            &self,
            _interpreter: Address,
            eval: &EvalV4,
            store_snapshot: &HashMap<StoreKey, B256>,
            _env: Env,
        ) -> Result<host::EvalOutcome> {
            self.evals.lock().unwrap().push(eval.clone());
            self.snapshots.lock().unwrap().push(store_snapshot.clone());
            let mut scripted = self.scripted.lock().unwrap();
            if !scripted.is_empty() {
                Ok(scripted.remove(0))
            } else {
                Ok(self.outcome.clone())
            }
        }
    }

    fn test_order() -> OrderV4 {
        let mut order = OrderV4::default();
        order.owner = Address::repeat_byte(0x42);
        order.evaluable.interpreter = Address::repeat_byte(0xAA);
        order.evaluable.store = Address::repeat_byte(0xBB);
        order.evaluable.bytecode = Bytes::from(vec![0u8]);
        order.validInputs = vec![IOV2 {
            token: Address::repeat_byte(0x10),
            vaultId: B256::from([1u8; 32]),
        }];
        order.validOutputs = vec![IOV2 {
            token: Address::repeat_byte(0x20),
            vaultId: B256::from([2u8; 32]),
        }];
        order
    }

    fn cache_with_code(order: &OrderV4) -> Arc<StaticCodeCache> {
        let cache = Arc::new(StaticCodeCache::default());
        cache.upsert_interpreter(order.evaluable.interpreter, &[0u8]);
        cache.upsert_store(order.evaluable.store, &[0u8]);
        cache
    }

    fn new_quote_request(order_ref: OrderRef) -> QuoteRequest {
        QuoteRequest::new(order_ref, 0, 0, Address::repeat_byte(0xE1))
    }

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_string()).expect("float parse")
    }

    #[test]
    fn snapshot_defaults_to_zeroed_env() {
        let raindex = new_raindex();
        let snapshot = raindex.snapshot();
        assert_eq!(snapshot.env, Env::default());
    }

    #[test]
    fn set_env_updates_fields() {
        let mut raindex = new_raindex();

        raindex
            .apply_mutations(&[RaindexMutation::SetEnv {
                block_number: Some(42),
                timestamp: Some(1337),
            }])
            .expect("set env mutation should succeed");

        assert_eq!(
            raindex.snapshot().env,
            Env {
                block_number: 42,
                timestamp: 1337
            }
        );
    }

    #[test]
    fn batch_recurses_and_preserves_missing_fields() {
        let mut raindex = new_raindex();

        let batch = RaindexMutation::Batch(vec![
            RaindexMutation::SetEnv {
                block_number: Some(1),
                timestamp: None,
            },
            RaindexMutation::Batch(vec![RaindexMutation::SetEnv {
                block_number: None,
                timestamp: Some(2),
            }]),
        ]);

        raindex
            .apply_mutations(&[batch])
            .expect("batch mutation should succeed");

        assert_eq!(
            raindex.snapshot().env,
            Env {
                block_number: 1,
                timestamp: 2
            }
        );
    }

    #[test]
    fn set_orders_insert_and_remove() {
        let mut raindex = new_raindex();
        let mut order_a = OrderV4::default();
        order_a.nonce = B256::from([1u8; 32]);
        let mut order_b = OrderV4::default();
        order_b.nonce = B256::from([2u8; 32]);

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order_a.clone(), order_b.clone()],
            }])
            .expect("set orders should succeed");

        let snapshot = raindex.snapshot();
        assert_eq!(snapshot.orders.len(), 2);

        let hash_a = state::order_hash(&order_a);
        let hash_b = state::order_hash(&order_b);
        assert!(snapshot.orders.contains_key(&hash_a));
        assert!(snapshot.orders.contains_key(&hash_b));

        raindex
            .apply_mutations(&[RaindexMutation::RemoveOrders {
                order_hashes: vec![hash_a],
            }])
            .expect("remove order should succeed");

        let snapshot = raindex.snapshot();
        assert!(!snapshot.orders.contains_key(&hash_a));
        assert!(snapshot.orders.contains_key(&hash_b));
    }

    #[test]
    fn set_orders_idempotent() {
        let mut raindex = new_raindex();
        let mut order = OrderV4::default();
        order.nonce = B256::from([7u8; 32]);

        let mutation = RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        };

        raindex
            .apply_mutations(&[mutation.clone()])
            .expect("initial insert should succeed");
        raindex
            .apply_mutations(&[mutation])
            .expect("re-inserting identical order should succeed");

        let snapshot = raindex.snapshot();
        assert_eq!(snapshot.orders.len(), 1);
        let hash = state::order_hash(&order);
        assert_eq!(snapshot.orders.get(&hash), Some(&order));
    }

    #[test]
    fn vault_delta_accumulates() {
        let mut raindex = new_raindex();
        let owner = Address::repeat_byte(0x01);
        let token = Address::repeat_byte(0x02);
        let vault_id = B256::from([9u8; 32]);

        let add = VaultDelta {
            owner,
            token,
            vault_id,
            delta: parse_float("1"),
        };
        raindex
            .apply_mutations(&[RaindexMutation::VaultDeltas { deltas: vec![add] }])
            .expect("first delta should succeed");

        let sub = VaultDelta {
            owner,
            token,
            vault_id,
            delta: parse_float("-0.4"),
        };
        raindex
            .apply_mutations(&[RaindexMutation::VaultDeltas { deltas: vec![sub] }])
            .expect("second delta should succeed");

        let snapshot = raindex.snapshot();
        let key = VaultKey::new(owner, token, vault_id);
        let balance = snapshot.vault_balances.get(&key).expect("balance entry");
        let expected = (parse_float("1") + parse_float("-0.4")).expect("float math");
        assert_eq!(balance.get_inner(), expected.get_inner());
    }

    #[test]
    fn apply_store_sets_values() {
        let mut raindex = new_raindex();
        let store = Address::repeat_byte(0xaa);
        let fqn = B256::from([3u8; 32]);
        let key = B256::from([4u8; 32]);
        let value = B256::from([5u8; 32]);

        raindex
            .apply_mutations(&[RaindexMutation::ApplyStore {
                sets: vec![StoreSet {
                    store,
                    fqn,
                    kvs: vec![StoreKeyValue { key, value }],
                }],
            }])
            .expect("apply store should succeed");

        let snapshot = raindex.snapshot();
        let store_key = StoreKey::new(store, fqn, key);
        assert_eq!(snapshot.store.get(&store_key), Some(&value));
    }

    #[test]
    fn take_orders_returns_mutations() {
        let order = test_order();
        let cache = cache_with_code(&order);

        let calc_outcome = host::EvalOutcome {
            stack: vec![parse_float("1").get_inner(), parse_float("1").get_inner()],
            writes: vec![
                B256::from(U256::from(7_u64)),
                B256::from(U256::from(11_u64)),
            ],
        };
        let handle_outcome = host::EvalOutcome {
            stack: Vec::new(),
            writes: vec![
                B256::from(U256::from(13_u64)),
                B256::from(U256::from(17_u64)),
            ],
        };

        let host = Arc::new(RecordingHost::new(calc_outcome.clone()));
        host.push_outcome(calc_outcome.clone());
        host.push_outcome(handle_outcome.clone());

        let orderbook = Address::repeat_byte(0xAA);
        let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

        let hash = state::order_hash(&order);

        raindex
            .apply_mutations(&[RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: order.validOutputs[0].token,
                        decimals: 18,
                    },
                ],
            }])
            .expect("set decimals");

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            }])
            .expect("set order");

        raindex
            .apply_mutations(&[RaindexMutation::VaultDeltas {
                deltas: vec![VaultDelta {
                    owner: order.owner,
                    token: order.validOutputs[0].token,
                    vault_id: order.validOutputs[0].vaultId,
                    delta: parse_float("5"),
                }],
            }])
            .expect("seed vault");

        let config = TakeOrdersConfig {
            orders: vec![TakeOrder {
                order: OrderRef::ByHash(hash),
                input_io_index: 0,
                output_io_index: 0,
                signed_context: Vec::new(),
            }],
            minimum_input: parse_float("0"),
            maximum_input: parse_float("1"),
            maximum_io_ratio: parse_float("10"),
            taker: Address::repeat_byte(0xDD),
            data: Vec::new(),
        };

        let input_key = VaultKey::new(
            order.owner,
            order.validInputs[0].token,
            order.validInputs[0].vaultId,
        );
        let output_key = VaultKey::new(
            order.owner,
            order.validOutputs[0].token,
            order.validOutputs[0].vaultId,
        );

        let before_snapshot = raindex.snapshot();

        let outcome = raindex
            .take_orders(config.clone())
            .expect("simulate take orders");

        assert_eq!(outcome.taken.len(), 1);
        assert!(outcome.warnings.is_empty());
        assert_eq!(
            outcome.total_input.get_inner(),
            parse_float("1").get_inner()
        );
        assert_eq!(
            outcome.total_output.get_inner(),
            parse_float("1").get_inner()
        );
        assert!(!outcome.mutations.is_empty());

        let after_sim_snapshot = raindex.snapshot();
        let before_input = before_snapshot
            .vault_balances
            .get(&input_key)
            .cloned()
            .unwrap_or_default();
        let after_input = after_sim_snapshot
            .vault_balances
            .get(&input_key)
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            before_input.format().expect("format before input"),
            after_input.format().expect("format after input")
        );

        let before_output = before_snapshot
            .vault_balances
            .get(&output_key)
            .cloned()
            .unwrap_or_default();
        let after_output = after_sim_snapshot
            .vault_balances
            .get(&output_key)
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            before_output.format().expect("format before output"),
            after_output.format().expect("format after output")
        );

        host.push_outcome(calc_outcome.clone());
        host.push_outcome(handle_outcome.clone());

        let applied = raindex
            .take_orders_and_apply_state(config)
            .expect("apply take orders");
        assert_eq!(applied.taken.len(), 1);

        let applied_snapshot = raindex.snapshot();

        let applied_input = applied_snapshot
            .vault_balances
            .get(&input_key)
            .cloned()
            .unwrap_or_default();
        let applied_output = applied_snapshot
            .vault_balances
            .get(&output_key)
            .cloned()
            .unwrap_or_default();

        assert_eq!(
            applied_input.format().expect("format input"),
            parse_float("1").format().expect("expected input format"),
        );
        assert_eq!(
            applied_output.format().expect("format output"),
            parse_float("4").format().expect("expected output format"),
        );

        let qualified = derive_fqn(address_to_u256(order.owner), orderbook);
        let calc_key = StoreKey::new(
            order.evaluable.store,
            qualified,
            B256::from(U256::from(7_u64)),
        );
        let handle_key = StoreKey::new(
            order.evaluable.store,
            qualified,
            B256::from(U256::from(13_u64)),
        );

        assert_eq!(
            applied_snapshot.store.get(&calc_key),
            Some(&B256::from(U256::from(11_u64)))
        );
        assert_eq!(
            applied_snapshot.store.get(&handle_key),
            Some(&B256::from(U256::from(17_u64)))
        );
    }

    #[test]
    fn take_orders_enforces_minimum_input() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let host = Arc::new(RecordingHost::new(host::EvalOutcome {
            stack: vec![parse_float("1").get_inner(), parse_float("0.5").get_inner()],
            writes: Vec::new(),
        }));

        let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        raindex
            .apply_mutations(&[RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: order.validOutputs[0].token,
                        decimals: 18,
                    },
                ],
            }])
            .expect("set decimals");

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            }])
            .expect("set order");

        raindex
            .apply_mutations(&[RaindexMutation::VaultDeltas {
                deltas: vec![VaultDelta {
                    owner: order.owner,
                    token: order.validOutputs[0].token,
                    vault_id: order.validOutputs[0].vaultId,
                    delta: parse_float("1"),
                }],
            }])
            .expect("seed vault");

        let hash = state::order_hash(&order);
        let err = raindex
            .take_orders(TakeOrdersConfig {
                orders: vec![TakeOrder {
                    order: OrderRef::ByHash(hash),
                    input_io_index: 0,
                    output_io_index: 0,
                    signed_context: Vec::new(),
                }],
                minimum_input: parse_float("0.75"),
                maximum_input: parse_float("0.5"),
                maximum_io_ratio: parse_float("10"),
                taker: Address::repeat_byte(0xEE),
                data: Vec::new(),
            })
            .expect_err("minimum input");

        matches!(err, RaindexError::MinimumInputNotMet { .. });
    }

    #[test]
    fn take_orders_skips_ratio_exceeded() {
        let order = test_order();
        let cache = cache_with_code(&order);
        let host = Arc::new(RecordingHost::new(host::EvalOutcome {
            stack: vec![parse_float("5").get_inner(), parse_float("1").get_inner()],
            writes: Vec::new(),
        }));

        let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        raindex
            .apply_mutations(&[RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: order.validOutputs[0].token,
                        decimals: 18,
                    },
                ],
            }])
            .expect("set decimals");

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            }])
            .expect("set order");

        let hash = state::order_hash(&order);
        let outcome = raindex
            .take_orders(TakeOrdersConfig {
                orders: vec![TakeOrder {
                    order: OrderRef::ByHash(hash),
                    input_io_index: 0,
                    output_io_index: 0,
                    signed_context: Vec::new(),
                }],
                minimum_input: parse_float("0"),
                maximum_input: parse_float("10"),
                maximum_io_ratio: parse_float("1"),
                taker: Address::repeat_byte(0xEF),
                data: Vec::new(),
            })
            .expect("take orders");

        assert!(outcome.taken.is_empty());
        assert!(matches!(
            outcome.warnings.first(),
            Some(TakeOrderWarning::RatioExceeded { .. })
        ));
    }

    #[test]
    fn set_token_decimals() {
        let mut raindex = new_raindex();
        let token_a = Address::repeat_byte(0xa1);
        let token_b = Address::repeat_byte(0xb2);

        raindex
            .apply_mutations(&[RaindexMutation::SetTokenDecimals {
                entries: vec![
                    TokenDecimalEntry {
                        token: token_a,
                        decimals: 18,
                    },
                    TokenDecimalEntry {
                        token: token_b,
                        decimals: 6,
                    },
                ],
            }])
            .expect("set token decimals should succeed");

        let snapshot = raindex.snapshot();
        assert_eq!(snapshot.token_decimals.get(&token_a), Some(&18));
        assert_eq!(snapshot.token_decimals.get(&token_b), Some(&6));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn revm_host_matches_contract() {
        let local_evm = LocalEvm::new().await;
        let rain_src = b"/* 0. calculate-io */\n_ _: 10 20;\n\n/* 1. handle-io */\n:;".to_vec();

        let parse_return = local_evm
            .deployer
            .parse2(Bytes::from(rain_src.clone()))
            .call()
            .await
            .expect("parse2");

        let mut order = OrderV4::default();
        order.evaluable.interpreter = *local_evm.interpreter.address();
        order.evaluable.store = *local_evm.store.address();
        order.evaluable.bytecode = parse_return.clone();

        let contract_eval = Interpreter::EvalV4 {
            store: order.evaluable.store,
            namespace: U256::ZERO,
            bytecode: parse_return.clone(),
            sourceIndex: U256::ZERO,
            context: vec![],
            inputs: vec![],
            stateOverlay: vec![],
        };

        let expected = local_evm
            .interpreter
            .eval4(contract_eval.clone())
            .call()
            .await
            .expect("contract eval");
        let expected_stack: Vec<B256> = expected._0.into_iter().map(B256::from).collect();
        let expected_writes: Vec<B256> = expected._1.into_iter().map(B256::from).collect();

        let cache = Arc::new(StaticCodeCache::default());
        cache.upsert_interpreter(
            order.evaluable.interpreter,
            Interpreter::DEPLOYED_BYTECODE.as_ref(),
        );
        cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

        let host = RevmInterpreterHost::new(cache);

        let eval = EvalV4 {
            store: order.evaluable.store,
            namespace: U256::ZERO,
            bytecode: order.evaluable.bytecode.clone(),
            sourceIndex: U256::ZERO,
            context: vec![],
            inputs: vec![],
            stateOverlay: vec![],
        };

        let outcome = host
            .eval4(
                order.evaluable.interpreter,
                &eval,
                &std::collections::HashMap::new(),
                Env::default(),
            )
            .expect("eval4");

        assert_eq!(outcome.stack, expected_stack);
        assert_eq!(outcome.writes, expected_writes);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn quote_matches_contract_eval() {
        let local_evm = LocalEvm::new().await;
        let rain_src = b"/* 0. calculate-io */\n_ _: 10 20;\n\n/* 1. handle-io */\n:;".to_vec();

        let parse_return = local_evm
            .deployer
            .parse2(Bytes::from(rain_src.clone()))
            .call()
            .await
            .expect("parse2");
        let orderbook = Address::repeat_byte(0xAB);
        let mut order = OrderV4::default();
        order.owner = Address::repeat_byte(0x42);
        order.evaluable.interpreter = *local_evm.interpreter.address();
        order.evaluable.store = *local_evm.store.address();
        order.evaluable.bytecode = parse_return.clone();

        let input_token = Address::repeat_byte(0x11);
        let output_token = Address::repeat_byte(0x22);
        let input_vault_id = B256::from([1u8; 32]);
        let output_vault_id = B256::from([2u8; 32]);

        order.validInputs = vec![IOV2 {
            token: input_token,
            vaultId: input_vault_id,
        }];
        order.validOutputs = vec![IOV2 {
            token: output_token,
            vaultId: output_vault_id,
        }];

        let cache = Arc::new(StaticCodeCache::default());
        cache.upsert_interpreter(
            order.evaluable.interpreter,
            Interpreter::DEPLOYED_BYTECODE.as_ref(),
        );
        cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

        let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
        let mut raindex = VirtualRaindex::new(orderbook, cache, host);

        let decimals_mutation = RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: input_token,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: output_token,
                    decimals: 18,
                },
            ],
        };

        let output_balance = parse_float("5");
        let vault_delta = RaindexMutation::VaultDeltas {
            deltas: vec![VaultDelta {
                owner: order.owner,
                token: output_token,
                vault_id: output_vault_id,
                delta: output_balance,
            }],
        };

        raindex
            .apply_mutations(&[
                decimals_mutation,
                RaindexMutation::SetOrders {
                    orders: vec![order.clone()],
                },
                vault_delta,
            ])
            .expect("mutations");

        let order_hash = state::order_hash(&order);
        let state_namespace = address_to_u256(order.owner);
        let fqn = derive_fqn(state_namespace, orderbook);
        let namespace = U256::from_be_slice(fqn.as_slice());

        let counterparty = Address::repeat_byte(0xE1);
        let input_balance = Float::default();
        let context = raindex.build_quote_context(
            order_hash,
            order.owner,
            counterparty,
            &order.validInputs[0],
            18,
            input_balance,
            &order.validOutputs[0],
            18,
            output_balance,
            &[],
        );

        let contract_eval = Interpreter::EvalV4 {
            store: order.evaluable.store,
            namespace,
            bytecode: parse_return.clone(),
            sourceIndex: U256::ZERO,
            context: context.clone(),
            inputs: vec![],
            stateOverlay: vec![],
        };

        let expected = local_evm
            .interpreter
            .eval4(contract_eval)
            .call()
            .await
            .expect("contract eval");
        let expected_stack: Vec<B256> = expected._0.into_iter().map(B256::from).collect();
        assert_eq!(expected_stack.len(), 2);

        let stored_quote = raindex
            .quote(QuoteRequest::new(
                OrderRef::ByHash(order_hash),
                0,
                0,
                counterparty,
            ))
            .expect("stored quote");

        let inline_quote = raindex
            .quote(QuoteRequest::new(
                OrderRef::Inline(order.clone()),
                0,
                0,
                counterparty,
            ))
            .expect("inline quote");

        let expected_ratio = Float::from_raw(expected_stack[0]);
        let expected_max = Float::from_raw(expected_stack[1])
            .min(output_balance)
            .expect("min");

        assert_eq!(
            stored_quote.io_ratio.get_inner(),
            expected_ratio.get_inner()
        );
        assert_eq!(
            stored_quote.output_max.get_inner(),
            expected_max.get_inner()
        );
        assert_eq!(
            inline_quote.io_ratio.get_inner(),
            expected_ratio.get_inner()
        );
        assert_eq!(
            inline_quote.output_max.get_inner(),
            expected_max.get_inner()
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn quote_reflects_env_values() {
        let local_evm = LocalEvm::new().await;
        let rain_src =
            b"/* 0. calculate-io */\n_ _: now() block-number();\n\n/* 1. handle-io */\n:;".to_vec();

        let parse_return = local_evm
            .deployer
            .parse2(Bytes::from(rain_src.clone()))
            .call()
            .await
            .expect("parse2");

        let orderbook = Address::repeat_byte(0xCC);
        let mut order = OrderV4::default();
        order.owner = Address::repeat_byte(0x42);
        order.evaluable.interpreter = *local_evm.interpreter.address();
        order.evaluable.store = *local_evm.store.address();
        order.evaluable.bytecode = parse_return.clone();

        let input_token = Address::repeat_byte(0x11);
        let output_token = Address::repeat_byte(0x22);
        let input_vault_id = B256::from([0xA1; 32]);
        let output_vault_id = B256::from([0xB2; 32]);

        order.validInputs = vec![IOV2 {
            token: input_token,
            vaultId: input_vault_id,
        }];
        order.validOutputs = vec![IOV2 {
            token: output_token,
            vaultId: output_vault_id,
        }];

        let cache = Arc::new(StaticCodeCache::default());
        cache.upsert_interpreter(
            order.evaluable.interpreter,
            Interpreter::DEPLOYED_BYTECODE.as_ref(),
        );
        cache.upsert_store(order.evaluable.store, Store::DEPLOYED_BYTECODE.as_ref());

        let host = Arc::new(RevmInterpreterHost::new(cache.clone()));
        let mut raindex = VirtualRaindex::new(orderbook, cache, host);

        let token_decimals = RaindexMutation::SetTokenDecimals {
            entries: vec![
                TokenDecimalEntry {
                    token: input_token,
                    decimals: 18,
                },
                TokenDecimalEntry {
                    token: output_token,
                    decimals: 18,
                },
            ],
        };

        let initial_balance = parse_float("1000000");
        let vault_delta = RaindexMutation::VaultDeltas {
            deltas: vec![VaultDelta {
                owner: order.owner,
                token: output_token,
                vault_id: output_vault_id,
                delta: initial_balance,
            }],
        };

        let env_first = Env {
            block_number: 1_234,
            timestamp: 5_678,
        };

        let env_mutation = RaindexMutation::SetEnv {
            block_number: Some(env_first.block_number),
            timestamp: Some(env_first.timestamp),
        };

        let order_mutation = RaindexMutation::SetOrders {
            orders: vec![order.clone()],
        };

        raindex
            .apply_mutations(&[
                token_decimals,
                vault_delta.clone(),
                env_mutation,
                order_mutation,
            ])
            .expect("initial mutations");

        let order_hash = state::order_hash(&order);
        let request = QuoteRequest::new(OrderRef::ByHash(order_hash), 0, 0, Address::ZERO);

        let quote_first = raindex.quote(request.clone()).expect("quote env 1");
        // Rainlang pushes `block-number()` then `now()` so stack[0] maps to the block.
        let first_block = quote_first
            .io_ratio
            .to_fixed_decimal(0)
            .expect("block to fixed");
        assert_eq!(first_block, U256::from(env_first.block_number));

        let first_timestamp = quote_first
            .output_max
            .to_fixed_decimal(0)
            .expect("timestamp to fixed");
        assert_eq!(first_timestamp, U256::from(env_first.timestamp));

        let env_second = Env {
            block_number: 9_999,
            timestamp: 44_444,
        };

        raindex
            .apply_mutations(&[RaindexMutation::SetEnv {
                block_number: Some(env_second.block_number),
                timestamp: Some(env_second.timestamp),
            }])
            .expect("update env");

        // refresh vault balance so the min check does not clip our output
        raindex
            .apply_mutations(&[vault_delta.clone()])
            .expect("refresh balance");

        let quote_second = raindex.quote(request).expect("quote env 2");
        let second_block = quote_second
            .io_ratio
            .to_fixed_decimal(0)
            .expect("block to fixed second");
        assert_eq!(second_block, U256::from(env_second.block_number));

        let second_timestamp = quote_second
            .output_max
            .to_fixed_decimal(0)
            .expect("timestamp to fixed second");
        assert_eq!(second_timestamp, U256::from(env_second.timestamp));
    }

    #[test]
    fn quote_errors_when_order_missing() {
        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let cache = Arc::new(StaticCodeCache::default());
        let raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        let err = raindex
            .quote(new_quote_request(OrderRef::ByHash(B256::ZERO)))
            .expect_err("missing order");
        matches!(err, RaindexError::OrderNotFound { .. });
    }

    #[test]
    fn quote_errors_on_invalid_io_index() {
        let mut order = test_order();
        order.validInputs.push(IOV2 {
            token: Address::repeat_byte(0x11),
            vaultId: B256::from([3u8; 32]),
        });

        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let cache = cache_with_code(&order);
        let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            }])
            .expect("set order");

        let err = raindex
            .quote(QuoteRequest::new(
                OrderRef::ByHash(state::order_hash(&order)),
                2,
                0,
                Address::ZERO,
            ))
            .expect_err("invalid input index");
        matches!(err, RaindexError::InvalidInputIndex { .. });

        let err = raindex
            .quote(QuoteRequest::new(
                OrderRef::ByHash(state::order_hash(&order)),
                0,
                1,
                Address::ZERO,
            ))
            .expect_err("invalid output index");
        matches!(err, RaindexError::InvalidOutputIndex { .. });
    }

    #[test]
    fn quote_errors_without_token_decimals() {
        let order = test_order();
        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let cache = cache_with_code(&order);
        let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        raindex
            .apply_mutations(&[RaindexMutation::SetOrders {
                orders: vec![order.clone()],
            }])
            .expect("set order");

        let err = raindex
            .quote(new_quote_request(OrderRef::ByHash(state::order_hash(
                &order,
            ))))
            .expect_err("missing decimals");
        matches!(err, RaindexError::TokenDecimalMissing { .. });
    }

    #[test]
    fn add_order_runs_post_tasks() {
        let order = test_order();
        let cache = cache_with_code(&order);

        let key = B256::from(U256::from(7_u64));
        let value = B256::from(U256::from(42_u64));
        let outcome = host::EvalOutcome {
            stack: Vec::new(),
            writes: vec![key, value],
        };

        let host = Arc::new(RecordingHost::new(outcome));
        let orderbook = Address::repeat_byte(0x99);
        let mut raindex = VirtualRaindex::new(orderbook, cache, host);

        let task = TaskV2 {
            evaluable: order.evaluable.clone(),
            signedContext: Vec::new(),
        };

        raindex
            .add_order(order.clone(), vec![task])
            .expect("add order succeeds");

        let namespace = address_to_u256(order.owner);
        let qualified = derive_fqn(namespace, raindex.orderbook_address());
        let store_key = StoreKey::new(order.evaluable.store, qualified, key);

        let snapshot = raindex.snapshot();
        assert_eq!(snapshot.store.get(&store_key), Some(&value));
    }

    #[test]
    fn quote_errors_on_self_trade() {
        let mut order = test_order();
        order.validOutputs[0].token = order.validInputs[0].token;

        let host = Arc::new(RecordingHost::new(host::EvalOutcome::default()));
        let cache = cache_with_code(&order);
        let mut raindex = VirtualRaindex::new(Address::ZERO, cache, host);

        raindex
            .apply_mutations(&[
                RaindexMutation::SetOrders {
                    orders: vec![order.clone()],
                },
                RaindexMutation::SetTokenDecimals {
                    entries: vec![TokenDecimalEntry {
                        token: order.validInputs[0].token,
                        decimals: 18,
                    }],
                },
            ])
            .expect("mutations");

        let err = raindex
            .quote(new_quote_request(OrderRef::ByHash(state::order_hash(
                &order,
            ))))
            .expect_err("self trade");
        matches!(err, RaindexError::TokenSelfTrade);
    }

    #[test]
    fn quote_builds_expected_eval_context() {
        let order = test_order();
        let order_hash = state::order_hash(&order);
        let output_balance = parse_float("5");

        let outcome = host::EvalOutcome {
            stack: vec![
                parse_float("0.5").get_inner(),
                parse_float("10").get_inner(),
            ],
            writes: vec![B256::from([9u8; 32])],
        };
        let host = Arc::new(RecordingHost::new(outcome));
        let cache = cache_with_code(&order);
        let orderbook = Address::repeat_byte(0xAB);
        let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

        raindex
            .apply_mutations(&[
                RaindexMutation::SetTokenDecimals {
                    entries: vec![
                        TokenDecimalEntry {
                            token: order.validInputs[0].token,
                            decimals: 18,
                        },
                        TokenDecimalEntry {
                            token: order.validOutputs[0].token,
                            decimals: 18,
                        },
                    ],
                },
                RaindexMutation::SetOrders {
                    orders: vec![order.clone()],
                },
                RaindexMutation::VaultDeltas {
                    deltas: vec![VaultDelta {
                        owner: order.owner,
                        token: order.validOutputs[0].token,
                        vault_id: order.validOutputs[0].vaultId,
                        delta: output_balance,
                    }],
                },
            ])
            .expect("mutations");

        let quote = raindex
            .quote(new_quote_request(OrderRef::ByHash(order_hash)))
            .expect("quote");

        assert_eq!(quote.writes, vec![B256::from([9u8; 32])]);
        assert_eq!(quote.io_ratio.get_inner(), parse_float("0.5").get_inner());
        assert_eq!(quote.output_max.get_inner(), output_balance.get_inner());

        let recorded = host.last_eval().expect("eval recorded");
        assert_eq!(recorded.sourceIndex, U256::ZERO);

        let namespace = derive_fqn(address_to_u256(order.owner), orderbook);
        assert_eq!(
            recorded.namespace,
            U256::from_be_slice(namespace.as_slice())
        );

        assert_eq!(recorded.context.len(), 5);
        let base_column = &recorded.context[0];
        assert_eq!(base_column[0], Address::repeat_byte(0xE1).into_word());
        assert_eq!(base_column[1], orderbook.into_word());

        let calling_context = &recorded.context[CONTEXT_CALLING_CONTEXT_COLUMN];
        assert_eq!(calling_context.len(), 3);
        assert_eq!(calling_context[0], order_hash);
        assert_eq!(calling_context[1], order.owner.into_word());

        let vault_inputs = &recorded.context[CONTEXT_VAULT_INPUTS_COLUMN];
        assert_eq!(vault_inputs[0], order.validInputs[0].token.into_word());
        assert_eq!(vault_inputs[3], Float::default().get_inner());

        let vault_outputs = &recorded.context[CONTEXT_VAULT_OUTPUTS_COLUMN];
        assert_eq!(vault_outputs[3], output_balance.get_inner());
    }

    #[test]
    fn quote_applies_store_overrides() {
        let order = test_order();
        let order_hash = state::order_hash(&order);

        let outcome = host::EvalOutcome {
            stack: vec![parse_float("1").get_inner(), parse_float("1").get_inner()],
            writes: Vec::new(),
        };
        let host = Arc::new(RecordingHost::new(outcome));
        let cache = cache_with_code(&order);
        let orderbook = Address::repeat_byte(0xAB);
        let mut raindex = VirtualRaindex::new(orderbook, cache, host.clone());

        let namespace = derive_fqn(address_to_u256(order.owner), orderbook);
        let existing_key = B256::from([0xA5; 32]);
        let existing_value = B256::from([0xB6; 32]);

        raindex
            .apply_mutations(&[
                RaindexMutation::SetTokenDecimals {
                    entries: vec![
                        TokenDecimalEntry {
                            token: order.validInputs[0].token,
                            decimals: 18,
                        },
                        TokenDecimalEntry {
                            token: order.validOutputs[0].token,
                            decimals: 18,
                        },
                    ],
                },
                RaindexMutation::SetOrders {
                    orders: vec![order.clone()],
                },
                RaindexMutation::ApplyStore {
                    sets: vec![StoreSet {
                        store: order.evaluable.store,
                        fqn: namespace,
                        kvs: vec![StoreKeyValue {
                            key: existing_key,
                            value: existing_value,
                        }],
                    }],
                },
            ])
            .expect("mutations");

        let override_entry = StoreOverride {
            store: order.evaluable.store,
            fqn: namespace,
            key: B256::from([0xC7; 32]),
            value: B256::from([0xD8; 32]),
        };

        raindex
            .quote(
                new_quote_request(OrderRef::ByHash(order_hash))
                    .with_overrides(vec![override_entry]),
            )
            .expect("quote");

        let snapshot = host.last_snapshot().expect("snapshot recorded");
        assert_eq!(snapshot.len(), 2);
        assert_eq!(
            snapshot
                .get(&StoreKey::new(
                    order.evaluable.store,
                    namespace,
                    existing_key
                ))
                .copied()
                .unwrap(),
            existing_value
        );
        assert_eq!(
            snapshot
                .get(&StoreKey::new(
                    order.evaluable.store,
                    namespace,
                    override_entry.key
                ))
                .copied()
                .unwrap(),
            override_entry.value
        );
    }
}
