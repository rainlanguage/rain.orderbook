//! Virtual Raindex core primitives and interpreter host glue.

mod cache;
mod engine;
mod error;
pub mod host;
mod state;
mod types;

pub use cache::{CodeCache, StaticCodeCache};
pub use engine::VirtualRaindex;
pub use error::{BytecodeKind, RaindexError, Result};
pub use host::RevmInterpreterHost;
pub use rain_math_float::Float;
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
