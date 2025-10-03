//! Bytecode caching utilities for the Virtual Raindex engine.

use std::{collections::HashMap, sync::Arc};

use alloy::primitives::{keccak256, Address, B256};
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
    interpreters: RwLock<HashMap<Address, CachedBytecode>>,
    stores: RwLock<HashMap<Address, CachedBytecode>>,
}

impl StaticCodeCache {
    /// Instantiates a cache pre-populated with a single interpreter/store pair.
    pub fn with_pair(
        interpreter: Address,
        interpreter_code: impl AsRef<[u8]>,
        store: Address,
        store_code: impl AsRef<[u8]>,
    ) -> Result<Self> {
        let cache = Self::default();
        cache.upsert_interpreter(interpreter, interpreter_code)?;
        cache.upsert_store(store, store_code)?;
        Ok(cache)
    }

    /// Instantiates a cache using complete interpreter/store collections.
    pub fn from_artifacts<IA, IS, BI, BS>(interpreters: IA, stores: IS) -> Result<Self>
    where
        IA: IntoIterator<Item = (Address, BI)>,
        IS: IntoIterator<Item = (Address, BS)>,
        BI: AsRef<[u8]>,
        BS: AsRef<[u8]>,
    {
        let cache = Self::default();
        cache.ingest_interpreters(interpreters)?;
        cache.ingest_stores(stores)?;
        Ok(cache)
    }

    /// Inserts or replaces the interpreter bytecode for an address.
    pub fn upsert_interpreter(&self, address: Address, bytecode: impl AsRef<[u8]>) -> Result<()> {
        ingest(
            &self.interpreters,
            address,
            bytecode.as_ref(),
            BytecodeKind::Interpreter,
        )
    }

    /// Inserts or replaces the store bytecode for an address.
    pub fn upsert_store(&self, address: Address, bytecode: impl AsRef<[u8]>) -> Result<()> {
        ingest(
            &self.stores,
            address,
            bytecode.as_ref(),
            BytecodeKind::Store,
        )
    }

    /// Bulk-ingests interpreter bytecode.
    pub fn ingest_interpreters<I, B>(&self, interpreters: I) -> Result<()>
    where
        I: IntoIterator<Item = (Address, B)>,
        B: AsRef<[u8]>,
    {
        for (address, bytes) in interpreters {
            self.upsert_interpreter(address, bytes)?;
        }
        Ok(())
    }

    /// Bulk-ingests store bytecode.
    pub fn ingest_stores<I, B>(&self, stores: I) -> Result<()>
    where
        I: IntoIterator<Item = (Address, B)>,
        B: AsRef<[u8]>,
    {
        for (address, bytes) in stores {
            self.upsert_store(address, bytes)?;
        }
        Ok(())
    }
}

impl CodeCache for StaticCodeCache {
    /// Retrieves interpreter bytecode from the cache.
    fn interpreter(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.interpreters
            .read()
            .get(&address)
            .map(|cached| cached.code.clone())
    }

    /// Retrieves store bytecode from the cache.
    fn store(&self, address: Address) -> Option<Arc<Bytecode>> {
        self.stores
            .read()
            .get(&address)
            .map(|cached| cached.code.clone())
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

struct CachedBytecode {
    code: Arc<Bytecode>,
    hash: B256,
}

fn ingest(
    map: &RwLock<HashMap<Address, CachedBytecode>>,
    address: Address,
    bytes: &[u8],
    kind: BytecodeKind,
) -> Result<()> {
    if bytes.is_empty() {
        return Err(RaindexError::InvalidBytecodeEncoding { address, kind });
    }

    let hash = keccak256(bytes);
    let mut guard = map.write();
    if let Some(existing) = guard.get(&address) {
        if existing.hash != hash {
            return Err(RaindexError::BytecodeCollision { address, kind });
        }
        return Ok(());
    }

    let byte_vec = bytes.to_vec();
    let code = Arc::new(Bytecode::new_legacy(byte_vec.into()));
    guard.insert(address, CachedBytecode { code, hash });
    Ok(())
}
