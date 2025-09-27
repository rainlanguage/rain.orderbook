//! Shared data structures used by the Virtual Raindex engine.

mod order_ref;
pub mod quote;
pub mod take;

pub use order_ref::OrderRef;
pub use quote::{Quote, QuoteRequest, StoreOverride};
pub use take::{TakeOrder, TakeOrderWarning, TakeOrdersConfig, TakeOrdersOutcome, TakenOrder};
