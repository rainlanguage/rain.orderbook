//! Interpreter helper utilities shared across engine flows.

use alloy::primitives::{B256, U256};
use rain_interpreter_bindings::IInterpreterV4::EvalV4;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;

/// Known Rain interpreter entrypoints used by the virtual engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EvalEntrypoint {
    CalculateIo,
    HandleIo,
}

impl EvalEntrypoint {
    pub const fn source_index(self) -> u64 {
        match self {
            Self::CalculateIo => 0,
            Self::HandleIo => 1,
        }
    }
}

/// Builds an `EvalV4` payload for a given order and interpreter entrypoint.
pub(super) fn build_eval_call(
    order: &OrderV4,
    namespace: U256,
    context: Vec<Vec<B256>>,
    entrypoint: EvalEntrypoint,
) -> EvalV4 {
    EvalV4 {
        store: order.evaluable.store,
        namespace,
        bytecode: order.evaluable.bytecode.clone(),
        sourceIndex: U256::from(entrypoint.source_index()),
        context,
        inputs: Vec::new(),
        stateOverlay: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use rain_orderbook_bindings::IOrderBookV5::{EvaluableV4, OrderV4};

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: Address::repeat_byte(0x11),
            evaluable: EvaluableV4 {
                interpreter: Address::repeat_byte(0x22),
                store: Address::repeat_byte(0x33),
                bytecode: alloy::primitives::Bytes::from(vec![0xAA, 0xBB]),
            },
            validInputs: Vec::new(),
            validOutputs: Vec::new(),
            nonce: B256::ZERO,
        }
    }

    #[test]
    fn eval_entrypoint_indices_match_expected_values() {
        assert_eq!(EvalEntrypoint::CalculateIo.source_index(), 0);
        assert_eq!(EvalEntrypoint::HandleIo.source_index(), 1);
    }

    #[test]
    fn build_eval_call_sets_entrypoint_and_context() {
        let order = sample_order();
        let namespace = U256::from(42);
        let context = vec![vec![B256::from(U256::from(1u64))]];

        let eval = build_eval_call(&order, namespace, context.clone(), EvalEntrypoint::HandleIo);

        assert_eq!(eval.store, order.evaluable.store);
        assert_eq!(eval.namespace, namespace);
        assert_eq!(eval.bytecode, order.evaluable.bytecode);
        assert_eq!(eval.context, context);
        assert_eq!(eval.inputs.len(), 0);
        assert_eq!(eval.stateOverlay.len(), 0);
        assert_eq!(
            eval.sourceIndex,
            U256::from(EvalEntrypoint::HandleIo.source_index())
        );
    }
}
