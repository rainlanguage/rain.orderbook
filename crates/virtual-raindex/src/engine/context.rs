use alloy::primitives::{Address, B256, U256};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{OrderV4, SignedContextV1, IOV2};

use crate::{cache::CodeCache, host, state};

use super::{address_to_u256, VirtualRaindex};

pub(super) const CALLING_CONTEXT_COLUMNS: usize = 4;
pub(super) const CONTEXT_CALLING_CONTEXT_COLUMN: usize = 1;
pub(super) const CONTEXT_CALCULATIONS_COLUMN: usize = 2;
pub(super) const CONTEXT_VAULT_INPUTS_COLUMN: usize = 3;
pub(super) const CONTEXT_VAULT_OUTPUTS_COLUMN: usize = 4;
pub(super) const HANDLE_IO_ENTRYPOINT: u64 = 1;
pub(super) const CONTEXT_VAULT_IO_BALANCE_DIFF_ROW: usize = 5;

impl<C, H> VirtualRaindex<C, H>
where
    C: CodeCache,
    H: host::InterpreterHost,
{
    pub(super) fn build_post_context(&self, order: &OrderV4) -> Vec<Vec<B256>> {
        let order_hash = state::order_hash(order);
        vec![vec![order_hash, order.owner.into_word()]]
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_quote_context(
        &self,
        order_hash: B256,
        owner: Address,
        counterparty: Address,
        input: &IOV2,
        input_decimals: u8,
        input_balance: Float,
        output: &IOV2,
        output_decimals: u8,
        output_balance: Float,
        signed_context: &[SignedContextV1],
    ) -> Vec<Vec<B256>> {
        let mut base_columns = vec![Vec::new(); CALLING_CONTEXT_COLUMNS];

        base_columns[CONTEXT_CALLING_CONTEXT_COLUMN - 1] =
            vec![order_hash, owner.into_word(), counterparty.into_word()];

        base_columns[CONTEXT_CALCULATIONS_COLUMN - 1] = vec![B256::ZERO; 2];

        base_columns[CONTEXT_VAULT_INPUTS_COLUMN - 1] = vec![
            input.token.into_word(),
            u8_to_b256(input_decimals),
            input.vaultId,
            input_balance.get_inner(),
            B256::ZERO,
        ];

        base_columns[CONTEXT_VAULT_OUTPUTS_COLUMN - 1] = vec![
            output.token.into_word(),
            u8_to_b256(output_decimals),
            output.vaultId,
            output_balance.get_inner(),
            B256::ZERO,
        ];

        self.build_context(base_columns, signed_context, counterparty)
    }

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
            let mut signers = Vec::with_capacity(signed_context.len());
            for sc in signed_context {
                signers.push(sc.signer.into_word());
            }
            context.push(signers);

            for sc in signed_context {
                context.push(sc.context.clone());
            }
        }

        context
    }
}

pub(super) fn namespace_for_order(order: &OrderV4, orderbook: Address) -> (U256, B256) {
    let state_namespace = address_to_u256(order.owner);
    let qualified = state::derive_fqn(state_namespace, orderbook);
    let namespace = U256::from_be_slice(qualified.as_slice());
    (namespace, qualified)
}

pub(super) fn u8_to_b256(value: u8) -> B256 {
    B256::from(U256::from(value))
}
