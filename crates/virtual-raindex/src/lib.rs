//! Core primitives for executing Rain Orderbook flows entirely in memory.
//!
//! The crate exposes the [`VirtualRaindex`] engine, interpreter host adapters, state
//! management utilities, and supporting types used to quote and take orders.

mod cache;
mod engine;
mod error;
pub mod events;
pub mod host;
pub mod live;
pub mod snapshot;
mod state;
mod types;

pub use cache::{CodeCache, StaticCodeCache};
pub use engine::VirtualRaindex;
pub use error::{BytecodeKind, RaindexError, Result};
pub use host::RevmInterpreterHost;
pub use rain_math_float::Float;
pub use snapshot::{CacheHandles, OrderRecord, SnapshotBundle, StoreRecord, VaultBalanceRecord};
pub use state::{
    derive_fqn, Env, RaindexMutation, Snapshot, StoreKey, StoreKeyValue, StoreSet,
    TokenDecimalEntry, VaultDelta, VaultKey,
};
pub use types::{
    OrderRef, Quote, QuoteRequest, StoreOverride, TakeOrder, TakeOrderWarning, TakeOrdersConfig,
    TakeOrdersOutcome, TakenOrder,
};

#[cfg(test)]
mod integration_tests;
