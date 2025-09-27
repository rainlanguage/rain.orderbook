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
