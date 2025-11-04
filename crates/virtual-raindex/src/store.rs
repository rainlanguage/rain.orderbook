//! Shared helpers for managing interpreter store namespaces and overlays.

use std::collections::HashMap;

use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol_types::SolValue,
};
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

use crate::{
    error::{RaindexError, Result},
    state::StoreKey,
};

/// Derived namespace identifiers for interpreter store state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StoreNamespace {
    pub namespace: U256,
    pub qualified: B256,
}

impl StoreNamespace {
    pub const fn new(namespace: U256, qualified: B256) -> Self {
        Self { namespace, qualified }
    }
}

/// Converts an `Address` into the interpreter namespace representation.
pub(crate) fn address_to_u256(address: Address) -> U256 {
    U256::from_be_slice(address.into_word().as_slice())
}

/// Derives the fully qualified namespace for a Rain interpreter store namespace.
pub fn derive_fqn(namespace: U256, caller: Address) -> B256 {
    keccak256((namespace, caller).abi_encode())
}

/// Computes the store namespace metadata for a given order.
pub(crate) fn namespace_for_order(order: &OrderV4, orderbook: Address) -> StoreNamespace {
    let state_namespace = address_to_u256(order.owner);
    let qualified = derive_fqn(state_namespace, orderbook);
    let namespace = U256::from_be_slice(qualified.as_slice());
    StoreNamespace::new(namespace, qualified)
}

/// Converts a flat write buffer into key/value pairs, ensuring even length.
pub(crate) fn writes_to_pairs(writes: &[B256]) -> Result<Vec<(B256, B256)>> {
    if writes.len() % 2 != 0 {
        return Err(RaindexError::Unimplemented(
            "unpaired store write from interpreter",
        ));
    }

    Ok(writes.chunks(2).map(|chunk| (chunk[0], chunk[1])).collect())
}

/// Applies store writes produced by interpreter executions onto the snapshot.
pub(crate) fn apply_store_writes(
    store: &mut HashMap<StoreKey, B256>,
    store_address: Address,
    qualified: B256,
    writes: &[(B256, B256)],
) {
    for (key, value) in writes {
        store.insert(StoreKey::new(store_address, qualified, *key), *value);
    }
}

/// Applies temporary overrides to a snapshot before interpreter execution.
pub(crate) fn apply_overrides<I>(store_snapshot: &mut HashMap<StoreKey, B256>, overrides: I)
where
    I: IntoIterator<Item = (StoreKey, B256)>,
{
    for (key, value) in overrides {
        store_snapshot.insert(key, value);
    }
}

