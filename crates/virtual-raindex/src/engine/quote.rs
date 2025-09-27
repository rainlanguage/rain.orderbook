//! Quote calculation pipeline for Virtual Raindex orders.

use std::collections::HashMap;

use alloy::primitives::{B256, U256};
use rain_math_float::Float;

use crate::{
    cache::CodeCache,
    error::{RaindexError, Result},
    host,
    state::{self, StoreKey, VaultKey},
    types::{Quote, QuoteRequest, StoreOverride},
};

use super::{address_to_u256, VirtualRaindex};

/// Computes a quote for a specific IO pairing on an order reference.
pub(super) fn quote<C, H>(raindex: &VirtualRaindex<C, H>, request: QuoteRequest) -> Result<Quote>
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

    let order_hash = state::order_hash(&order);

    let input_vault_balance = raindex
        .state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            input_io.token,
            input_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();
    let output_vault_balance = raindex
        .state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            output_io.token,
            output_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();

    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        input_io,
        input_decimals,
        input_vault_balance,
        output_io,
        output_decimals,
        output_vault_balance,
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
        let key = StoreKey::new(override_entry.store, override_entry.fqn, override_entry.key);
        store_snapshot.insert(key, override_entry.value);
    }
}
