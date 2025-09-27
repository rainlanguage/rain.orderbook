//! Pre-processing for state mutations applied to the Virtual Raindex.

use rain_orderbook_bindings::IOrderBookV5::OrderV4;

use crate::{
    cache::CodeCache,
    error::Result,
    state::{self, RaindexMutation, VaultKey},
};

use super::VirtualRaindex;

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: crate::host::InterpreterHost,
{
    /// Ensures all resources referenced in the mutation batch are available before applying them.
    pub(super) fn prepare_mutations(&self, mutations: &[RaindexMutation]) -> Result<()> {
        for mutation in mutations {
            match mutation {
                RaindexMutation::SetOrders { orders } => {
                    for order in orders {
                        self.code_cache.ensure_artifacts(order)?;
                    }
                }
                RaindexMutation::Batch(batch) => self.prepare_mutations(batch)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Ensures the state has baseline entries needed to process an order.
    pub(super) fn ensure_order_context(
        &self,
        state: &mut state::RaindexState,
        order: &OrderV4,
    ) -> Result<()> {
        ensure_vault_entries(state, order);
        Ok(())
    }
}

/// Adds empty vault balance entries for every output vault referenced by the order.
pub(super) fn ensure_vault_entries(state: &mut state::RaindexState, order: &OrderV4) {
    for io in &order.validOutputs {
        state
            .vault_balances
            .entry(VaultKey::new(order.owner, io.token, io.vaultId))
            .or_default();
    }
}
