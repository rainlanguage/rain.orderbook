//! Quote calculation pipeline for Virtual Raindex orders.

use alloy::primitives::{Address, B256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::SignedContextV1;

use crate::{
    cache::CodeCache,
    error::RaindexError,
    host,
    state::StoreKey,
    store,
};

use super::{calc::calculate_order_io, VirtualRaindex};
use super::OrderRef;

/// Temporary overlay applied to interpreter store reads during evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreOverride {
    pub store: Address,
    pub fqn: B256,
    pub key: B256,
    pub value: B256,
}

impl From<StoreOverride> for StoreKey {
    fn from(value: StoreOverride) -> Self {
        StoreKey::new(value.store, value.fqn, value.key)
    }
}

impl From<StoreOverride> for (StoreKey, B256) {
    fn from(value: StoreOverride) -> Self {
        (StoreKey::from(value), value.value)
    }
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
    /// Creates a quote request for the given order reference and IO indices.
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

    /// Sets signed context payloads to append to the interpreter context grid.
    pub fn with_signed_context(mut self, signed_context: Vec<SignedContextV1>) -> Self {
        self.signed_context = signed_context;
        self
    }

    /// Applies temporary store overrides used during evaluation.
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

/// Computes a quote for a specific IO pairing on an order reference.
pub(super) fn quote<C, H>(
    raindex: &VirtualRaindex<C, H>,
    request: QuoteRequest,
) -> crate::error::Result<Quote>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let QuoteRequest {
        order,
        input_io_index,
        output_io_index,
        counterparty,
        signed_context,
        overrides,
    } = request;

    let order = raindex.resolve_order(order)?;

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

    let mut store_snapshot = raindex.state.store.clone();
    store::apply_overrides(
        &mut store_snapshot,
        overrides.into_iter().map(Into::into),
    );

    let calculation = calculate_order_io(
        raindex,
        &raindex.state,
        &store_snapshot,
        &order,
        input_io_index,
        output_io_index,
        counterparty,
        &signed_context,
    )?;

    Ok(Quote {
        io_ratio: calculation.io_ratio,
        output_max: calculation.output_max,
        stack: calculation.stack,
        writes: calculation
            .store_writes
            .iter()
            .flat_map(|(k, v)| [*k, *v])
            .collect(),
    })
}
