//! Virtual Raindex execution engine and high-level orchestration helpers.

use std::sync::Arc;

use alloy::primitives::{Address, B256, U256};

use crate::{
    cache::CodeCache,
    error::Result,
    host,
    state::{self, RaindexMutation, Snapshot},
};
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, TaskV2};

pub(super) mod context;
pub(super) mod calc;
pub(super) mod eval;
pub(super) mod mutations;
pub(super) mod post_tasks;
pub(super) mod quote;
pub(super) mod take;

pub use self::{
    quote::{Quote, QuoteRequest, StoreOverride},
    take::{TakeOrder, TakeOrderWarning, TakeOrdersConfig, TakeOrdersOutcome, TakenOrder},
};

#[cfg(test)]
mod tests;

/// Describes how to locate an order for quote/take operations.
#[derive(Clone, Debug)]
pub enum OrderRef {
    /// Reference an order already stored within the virtual raindex by hash.
    ByHash(B256),
    /// Provide an inline order payload without mutating virtual state.
    Inline(OrderV4),
}

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
    pub fn quote(&self, request: QuoteRequest) -> Result<Quote> {
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
}

/// Convenience helper for encoding a `u8` value as a `B256` word.
pub(super) fn u8_to_b256(value: u8) -> B256 {
    B256::from(U256::from(value))
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{
        cache::CodeCache,
        error::{RaindexError, Result},
        host::{self, InterpreterHost},
        state::{self, StoreKey},
    };
    use alloy::primitives::{Address, Bytes, B256};
    use rain_interpreter_bindings::IInterpreterV4::EvalV4;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4, IOV2};
    use rain_orderbook_common::utils::order_hash;
    use std::{collections::HashMap, sync::Arc};

    #[derive(Default)]
    struct NullCache;

    impl CodeCache for NullCache {
        fn interpreter(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
            None
        }

        fn store(&self, _address: Address) -> Option<Arc<revm::state::Bytecode>> {
            None
        }

        fn ensure_artifacts(&self, _order: &OrderV4) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct NullHost;

    impl InterpreterHost for NullHost {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            Err(RaindexError::Unimplemented("test interpreter host"))
        }
    }

    fn test_raindex() -> VirtualRaindex<NullCache, NullHost> {
        let orderbook = Address::repeat_byte(0xAB);
        VirtualRaindex::new(
            orderbook,
            Arc::new(NullCache::default()),
            Arc::new(NullHost::default()),
        )
    }

    fn sample_order(owner_byte: u8) -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(owner_byte),
            evaluable: EvaluableV4 {
                interpreter: Address::repeat_byte(0xAA),
                store: Address::repeat_byte(0xBB),
                bytecode: Bytes::from(vec![0u8]),
            },
            validInputs: vec![IOV2 {
                token: Address::repeat_byte(0x10),
                vaultId: B256::from([1u8; 32]),
            }],
            validOutputs: vec![IOV2 {
                token: Address::repeat_byte(0x20),
                vaultId: B256::from([2u8; 32]),
            }],
            nonce: B256::ZERO,
        }
    }

    #[test]
    fn resolve_order_returns_inline_order() {
        let raindex = test_raindex();
        let order = sample_order(0x44);

        let resolved = raindex
            .resolve_order(OrderRef::Inline(order.clone()))
            .expect("inline order resolves");

        assert_eq!(resolved.owner, order.owner);
        assert_eq!(resolved.validInputs[0].token, order.validInputs[0].token);
        assert_eq!(resolved.validOutputs[0].token, order.validOutputs[0].token);
    }

    #[test]
    fn resolve_order_loads_from_state_by_hash() {
        let mut raindex = test_raindex();
        let order = sample_order(0x55);
        let order_hash = order_hash(&order);
        raindex.state.orders.insert(order_hash, order.clone());

        let resolved = raindex
            .resolve_order(OrderRef::ByHash(order_hash))
            .expect("hash-backed order resolves");

        assert_eq!(resolved.owner, order.owner);
        assert_eq!(
            resolved.validInputs[0].vaultId,
            order.validInputs[0].vaultId
        );
    }

    #[test]
    fn resolve_order_errors_when_missing() {
        let raindex = test_raindex();
        let missing_hash = B256::from([0xFFu8; 32]);

        let err = raindex
            .resolve_order(OrderRef::ByHash(missing_hash))
            .expect_err("missing order should error");

        match err {
            RaindexError::OrderNotFound { order_hash } => assert_eq!(order_hash, missing_hash),
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn u8_to_b256_places_value_in_low_byte() {
        let encoded = u8_to_b256(42);

        let mut expected_bytes = [0u8; 32];
        expected_bytes[31] = 42;

        assert_eq!(encoded, B256::from(expected_bytes));
    }
}