/// Builds the state overlay vector expected by the interpreter for a namespace.
pub(crate) fn build_state_overlay(
    snapshot: &HashMap<StoreKey, B256>,
    store_address: Address,
    namespace: U256,
) -> Vec<B256> {
    let namespace = B256::from(namespace.to_be_bytes());
    snapshot
        .iter()
        .filter(|(k, _)| k.store == store_address && k.fqn == namespace)
        .flat_map(|(key, value)| [key.key, *value])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Bytes;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4};

    fn sample_order(owner_byte: u8) -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(owner_byte),
            evaluable: EvaluableV4 {
                interpreter: Address::repeat_byte(0xAA),
                store: Address::repeat_byte(0xBB),
                bytecode: Bytes::from(vec![0x01, 0x02]),
            },
            validInputs: Vec::new(),
            validOutputs: Vec::new(),
            nonce: B256::ZERO,
        }
    }

    #[test]
    fn address_to_u256_encodes_big_endian_bytes() {
        let address = Address::repeat_byte(0x11);
        let encoded = address_to_u256(address);

        let mut expected_bytes = [0u8; 32];
        expected_bytes[12..].fill(0x11);
        let expected = U256::from_be_slice(&expected_bytes);

        assert_eq!(encoded, expected);
    }

    #[test]
    fn derive_fqn_matches_expected_hash() {
        let namespace = U256::from(123u64);
        let caller = Address::repeat_byte(0x44);

        let expected = keccak256((namespace, caller).abi_encode());
        assert_eq!(derive_fqn(namespace, caller), expected);
    }

    #[test]
    fn namespace_for_order_uses_owner_and_orderbook() {
        let order = sample_order(0x55);
        let orderbook = Address::repeat_byte(0x99);

        let ns = namespace_for_order(&order, orderbook);
        let expected_state = address_to_u256(order.owner);
        let expected_fqn = derive_fqn(expected_state, orderbook);

        assert_eq!(ns.namespace, U256::from_be_slice(expected_fqn.as_slice()));
        assert_eq!(ns.qualified, expected_fqn);
    }

    #[test]
    fn writes_to_pairs_splits_even_writes() {
        let writes = vec![
            B256::from(U256::from(1_u64)),
            B256::from(U256::from(2_u64)),
            B256::from(U256::from(3_u64)),
            B256::from(U256::from(4_u64)),
        ];

        let pairs = writes_to_pairs(&writes).expect("should split pairs");

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (writes[0], writes[1]));
        assert_eq!(pairs[1], (writes[2], writes[3]));
    }

    #[test]
    fn writes_to_pairs_errors_on_odd_length() {
        let writes = vec![
            B256::from(U256::from(1_u64)),
            B256::from(U256::from(2_u64)),
            B256::from(U256::from(3_u64)),
        ];

        let err = writes_to_pairs(&writes).expect_err("odd length should error");

        assert!(matches!(
            err,
            RaindexError::Unimplemented("unpaired store write from interpreter")
        ));
    }

    #[test]
    fn apply_store_writes_updates_snapshot() {
        let mut store = HashMap::new();
        let store_address = Address::repeat_byte(0xAA);
        let qualified = B256::from([0x11; 32]);
        let writes = vec![
            (B256::from(U256::from(1_u64)), B256::from(U256::from(10_u64))),
            (B256::from(U256::from(2_u64)), B256::from(U256::from(20_u64))),
        ];

        apply_store_writes(&mut store, store_address, qualified, &writes);

        for (key, value) in &writes {
            let store_key = StoreKey::new(store_address, qualified, *key);
            assert_eq!(store.get(&store_key), Some(value));
        }
    }

    #[test]
    fn apply_overrides_inserts_and_overwrites_entries() {
        let mut snapshot = HashMap::new();
        let base_key = StoreKey::new(
            Address::repeat_byte(0x01),
            B256::from([0x02; 32]),
            B256::from([0x03; 32]),
        );
        snapshot.insert(base_key, B256::from([0x04; 32]));

        let overrides = vec![
            (
                base_key,
                B256::from([0x05; 32]), // overwrite existing entry
            ),
            (
                StoreKey::new(
                    Address::repeat_byte(0x06),
                    B256::from([0x07; 32]),
                    B256::from([0x08; 32]),
                ),
                B256::from([0x09; 32]),
            ),
        ];

        apply_overrides(&mut snapshot, overrides);

        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot.get(&base_key), Some(&B256::from([0x05; 32])));
    }

    #[test]
    fn build_state_overlay_filters_by_store_and_namespace() {
        let store = Address::repeat_byte(0x11);
        let other_store = Address::repeat_byte(0x22);
        let namespace = U256::from(42);
        let other_namespace = U256::from(7);
        let namespace_fqn = B256::from(namespace.to_be_bytes());
        let other_fqn = B256::from(other_namespace.to_be_bytes());

        let key = B256::from([0xAA; 32]);
        let value = B256::from([0xBB; 32]);

        let mut snapshot = HashMap::new();
        snapshot.insert(StoreKey::new(store, namespace_fqn, key), value);
        snapshot.insert(
            StoreKey::new(other_store, namespace_fqn, key),
            B256::from([0xCC; 32]),
        );
        snapshot.insert(
            StoreKey::new(store, other_fqn, key),
            B256::from([0xDD; 32]),
        );

        let overlay = build_state_overlay(&snapshot, store, namespace);

        assert_eq!(overlay, vec![key, value]);
    }
}
