use alloy::primitives::B256;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Describes how to locate an order for quote/take operations.
#[derive(Clone, Debug)]
pub enum OrderRef {
    /// Reference an order already stored within the virtual raindex by hash.
    ByHash(B256),
    /// Provide an inline order payload without mutating virtual state.
    Inline(OrderV4),
}
