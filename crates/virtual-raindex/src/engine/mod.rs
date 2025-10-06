//! Virtual Raindex execution engine and high-level orchestration helpers.

use std::sync::Arc;

use alloy::primitives::Address;

use crate::{
    cache::CodeCache,
    error::{BytecodeKind, Result},
    host,
    snapshot::{CacheHandles, SnapshotBundle},
    state::{self, RaindexMutation, Snapshot},
    types::{OrderRef, QuoteRequest, TakeOrdersConfig, TakeOrdersOutcome},
};
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, TaskV2};

pub(super) mod context;
pub(super) mod mutations;
pub(super) mod post_tasks;
pub(super) mod quote;
pub(super) mod take;

#[cfg(test)]
mod tests;

/// Virtual representation of a Rain Orderbook configured with a host interpreter.
pub struct VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    pub(super) state: state::RaindexState,
    pub(super) code_cache: Arc<C>,
    pub(super) interpreter_host: Arc<H>,
    pub(super) orderbook: Address,
}

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    /// Creates a new Virtual Raindex bound to an orderbook address and backing infrastructure.
    pub fn new(orderbook: Address, code_cache: Arc<C>, interpreter_host: Arc<H>) -> Self {
        Self {
            state: state::RaindexState::default(),
            code_cache,
            interpreter_host,
            orderbook,
        }
    }

    /// Rehydrates a Virtual Raindex from a previously captured snapshot.
    pub fn from_snapshot(
        orderbook: Address,
        snapshot: Snapshot,
        code_cache: Arc<C>,
        interpreter_host: Arc<H>,
    ) -> Self {
        Self {
            state: state::RaindexState::from_snapshot(snapshot),
            code_cache,
            interpreter_host,
            orderbook,
        }
    }

    /// Loads an engine instance from a [SnapshotBundle] and validates cache readiness.
    pub fn from_snapshot_bundle(
        bundle: SnapshotBundle,
        code_cache: Arc<C>,
        interpreter_host: Arc<H>,
    ) -> Result<Self> {
        let orderbook = bundle.orderbook;
        let handles = bundle.cache_handles().clone();
        let snapshot = bundle.into_snapshot();
        let instance = Self::from_snapshot(orderbook, snapshot, code_cache, interpreter_host);
        instance.ensure_cache_handles(&handles)?;
        Ok(instance)
    }

    /// Returns a snapshot of the current state suitable for persistence or inspection.
    pub fn snapshot(&self) -> Snapshot {
        self.state.snapshot()
    }

    /// Returns a reference to the underlying interpreter host.
    pub fn interpreter(&self) -> &Arc<H> {
        &self.interpreter_host
    }

    /// Returns a reference to the configured bytecode cache.
    pub fn code_cache(&self) -> &Arc<C> {
        &self.code_cache
    }

    /// Returns the canonical on-chain orderbook address for this instance.
    pub fn orderbook_address(&self) -> Address {
        self.orderbook
    }

    /// Applies a sequence of state mutations after verifying any required artifacts.
    pub fn apply_mutations(&mut self, mutations: &[RaindexMutation]) -> Result<()> {
        self.prepare_mutations(mutations)?;

        let mut draft = self.state.clone();
        for mutation in mutations {
            if let RaindexMutation::SetOrders { orders } = mutation {
                for order in orders {
                    self.ensure_order_context(&mut draft, order)?;
                }
            }
        }

        draft.apply_mutations(mutations)?;
        self.state = draft;
        Ok(())
    }

    /// Executes a read-only take orders simulation returning the computed outcome.
    pub fn take_orders(&self, config: TakeOrdersConfig) -> Result<TakeOrdersOutcome> {
        take::take_orders(self, config)
    }

    /// Executes take orders and applies the resulting state mutations if successful.
    pub fn take_orders_and_apply_state(
        &mut self,
        config: TakeOrdersConfig,
    ) -> Result<TakeOrdersOutcome> {
        take::take_orders_and_apply_state(self, config)
    }

    /// Produces a quote for a single input/output IO pair on an order reference.
    pub fn quote(&self, request: QuoteRequest) -> Result<crate::types::Quote> {
        quote::quote(self, request)
    }

    /// Adds an order and executes any provided post tasks mutating state atomically.
    pub fn add_order(&mut self, order: OrderV4, post_tasks: Vec<TaskV2>) -> Result<()> {
        post_tasks::add_order(self, order, post_tasks)
    }
}

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    /// Resolves an [OrderRef] into a fully materialized `OrderV4`.
    pub(super) fn resolve_order(&self, reference: OrderRef) -> Result<OrderV4> {
        match reference {
            OrderRef::Inline(order) => Ok(order),
            OrderRef::ByHash(hash) => self
                .state
                .orders
                .get(&hash)
                .cloned()
                .ok_or(crate::error::RaindexError::OrderNotFound { order_hash: hash }),
        }
    }

    fn ensure_cache_handles(&self, handles: &CacheHandles) -> Result<()> {
        for address in &handles.interpreters {
            if self.code_cache.interpreter(*address).is_none() {
                return Err(crate::error::RaindexError::MissingBytecode {
                    address: *address,
                    kind: BytecodeKind::Interpreter,
                });
            }
        }

        for address in &handles.stores {
            if self.code_cache.store(*address).is_none() {
                return Err(crate::error::RaindexError::MissingBytecode {
                    address: *address,
                    kind: BytecodeKind::Store,
                });
            }
        }

        Ok(())
    }
}

/// Converts an address into the `U256` namespace representation used by interpreter state.
pub(super) fn address_to_u256(address: Address) -> alloy::primitives::U256 {
    alloy::primitives::U256::from_be_slice(address.into_word().as_slice())
}
