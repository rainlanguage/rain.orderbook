//! Helpers for constructing interpreter context grids passed to Rain evals.

use alloy::primitives::{Address, B256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1, IOV2};
use rain_orderbook_common::utils::order_hash;

use crate::{cache::CodeCache, host};

use super::{u8_to_b256, VirtualRaindex};

pub(super) const CALLING_CONTEXT_COLUMNS: usize = 4;
pub(super) const CONTEXT_CALLING_CONTEXT_COLUMN: usize = 1;
pub(super) const CONTEXT_CALCULATIONS_COLUMN: usize = 2;
pub(super) const CONTEXT_VAULT_INPUTS_COLUMN: usize = 3;
pub(super) const CONTEXT_VAULT_OUTPUTS_COLUMN: usize = 4;
pub(super) const CONTEXT_VAULT_IO_BALANCE_DIFF_ROW: usize = 5;

pub(super) struct IOContext {
    pub(super) io: IOV2,
    pub(super) balance: Float,
    pub(super) decimals: u8,
}

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    /// Constructs the base context for order post tasks.
    pub(super) fn build_post_context(&self, order: &OrderV4) -> Vec<Vec<B256>> {
        let order_hash = order_hash(order);
        vec![vec![order_hash, order.owner.into_word()]]
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_quote_context(
        &self,
        order_hash: B256,
        owner: Address,
        counterparty: Address,
        input: &IOContext,
        output: &IOContext,
        signed_context: &[SignedContextV1],
    ) -> Vec<Vec<B256>> {
        let mut base_columns = vec![Vec::new(); CALLING_CONTEXT_COLUMNS];

        base_columns[CONTEXT_CALLING_CONTEXT_COLUMN - 1] =
            vec![order_hash, owner.into_word(), counterparty.into_word()];

        base_columns[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![B256::ZERO; 2];

        let input_io = &input.io;
        base_columns[CONTEXT_VAULT_INPUTS_COLUMN - 1] = vec![
            input_io.token.into_word(),
            u8_to_b256(input.decimals),
            input_io.vaultId,
            input.balance.get_inner(),
            B256::ZERO,
        ];

        let output_io = &output.io;
        base_columns[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = vec![
            output_io.token.into_word(),
            u8_to_b256(output.decimals),
            output_io.vaultId,
            output.balance.get_inner(),
            B256::ZERO,
        ];

        self.build_context(base_columns, signed_context, counterparty)
    }

    /// Finalizes the context grid by prepending global metadata and signed blobs.
    pub(super) fn build_context(
        &self,
        base_columns: Vec<Vec<B256>>,
        signed_context: &[SignedContextV1],
        counterparty: Address,
    ) -> Vec<Vec<B256>> {
        let mut context = Vec::with_capacity(
            1 + base_columns.len()
                + if signed_context.is_empty() {
                    0
                } else {
                    signed_context.len() + 1
                },
        );

        context.push(vec![counterparty.into_word(), self.orderbook.into_word()]);
        context.extend(base_columns);

        if !signed_context.is_empty() {
            let signers: Vec<_> = signed_context
                .iter()
                .map(|sc| sc.signer.into_word())
                .collect();
            context.push(signers);

            context.extend(signed_context.iter().map(|sc| sc.context.clone()));
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{IOContext, VirtualRaindex};
    use crate::{
        cache::CodeCache,
        error::{RaindexError, Result},
        host::{self, InterpreterHost},
        state::{self, StoreKey},
    };
    use alloy::primitives::{Address, B256};
    use rain_interpreter_bindings::IInterpreterV4::EvalV4;
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

    fn new_raindex() -> VirtualRaindex<NullCache, NullHost> {
        let orderbook = Address::repeat_byte(0xAB);
        VirtualRaindex::new(orderbook, Arc::new(NullCache), Arc::new(NullHost))
    }

    fn parse_float(value: &str) -> Float {
        Float::parse(value.to_owned()).expect("float parse")
    }

    fn make_io_context(token_byte: u8, vault_byte: u8, decimals: u8, balance: &str) -> IOContext {
        IOContext {
            io: IOV2 {
                token: Address::repeat_byte(token_byte),
                vaultId: B256::from([vault_byte; 32]),
            },
            balance: parse_float(balance),
            decimals,
        }
    }

    #[test]
    fn quote_context_populates_expected_columns() {
        let raindex = new_raindex();
        let order_hash = B256::from([0x11; 32]);
        let owner = Address::repeat_byte(0x22);
        let counterparty = Address::repeat_byte(0x33);
        let input = make_io_context(0x44, 0x55, 6, "123.5");
        let output = make_io_context(0x66, 0x77, 8, "42.25");

        let context =
            raindex.build_quote_context(order_hash, owner, counterparty, &input, &output, &[]);

        assert_eq!(context.len(), 1 + CALLING_CONTEXT_COLUMNS);
        assert_eq!(
            context[0],
            vec![counterparty.into_word(), raindex.orderbook.into_word()]
        );
        assert_eq!(
            context[CONTEXT_CALLING_CONTEXT_COLUMN],
            vec![order_hash, owner.into_word(), counterparty.into_word()]
        );
        assert_eq!(
            context[CONTEXT_CALCULATIONS_COLUMN],
            vec![B256::ZERO, B256::ZERO]
        );

        let expected_input = vec![
            input.io.token.into_word(),
            u8_to_b256(input.decimals),
            input.io.vaultId,
            input.balance.clone().get_inner(),
            B256::ZERO,
        ];
        assert_eq!(context[CONTEXT_VAULT_INPUTS_COLUMN], expected_input);

        let expected_output = vec![
            output.io.token.into_word(),
            u8_to_b256(output.decimals),
            output.io.vaultId,
            output.balance.clone().get_inner(),
            B256::ZERO,
        ];
        assert_eq!(context[CONTEXT_VAULT_OUTPUTS_COLUMN], expected_output);
    }

    #[test]
    fn quote_context_appends_signed_rows() {
        let raindex = new_raindex();
        let order_hash = B256::from([0x01; 32]);
        let owner = Address::repeat_byte(0x02);
        let counterparty = Address::repeat_byte(0x03);
        let input = make_io_context(0x04, 0x05, 7, "1.0");
        let output = make_io_context(0x06, 0x07, 9, "2.0");
        let signed_context = vec![
            SignedContextV1 {
                signer: Address::repeat_byte(0x10),
                context: vec![B256::from([0xAA; 32])],
                signature: B256::from([0xCC; 32]).into(),
            },
            SignedContextV1 {
                signer: Address::repeat_byte(0x20),
                context: vec![B256::from([0xBB; 32])],
                signature: B256::from([0xDD; 32]).into(),
            },
        ];

        let context = raindex.build_quote_context(
            order_hash,
            owner,
            counterparty,
            &input,
            &output,
            &signed_context,
        );

        let signers_row_index = 1 + CALLING_CONTEXT_COLUMNS;
        assert_eq!(
            context[signers_row_index],
            signed_context
                .iter()
                .map(|sc| sc.signer.into_word())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            context[signers_row_index + 1],
            signed_context[0].context.clone()
        );
        assert_eq!(
            context[signers_row_index + 2],
            signed_context[1].context.clone()
        );
    }
}
