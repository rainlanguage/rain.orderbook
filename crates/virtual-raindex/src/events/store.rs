//! Helpers for converting interpreter store events into [`RaindexMutation`]s.

use alloy::primitives::{Address, B256};
use rain_interpreter_bindings::IInterpreterStoreV3::Set;

use crate::state::{RaindexMutation, StoreKeyValue, StoreSet};

/// Wrapper connecting a decoded store [`Set`] event with the originating
/// contract address.
#[derive(Clone, Copy, Debug)]
pub struct StoreEvent<'a> {
    /// Address of the interpreter store contract that emitted the event.
    pub store: Address,
    /// Decoded event payload.
    pub data: &'a Set,
}

/// Converts a [`StoreEvent`] into a [`RaindexMutation::ApplyStore`].
pub fn store_event_to_mutation(event: StoreEvent<'_>) -> RaindexMutation {
    let namespace: B256 = event.data.namespace.into();

    RaindexMutation::ApplyStore {
        sets: vec![StoreSet {
            store: event.store,
            fqn: namespace,
            kvs: vec![StoreKeyValue {
                key: event.data.key,
                value: event.data.value,
            }],
        }],
    }
}

/// Converts multiple [`StoreEvent`]s into [`RaindexMutation`] entries.
pub fn store_events_to_mutations<'a>(
    events: impl IntoIterator<Item = StoreEvent<'a>>,
) -> Vec<RaindexMutation> {
    events.into_iter().map(store_event_to_mutation).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};

    #[test]
    fn converts_store_event_to_mutation() {
        let data = Set {
            namespace: U256::from(42).into(),
            key: B256::from([1u8; 32]),
            value: B256::from([2u8; 32]),
        };

        let store = Address::repeat_byte(0xEE);
        let mutation = store_event_to_mutation(StoreEvent { store, data: &data });

        match mutation {
            RaindexMutation::ApplyStore { sets } => {
                assert_eq!(sets.len(), 1);
                let set = &sets[0];
                assert_eq!(set.store, store);
                assert_eq!(set.fqn, B256::from(U256::from(42)));
                assert_eq!(set.kvs.len(), 1);
                let kv = &set.kvs[0];
                assert_eq!(kv.key, data.key);
                assert_eq!(kv.value, data.value);
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }
}
