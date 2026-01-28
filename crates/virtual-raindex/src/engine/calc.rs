//! Shared helpers for running calculate-io style interpreter entrypoints.

use std::collections::HashMap;

use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1};
use rain_orderbook_common::utils::order_hash::order_hash;

use crate::{
    cache::CodeCache,
    error::{RaindexError, Result},
    host,
    state::{self, StoreKey, VaultKey},
    store::{namespace_for_order, StoreNamespace},
};

use super::{
    context::{IOContext, CONTEXT_CALCULATIONS_COLUMN},
    eval::{build_eval_call, EvalEntrypoint},
    VirtualRaindex,
};

/// Output of a calculate-io interpreter call.
#[derive(Clone)]
pub(super) struct OrderCalculation {
    pub(super) order: OrderV4,
    pub(super) io_ratio: Float,
    pub(super) output_max: Float,
    pub(super) context: Vec<Vec<B256>>,
    pub(super) stack: Vec<B256>,
    pub(super) store_writes: Vec<(B256, B256)>,
    pub(super) namespace: U256,
    pub(super) qualified_namespace: B256,
    pub(super) store: Address,
}

#[derive(Debug)]
struct CalculateIoPrep {
    context: Vec<Vec<B256>>,
    output_balance: Float,
}

#[allow(clippy::too_many_arguments)]
/// Executes calculate-io using the provided state snapshot and returns the interpreter outcome.
pub(super) fn calculate_order_io<C, H>(
    raindex: &VirtualRaindex<C, H>,
    working_state: &state::RaindexState,
    store_snapshot: &HashMap<StoreKey, B256>,
    order: &OrderV4,
    input_io_index: usize,
    output_io_index: usize,
    counterparty: Address,
    signed_context: &[SignedContextV1],
) -> Result<OrderCalculation>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    let CalculateIoPrep {
        context,
        output_balance,
    } = prepare_calculate_io(
        raindex,
        working_state,
        order,
        input_io_index,
        output_io_index,
        counterparty,
        signed_context,
    )?;

    let StoreNamespace {
        namespace,
        qualified: qualified_namespace,
    } = namespace_for_order(order, raindex.orderbook);

    let eval = build_eval_call(
        order,
        namespace,
        context.clone(),
        EvalEntrypoint::CalculateIo,
    );

    let outcome = raindex.interpreter_host.eval4(
        order.evaluable.interpreter,
        &eval,
        store_snapshot,
        raindex.state.env,
    )?;

    if outcome.stack.len() < 2 {
        return Err(RaindexError::CalculateIoOutputsMissing {
            stack_len: outcome.stack.len(),
        });
    }

    let mut stack = outcome.stack;
    let io_ratio = Float::from_raw(stack[0]);
    let mut output_max = Float::from_raw(stack[1]);
    output_max = output_max.min(output_balance)?;
    stack[1] = output_max.get_inner();

    let mut context = context;
    context[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![output_max.get_inner(), io_ratio.get_inner()];

    let store_writes = crate::store::writes_to_pairs(&outcome.writes)?;

    Ok(OrderCalculation {
        order: order.clone(),
        io_ratio,
        output_max,
        context,
        stack,
        store_writes,
        namespace,
        qualified_namespace,
        store: order.evaluable.store,
    })
}

