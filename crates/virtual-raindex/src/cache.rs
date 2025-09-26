use std::{collections::HashMap, sync::Arc};

use alloy::primitives::Address;
use parking_lot::RwLock;
use revm::state::Bytecode;

use crate::{BytecodeKind, RaindexError, Result};
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

pub trait CodeCache: Send + Sync {
    fn interpreter(&self, address: Address) -> Option<Arc<Bytecode>>;
    fn store(&self, address: Address) -> Option<Arc<Bytecode>>;
    fn ensure_artifacts(&self, order: &OrderV4) -> Result<()>;
}

#[derive(Default)]
pub struct StaticCodeCache {
    interpreters: RwLock<HashMap<Address, Arc<Bytecode>>>,
    stores: RwLock<HashMap<Address, Arc<Bytecode>>>,
}

impl StaticCodeCache {
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

    pub fn upsert_interpreter(&self, address: Address, bytecode: &[u8]) {
        let code = Arc::new(Bytecode::new_legacy(bytecode.to_vec().into()));
        self.interpreters.write().insert(address, code);
    }

    pub fn upsert_store(&self, address: Address, bytecode: &[u8]) {
        let code = Arc::new(Bytecode::new_legacy(bytecode.to_vec().into()));
        self.stores.write().insert(address, code);
    }
}

impl CodeCache for StaticCodeCache {
    fn interpreter(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.interpreters.read().get(&address).cloned()
    }

    fn store(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.stores.read().get(&address).cloned()
    }

    fn ensure_artifacts(&self, order: &OrderV4) -> Result<()> {
        let interpreter = Address::from(order.evaluable.interpreter);
        if self.interpreter(interpreter).is_none() {
            return Err(RaindexError::MissingBytecode {
                address: interpreter,
                kind: BytecodeKind::Interpreter,
            });
        }
        let store = Address::from(order.evaluable.store);
        if self.store(store).is_none() {
            return Err(RaindexError::MissingBytecode {
                address: store,
                kind: BytecodeKind::Store,
            });
        }
        Ok(())
    }
}
