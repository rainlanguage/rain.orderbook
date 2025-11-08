use alloy::primitives::{keccak256, B256};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Computes the canonical keccak256 hash for an [`OrderV4`].
#[inline]
pub fn order_hash(order: &OrderV4) -> B256 {
    keccak256(order.abi_encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn default_order_hash_is_stable() {
        let hash = order_hash(&OrderV4::default());
        assert_eq!(
            hash,
            B256::from_str("0xdcf6b886b1922d32accc60b1a0cdc53fb4bcbe74af2987b22046820030e3423b")
                .expect("hash is correct")
        );
    }
}