#[allow(clippy::too_many_arguments)]
fn prepare_calculate_io<C, H>(
    raindex: &VirtualRaindex<C, H>,
    working_state: &state::RaindexState,
    order: &OrderV4,
    input_io_index: usize,
    output_io_index: usize,
    counterparty: Address,
    signed_context: &[SignedContextV1],
) -> Result<CalculateIoPrep>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    raindex.code_cache.ensure_artifacts(order)?;

    let input_io = &order.validInputs[input_io_index];
    let output_io = &order.validOutputs[output_io_index];

    let input_decimals = *working_state.token_decimals.get(&input_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: input_io.token,
        },
    )?;
    let output_decimals = *working_state.token_decimals.get(&output_io.token).ok_or(
        RaindexError::TokenDecimalMissing {
            token: output_io.token,
        },
    )?;

    let input_balance = working_state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            input_io.token,
            input_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();
    let output_balance = working_state
        .vault_balances
        .get(&VaultKey::new(
            order.owner,
            output_io.token,
            output_io.vaultId,
        ))
        .cloned()
        .unwrap_or_default();

    let order_hash = order_hash(order);
    let context = raindex.build_quote_context(
        order_hash,
        order.owner,
        counterparty,
        &IOContext {
            io: input_io.clone(),
            balance: input_balance,
            decimals: input_decimals,
        },
        &IOContext {
            io: output_io.clone(),
            balance: output_balance,
            decimals: output_decimals,
        },
        signed_context,
    );

    Ok(CalculateIoPrep {
        context,
        output_balance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CodeCache,
        error::{RaindexError, Result},
        host::{self, InterpreterHost},
        state::{self, StoreKey, VaultKey},
    };
    use alloy::primitives::{Address, Bytes};
    use rain_interpreter_bindings::IInterpreterV4::EvalV4;
    use rain_math_float::Float;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, IOV2};
    use std::{collections::HashMap, sync::Arc};

    #[derive(Default)]
    struct TestCache;

    impl CodeCache for TestCache {
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
    struct FailingHost;

    impl InterpreterHost for FailingHost {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            Err(RaindexError::Unimplemented("failing host"))
        }
    }

    struct FixedHost {
        outcome: host::EvalOutcome,
    }

    impl FixedHost {
        fn new(outcome: host::EvalOutcome) -> Self {
            Self { outcome }
        }
    }

    impl InterpreterHost for FixedHost {
        fn eval4(
            &self,
            _interpreter: Address,
            _eval: &EvalV4,
            _store_snapshot: &HashMap<StoreKey, B256>,
            _env: state::Env,
        ) -> Result<host::EvalOutcome> {
            Ok(self.outcome.clone())
        }
    }

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(0x42),
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

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_owned()).expect("float parse")
    }

    #[test]
    fn prepare_calculate_io_requires_token_decimals() {
        let cache = Arc::new(TestCache);
        let host = Arc::new(FailingHost);
        let raindex = VirtualRaindex::new(Address::ZERO, cache, host);
        let order = sample_order();
        let working_state = state::RaindexState::default();

        let err = prepare_calculate_io(
            &raindex,
            &working_state,
            &order,
            0,
            0,
            Address::repeat_byte(0xEE),
            &[],
        )
        .expect_err("missing token decimals should error");

        match err {
            RaindexError::TokenDecimalMissing { token } => {
                assert_eq!(token, order.validInputs[0].token);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn calculate_order_io_clamps_output_and_updates_context() {
        let cache = Arc::new(TestCache);
        let ratio = parse_float("1.5");
        let host_output_max = parse_float("10");
        let writes = vec![B256::repeat_byte(0xAA), B256::repeat_byte(0xBB)];
        let host = Arc::new(FixedHost::new(host::EvalOutcome {
            stack: vec![ratio.get_inner(), host_output_max.get_inner()],
            writes: writes.clone(),
        }));

        let raindex = VirtualRaindex::new(Address::repeat_byte(0x01), cache, host);
        let order = sample_order();
        let mut working_state = state::RaindexState::default();

        working_state
            .token_decimals
            .insert(order.validInputs[0].token, 18);
        working_state
            .token_decimals
            .insert(order.validOutputs[0].token, 6);

        working_state.vault_balances.insert(
            VaultKey::new(
                order.owner,
                order.validInputs[0].token,
                order.validInputs[0].vaultId,
            ),
            parse_float("100"),
        );
        working_state.vault_balances.insert(
            VaultKey::new(
                order.owner,
                order.validOutputs[0].token,
                order.validOutputs[0].vaultId,
            ),
            parse_float("5"),
        );

        let store_snapshot = HashMap::<StoreKey, B256>::new();
        let result = calculate_order_io(
            &raindex,
            &working_state,
            &store_snapshot,
            &order,
            0,
            0,
            Address::repeat_byte(0xDD),
            &[],
        )
        .expect("calculate_order_io should succeed");

        let clamped = parse_float("5");

        assert_eq!(result.io_ratio.get_inner(), ratio.get_inner());
        assert_eq!(result.output_max.get_inner(), clamped.get_inner());
        assert_eq!(result.stack[0], ratio.get_inner());
        assert_eq!(result.stack[1], clamped.get_inner());
        assert_eq!(
            result.context[CONTEXT_CALCULATIONS_COLUMN - 1],
            vec![clamped.get_inner(), ratio.get_inner()]
        );
        assert_eq!(result.store_writes, vec![(writes[0], writes[1])]);
    }
}
