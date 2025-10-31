//! Quote calculation pipeline for Virtual Raindex orders.

use std::collections::HashMap;

use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_common::utils::order_hash;
use rain_orderbook_bindings::IOrderBookV5::SignedContextV1;

use crate::{
    cache::CodeCache,
    engine::context::IOContext,
    error::RaindexError,
    host,
    state::{self, StoreKey, VaultKey},
};

use super::{address_to_u256, VirtualRaindex};
use super::OrderRef;

/// Temporary overlay applied to interpreter store reads during evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreOverride {
    pub store: Address,
    pub fqn: B256,
    pub key: B256,
    pub value: B256,
}

impl From<StoreOverride> for StoreKey {
    fn from(value: StoreOverride) -> Self {
        StoreKey::new(value.store, value.fqn, value.key)
    }
}

impl From<StoreOverride> for (StoreKey, B256) {
    fn from(value: StoreOverride) -> Self {
        (StoreKey::from(value), value.value)
    }
}

/// Input parameters for calculating a quote against an order.
#[derive(Clone, Debug)]
pub struct QuoteRequest {
    pub order: OrderRef,
    pub input_io_index: usize,
    pub output_io_index: usize,
    pub counterparty: Address,
    pub signed_context: Vec<SignedContextV1>,
    pub overrides: Vec<StoreOverride>,
}

impl QuoteRequest {
    /// Creates a quote request for the given order reference and IO indices.
    pub fn new(
        order: OrderRef,
        input_io_index: usize,
        output_io_index: usize,
        counterparty: Address,
    ) -> Self {
        Self {
            order,
            input_io_index,
            output_io_index,
            counterparty,
            signed_context: Vec::new(),
            overrides: Vec::new(),
        }
    }

    /// Sets signed context payloads to append to the interpreter context grid.
    pub fn with_signed_context(mut self, signed_context: Vec<SignedContextV1>) -> Self {
        self.signed_context = signed_context;
        self
    }

    /// Applies temporary store overrides used during evaluation.
    pub fn with_overrides(mut self, overrides: Vec<StoreOverride>) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Result of executing calculate-io for an order.
#[derive(Clone, Debug)]
pub struct Quote {
    pub io_ratio: Float,
    pub output_max: Float,
    pub stack: Vec<B256>,
    pub writes: Vec<B256>,
}

/// Computes a quote for a specific IO pairing on an order reference.
pub(super) fn quote<C, H>(
    raindex: &VirtualRaindex<C, H>,
    request: QuoteRequest,
) -> crate::error::Result<Quote>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let QuoteRequest {
        order,
        input_io_index,
        output_io_index,
        counterparty,
        signed_context,
        overrides,
    } = request;

    let order = raindex.resolve_order(order)?;
    raindex.code_cache.ensure_artifacts(&order)?;

    let input_len = order.validInputs.len();
    if input_io_index >= input_len {
        return Err(RaindexError::InvalidInputIndex {
            index: input_io_index,
            len: input_len,
        });
    }

    let output_len = order.validOutputs.len();
    if output_io_index >= output_len {
        return Err(RaindexError::InvalidOutputIndex {
            index: output_io_index,
            len: output_len,
        });
    }

    let input_io = &order.validInputs[input_io_index];
    let output_io = &order.validOutputs[output_io_index];

    if input_io.token == output_io.token {
        return Err(RaindexError::TokenSelfTrade);
    }

    let input_decimals = *raindex.state.token_decimals.get(&input_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: input_io.token,
        },
    )?;
    let output_decimals = *raindex.state.token_decimals.get(&output_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: output_io.token,
        },
    )?;

    let order_hash = order_hash(&order);

    let input_vault_balance = raindex
        .state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            input_io.token,
            input_io.vaultId,
        ))
        .cloned()
        .ok_or(RaindexError::VaultBalanceMissing {
            owner: order.owner,
            token: input_io.token,
            vault_id: input_io.vaultId,
        })?;
    let output_vault_balance = raindex
        .state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            output_io.token,
            output_io.vaultId,
        ))
        .cloned()
        .ok_or(RaindexError::VaultBalanceMissing {
            owner: order.owner,
            token: output_io.token,
            vault_id: output_io.vaultId,
        })?;

    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        &IOContext {
            io: input_io.clone(),
            balance: input_vault_balance,
            decimals: input_decimals,
        },
        &IOContext {
            io: output_io.clone(),
            balance: output_vault_balance,
            decimals: output_decimals,
        },
        &signed_context,
    );

    let mut store_snapshot = raindex.state.store.clone();
    apply_overrides(&mut store_snapshot, overrides);

    let state_namespace = address_to_u256(order.owner);
    let qualified = state::derive_fqn(state_namespace, raindex.orderbook);
    let namespace = U256::from_be_slice(qualified.as_slice());

    let eval = rain_interpreter_bindings::IInterpreterV4::EvalV4 {
        store: order.evaluable.store,
        namespace,
        bytecode: order.evaluable.bytecode.clone(),
        sourceIndex: U256::ZERO,
        context,
        inputs: Vec::new(),
        stateOverlay: Vec::new(),
    };

    let mut outcome = raindex.interpreter_host.eval4(
        order.evaluable.interpreter,
        &eval,
        &store_snapshot,
        raindex.state.env,
    )?;

    if outcome.stack.len() < 2 {
        return Err(RaindexError::Unimplemented("calculate-io outputs"));
    }

    let io_ratio = Float::from_raw(outcome.stack[0]);
    let mut output_max = Float::from_raw(outcome.stack[1]);
    output_max = output_max.min(output_vault_balance)?;
    outcome.stack[1] = output_max.get_inner();

    Ok(Quote {
        io_ratio,
        output_max,
        stack: outcome.stack,
        writes: outcome.writes,
    })
}

/// Applies temporary store overrides to a snapshot before interpreter execution.
fn apply_overrides(store_snapshot: &mut HashMap<StoreKey, B256>, overrides: Vec<StoreOverride>) {
    for override_entry in overrides {
        let (key, value): (StoreKey, B256) = override_entry.into();
        store_snapshot.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, B256};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn apply_overrides_inserts_new_entries() {
        let override_entry = StoreOverride {
            store: Address::from([1u8; 20]),
            fqn: B256::from([2u8; 32]),
            key: B256::from([3u8; 32]),
            value: B256::from([4u8; 32]),
        };
        let mut store_snapshot = HashMap::new();

        apply_overrides(&mut store_snapshot, vec![override_entry]);

        let key = StoreKey::from(override_entry);
        assert_eq!(store_snapshot.get(&key), Some(&override_entry.value));
    }

    #[test]
    fn apply_overrides_overwrites_existing_entries() {
        let base_override = StoreOverride {
            store: Address::from([5u8; 20]),
            fqn: B256::from([6u8; 32]),
            key: B256::from([7u8; 32]),
            value: B256::from([8u8; 32]),
        };
        let mut store_snapshot = HashMap::new();
        let key = StoreKey::from(base_override);
        store_snapshot.insert(key, B256::from([9u8; 32]));

        let updated_override = StoreOverride {
            value: B256::from([10u8; 32]),
            ..base_override
        };

        apply_overrides(&mut store_snapshot, vec![updated_override]);

        assert_eq!(store_snapshot.get(&key), Some(&updated_override.value));
    }
}
