use std::sync::Arc;

use alloy::primitives::Address;

use crate::{
    cache::CodeCache,
    error::Result,
    host,
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
    pub fn new(orderbook: Address, code_cache: Arc<C>, interpreter_host: Arc<H>) -> Self {
        Self {
            state: state::RaindexState::default(),
            code_cache,
            interpreter_host,
            orderbook,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.snapshot()
    }

    pub fn interpreter(&self) -> &Arc<H> {
        &self.interpreter_host
    }

    pub fn code_cache(&self) -> &Arc<C> {
        &self.code_cache
    }

    pub fn orderbook_address(&self) -> Address {
        self.orderbook
    }

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

    pub fn take_orders(&self, config: TakeOrdersConfig) -> Result<TakeOrdersOutcome> {
        take::take_orders(self, config)
    }

    pub fn take_orders_and_apply_state(
        &mut self,
        config: TakeOrdersConfig,
    ) -> Result<TakeOrdersOutcome> {
        take::take_orders_and_apply_state(self, config)
    }

    pub fn quote(&self, request: QuoteRequest) -> Result<crate::types::Quote> {
        quote::quote(self, request)
    }

    pub fn add_order(&mut self, order: OrderV4, post_tasks: Vec<TaskV2>) -> Result<()> {
        post_tasks::add_order(self, order, post_tasks)
    }
}

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
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

pub(super) fn address_to_u256(address: Address) -> alloy::primitives::U256 {
    alloy::primitives::U256::from_be_slice(address.into_word().as_slice())
}
