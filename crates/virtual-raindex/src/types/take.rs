use alloy::primitives::{Address, B256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::SignedContextV1;

use crate::state::RaindexMutation;

use super::OrderRef;

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
    /// From the perspective of the taker.
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
