use alloy::primitives::{Address, B256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::SignedContextV1;

use super::OrderRef;

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
