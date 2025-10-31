use alloy::primitives::{keccak256, B256};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Computes the canonical keccak256 hash for an [`OrderV4`].
pub fn order_hash(order: &OrderV4) -> B256 {
    keccak256(order.abi_encode())
}
