//! Bytecode caching utilities for the Virtual Raindex engine.

use std::{collections::HashMap, sync::Arc};

use alloy::primitives::Address;
use parking_lot::RwLock;
use revm::state::Bytecode;

use crate::{BytecodeKind, RaindexError, Result};
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Shared interface for loading interpreter and store bytecode.
pub trait CodeCache: Send + Sync {
    /// Returns the interpreter bytecode for an address if it is available.
    fn interpreter(&self, address: Address) -> Option<Arc<Bytecode>>;
    /// Returns the store bytecode for an address if it is available.
    fn store(&self, address: Address) -> Option<Arc<Bytecode>>;
    /// Ensures the bytecode required by an order is cached, returning an error if
    /// either artifact is missing.
    fn ensure_artifacts(&self, order: &OrderV4) -> Result<()>;
}

/// In-memory bytecode cache backed by hash maps.
#[derive(Default)]
pub struct StaticCodeCache {
    interpreters: RwLock<HashMap<Address, Arc<Bytecode>>>,
    stores: RwLock<HashMap<Address, Arc<Bytecode>>>,
}

impl StaticCodeCache {
    /// Instantiates a cache pre-populated with a single interpreter/store pair.
    pub fn with_pair(
        interpreter: Address,
        interpreter_code: &[u8],
        store: Address,
        store_code: &[u8],
    ) -> Self {
        let cache = Self::default();
        cache.upsert_interpreter(interpreter, interpreter_code);
        cache.upsert_store(store, store_code);
        cache
    }

    /// Inserts or replaces the interpreter bytecode for an address.
    pub fn upsert_interpreter(&self, address: Address, bytecode: &[u8]) {
        let code = Arc::new(Bytecode::new_legacy(bytecode.to_vec().into()));
        self.interpreters.write().insert(address, code);
    }

    /// Inserts or replaces the store bytecode for an address.
    pub fn upsert_store(&self, address: Address, bytecode: &[u8]) {
        let code = Arc::new(Bytecode::new_legacy(bytecode.to_vec().into()));
        self.stores.write().insert(address, code);
    }
}

impl CodeCache for StaticCodeCache {
    /// Retrieves interpreter bytecode from the cache.
    fn interpreter(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.interpreters.read().get(&address).cloned()
    }

    /// Retrieves store bytecode from the cache.
    fn store(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.stores.read().get(&address).cloned()
    }

    /// Confirms the interpreter and store bytecode used by an order are present.
    fn ensure_artifacts(&self, order: &OrderV4) -> Result<()> {
        let interpreter = order.evaluable.interpreter;
        if self.interpreter(interpreter).is_none() {
            return Err(RaindexError::MissingBytecode {
                address: interpreter,
                kind: BytecodeKind::Interpreter,
            });
        }
        let store = order.evaluable.store;
        if self.store(store).is_none() {
            return Err(RaindexError::MissingBytecode {
                address: store,
                kind: BytecodeKind::Store,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256};
    use rain_orderbook_bindings::IOrderBookV5::EvaluableV4;

    fn make_order(interpreter: Address, store: Address) -> OrderV4 {
        OrderV4 {
            owner: Address::ZERO,
            evaluable: EvaluableV4 {
                interpreter,
                store,
                bytecode: Default::default(),
            },
            validInputs: Vec::new(),
            validOutputs: Vec::new(),
            nonce: B256::ZERO,
        }
    }

    fn sample_address(byte: u8) -> Address {
        Address::from([byte; 20])
    }

    fn assert_cached_code(actual: &[u8], expected: &[u8]) {
        assert!(
            actual.starts_with(expected),
            "cached bytecode {actual:?} should begin with original {expected:?}"
        );

        match actual.len().cmp(&expected.len()) {
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                assert_eq!(
                    actual.len(),
                    expected.len() + 1,
                    "cached bytecode should only contain a trailing pad byte"
                );
                assert_eq!(
                    actual.last(),
                    Some(&0),
                    "cached bytecode trailing byte should be zero padding"
                );
            }
            std::cmp::Ordering::Less => {
                panic!("cached bytecode {actual:?} shorter than original {expected:?}")
            }
        }
    }

    #[test]
    fn with_pair_populates_cache() {
        let interpreter = sample_address(0x01);
        let store = sample_address(0x02);
        let interpreter_code = vec![0xAA, 0xBB, 0xCC];
        let store_code = vec![0x11, 0x22];

        let cache = StaticCodeCache::with_pair(interpreter, &interpreter_code, store, &store_code);

        let cached_interpreter = cache
            .interpreter(interpreter)
            .expect("interpreter bytecode should be cached");
        assert_cached_code(
            cached_interpreter.bytecode().as_ref(),
            interpreter_code.as_slice(),
        );

        let cached_store = cache.store(store).expect("store bytecode should be cached");
        assert_cached_code(cached_store.bytecode().as_ref(), store_code.as_slice());
    }

    #[test]
    fn ensure_artifacts_errors_when_interpreter_missing() {
        let interpreter = sample_address(0x03);
        let store = sample_address(0x04);
        let store_code = vec![0x33, 0x44];

        let cache = StaticCodeCache::default();
        cache.upsert_store(store, &store_code);

        let order = make_order(interpreter, store);
        let result = cache.ensure_artifacts(&order);

        match result {
            Err(RaindexError::MissingBytecode {
                address,
                kind: BytecodeKind::Interpreter,
            }) => assert_eq!(address, interpreter),
            other => panic!("expected interpreter missing error, got {other:?}"),
        }
    }

    #[test]
    fn ensure_artifacts_errors_when_store_missing() {
        let interpreter = sample_address(0x05);
        let store = sample_address(0x06);
        let interpreter_code = vec![0x55, 0x66];

        let cache = StaticCodeCache::default();
        cache.upsert_interpreter(interpreter, &interpreter_code);

        let order = make_order(interpreter, store);
        let result = cache.ensure_artifacts(&order);

        match result {
            Err(RaindexError::MissingBytecode {
                address,
                kind: BytecodeKind::Store,
            }) => assert_eq!(address, store),
            other => panic!("expected store missing error, got {other:?}"),
        }
    }

    #[test]
    fn ensure_artifacts_succeeds_when_both_present() {
        let interpreter = sample_address(0x07);
        let store = sample_address(0x08);
        let interpreter_code = vec![0x77, 0x88];
        let store_code = vec![0x99, 0xAA];

        let cache = StaticCodeCache::default();
        cache.upsert_interpreter(interpreter, &interpreter_code);
        cache.upsert_store(store, &store_code);

        let order = make_order(interpreter, store);
        assert!(cache.ensure_artifacts(&order).is_ok());
    }
}
