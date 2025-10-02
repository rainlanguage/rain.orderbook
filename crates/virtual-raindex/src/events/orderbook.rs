//! Helpers for converting OrderBook contract events into [`RaindexMutation`]s.

use std::convert::TryInto;

use alloy::primitives::{B256, U256};
use rain_orderbook_bindings::IOrderBookV5::{
    AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
};

use crate::{
    error::{RaindexError, Result},
    state::{RaindexMutation, VaultDelta},
    Float,
};

fn u256_to_usize(value: &U256) -> Result<usize> {
    value
        .try_into()
        .map_err(|_| RaindexError::Unimplemented("index too large for usize"))
}

fn push_delta(
    deltas: &mut Vec<VaultDelta>,
    owner: alloy::primitives::Address,
    token: alloy::primitives::Address,
    vault_id: B256,
    delta: Float,
) {
    // Skip strict zeroes to avoid redundant mutations.
    if matches!(delta.is_zero(), Ok(true)) {
        return;
    }

    deltas.push(VaultDelta {
        owner,
        token,
        vault_id,
        delta,
    });
}

fn take_order_to_mutations(event: &TakeOrderV3) -> Result<Vec<RaindexMutation>> {
    let input_index = u256_to_usize(&event.config.inputIOIndex)?;
    let output_index = u256_to_usize(&event.config.outputIOIndex)?;
    let order = &event.config.order;

    let taker_input = Float::from_raw(event.input);
    let taker_output = Float::from_raw(event.output);

    if input_index >= order.validInputs.len() || output_index >= order.validOutputs.len() {
        return Err(RaindexError::Unimplemented(
            "take order IO index out of bounds",
        ));
    }

    let input_io = &order.validInputs[input_index];
    let output_io = &order.validOutputs[output_index];

    let mut deltas = Vec::with_capacity(2);
    push_delta(
        &mut deltas,
        order.owner,
        input_io.token,
        input_io.vaultId,
        taker_output,
    );
    push_delta(
        &mut deltas,
        order.owner,
        output_io.token,
        output_io.vaultId,
        (-taker_input)?,
    );

    Ok(vec![RaindexMutation::VaultDeltas { deltas }])
}

fn clear_events_to_mutations(
    clear: &ClearV3,
    state_change: &AfterClearV2,
) -> Result<Vec<RaindexMutation>> {
    let clear_config = &clear.clearConfig;
    let state = &state_change.clearStateChange;

    let alice_input_index = u256_to_usize(&clear_config.aliceInputIOIndex)?;
    let alice_output_index = u256_to_usize(&clear_config.aliceOutputIOIndex)?;
    let bob_input_index = u256_to_usize(&clear_config.bobInputIOIndex)?;
    let bob_output_index = u256_to_usize(&clear_config.bobOutputIOIndex)?;

    let alice = &clear.alice;
    let bob = &clear.bob;

    if alice_input_index >= alice.validInputs.len()
        || alice_output_index >= alice.validOutputs.len()
        || bob_input_index >= bob.validInputs.len()
        || bob_output_index >= bob.validOutputs.len()
    {
        return Err(RaindexError::Unimplemented("clear IO index out of bounds"));
    }

    let alice_input = Float::from_raw(state.aliceInput);
    let alice_output = Float::from_raw(state.aliceOutput);
    let bob_input = Float::from_raw(state.bobInput);
    let bob_output = Float::from_raw(state.bobOutput);

    let alice_input_io = &alice.validInputs[alice_input_index];
    let alice_output_io = &alice.validOutputs[alice_output_index];
    let bob_input_io = &bob.validInputs[bob_input_index];
    let bob_output_io = &bob.validOutputs[bob_output_index];

    let mut deltas = Vec::with_capacity(6);

    push_delta(
        &mut deltas,
        alice.owner,
        alice_input_io.token,
        alice_input_io.vaultId,
        alice_input,
    );
    push_delta(
        &mut deltas,
        alice.owner,
        alice_output_io.token,
        alice_output_io.vaultId,
        (-alice_output)?,
    );

    push_delta(
        &mut deltas,
        bob.owner,
        bob_input_io.token,
        bob_input_io.vaultId,
        bob_input,
    );
    push_delta(
        &mut deltas,
        bob.owner,
        bob_output_io.token,
        bob_output_io.vaultId,
        (-bob_output)?,
    );

    let alice_bounty = (alice_output - bob_input)?;
    let bob_bounty = (bob_output - alice_input)?;

    push_delta(
        &mut deltas,
        clear.sender,
        alice_output_io.token,
        clear_config.aliceBountyVaultId,
        alice_bounty,
    );
    push_delta(
        &mut deltas,
        clear.sender,
        bob_output_io.token,
        clear_config.bobBountyVaultId,
        bob_bounty,
    );

    Ok(vec![RaindexMutation::VaultDeltas { deltas }])
}

/// Wrapper around supported OrderBook events for mutation conversion.
#[derive(Clone, Copy, Debug)]
pub enum OrderBookEvent<'a> {
    /// An order was added to the onchain orderbook.
    AddOrder(&'a AddOrderV3),
    /// An order was removed from the onchain orderbook.
    RemoveOrder(&'a RemoveOrderV3),
    /// Tokens were deposited into a vault.
    Deposit {
        event: &'a DepositV2,
        /// Token decimals required to reconstruct the original float amount.
        decimals: Option<u8>,
    },
    /// Tokens were withdrawn from a vault.
    Withdraw(&'a WithdrawV2),
    /// An order was taken against the orderbook.
    TakeOrder(&'a TakeOrderV3),
    /// Two orders were cleared against each other.
    Clear {
        clear: &'a ClearV3,
        state_change: &'a AfterClearV2,
    },
}

/// Converts a single [`OrderBookEvent`] into one or more [`RaindexMutation`]s.
///
/// The deposit variant requires token decimals to rebuild the float value that
/// the virtual engine tracks. If decimals are missing the function returns
/// [`RaindexError::TokenDecimalMissing`].
pub fn orderbook_event_to_mutations(event: OrderBookEvent<'_>) -> Result<Vec<RaindexMutation>> {
    match event {
        OrderBookEvent::AddOrder(event) => Ok(vec![RaindexMutation::SetOrders {
            orders: vec![event.order.clone()],
        }]),
        OrderBookEvent::RemoveOrder(event) => Ok(vec![RaindexMutation::RemoveOrders {
            order_hashes: vec![event.orderHash],
        }]),
        OrderBookEvent::Deposit { event, decimals } => {
            let decimals =
                decimals.ok_or(RaindexError::TokenDecimalMissing { token: event.token })?;

            let delta = Float::from_fixed_decimal(event.depositAmountUint256, decimals)?;
            let mutation = RaindexMutation::VaultDeltas {
                deltas: vec![VaultDelta {
                    owner: event.sender,
                    token: event.token,
                    vault_id: event.vaultId,
                    delta,
                }],
            };
            Ok(vec![mutation])
        }
        OrderBookEvent::Withdraw(event) => {
            let amount = Float::from_raw(event.withdrawAmount);
            let delta = (-amount)?;
            let mutation = RaindexMutation::VaultDeltas {
                deltas: vec![VaultDelta {
                    owner: event.sender,
                    token: event.token,
                    vault_id: event.vaultId,
                    delta,
                }],
            };
            Ok(vec![mutation])
        }
        OrderBookEvent::TakeOrder(event) => take_order_to_mutations(event),
        OrderBookEvent::Clear {
            clear,
            state_change,
        } => clear_events_to_mutations(clear, state_change),
    }
}

/// Converts an iterator of [`OrderBookEvent`]s into a flat list of mutations.
pub fn orderbook_events_to_mutations<'a>(
    events: impl IntoIterator<Item = OrderBookEvent<'a>>,
) -> Result<Vec<RaindexMutation>> {
    let mut mutations = Vec::new();
    for event in events {
        mutations.extend(orderbook_event_to_mutations(event)?);
    }
    Ok(mutations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, Bytes, B256, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        ClearConfigV2, ClearStateChangeV2, EvaluableV4, OrderV4, TakeOrderConfigV4, IOV2,
    };

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
            nonce: B256::from([9u8; 32]),
        }
    }

    #[test]
    fn add_order_event_converts_to_set_orders() {
        let order = sample_order();
        let event = AddOrderV3 {
            sender: Address::repeat_byte(0x11),
            orderHash: B256::from([0xAB; 32]),
            order: order.clone(),
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::AddOrder(&event))
            .expect("conversion should succeed");
        assert_eq!(mutations.len(), 1);

        match &mutations[0] {
            RaindexMutation::SetOrders { orders } => {
                assert_eq!(orders, &vec![order]);
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn remove_order_event_converts_to_remove_orders() {
        let event = RemoveOrderV3 {
            sender: Address::repeat_byte(0x11),
            orderHash: B256::from([0xCD; 32]),
            order: sample_order(),
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::RemoveOrder(&event))
            .expect("conversion should succeed");

        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            RaindexMutation::RemoveOrders { order_hashes } => {
                assert_eq!(order_hashes, &vec![event.orderHash]);
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn deposit_event_requires_decimals() {
        let event = DepositV2 {
            sender: Address::repeat_byte(0x01),
            token: Address::repeat_byte(0x02),
            vaultId: B256::from([3u8; 32]),
            depositAmountUint256: U256::from(10u64),
        };

        let err = orderbook_event_to_mutations(OrderBookEvent::Deposit {
            event: &event,
            decimals: None,
        })
        .expect_err("decimals missing should error");

        match err {
            RaindexError::TokenDecimalMissing { token } => {
                assert_eq!(token, event.token);
            }
            other => panic!("unexpected error {other:?}"),
        }
    }

    #[test]
    fn deposit_event_converts_to_vault_delta() {
        let event = DepositV2 {
            sender: Address::repeat_byte(0x01),
            token: Address::repeat_byte(0x02),
            vaultId: B256::from([3u8; 32]),
            depositAmountUint256: U256::from(1_500_000_000_000_000_000u128),
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::Deposit {
            event: &event,
            decimals: Some(18),
        })
        .expect("conversion should succeed");

        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            RaindexMutation::VaultDeltas { deltas } => {
                assert_eq!(deltas.len(), 1);
                let delta = &deltas[0];
                assert_eq!(delta.owner, event.sender);
                assert_eq!(delta.token, event.token);
                assert_eq!(delta.vault_id, event.vaultId);
                assert_eq!(delta.delta.format().unwrap(), "1.5");
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn withdraw_event_produces_negative_delta() {
        let amount = Float::parse("2.5".to_string()).expect("float parse");
        let event = WithdrawV2 {
            sender: Address::repeat_byte(0x04),
            token: Address::repeat_byte(0x05),
            vaultId: B256::from([6u8; 32]),
            targetAmount: B256::from([0u8; 32]),
            withdrawAmount: amount.get_inner(),
            withdrawAmountUint256: U256::ZERO,
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::Withdraw(&event))
            .expect("conversion should succeed");

        match &mutations[0] {
            RaindexMutation::VaultDeltas { deltas } => {
                assert_eq!(deltas.len(), 1);
                let delta = &deltas[0];
                assert_eq!(delta.owner, event.sender);
                assert_eq!(delta.token, event.token);
                assert_eq!(delta.vault_id, event.vaultId);
                let formatted = delta.delta.format().unwrap();
                assert_eq!(formatted, "-2.5");
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn take_order_event_updates_owner_vaults() {
        let order = sample_order();
        let taker_input = Float::parse("1.25".to_string()).expect("float parse");
        let taker_output = Float::parse("2".to_string()).expect("float parse");

        let event = TakeOrderV3 {
            sender: Address::repeat_byte(0xAA),
            config: TakeOrderConfigV4 {
                order: order.clone(),
                inputIOIndex: U256::ZERO,
                outputIOIndex: U256::ZERO,
                signedContext: Vec::new(),
            },
            input: taker_input.get_inner(),
            output: taker_output.get_inner(),
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::TakeOrder(&event))
            .expect("conversion should succeed");

        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            RaindexMutation::VaultDeltas { deltas } => {
                assert_eq!(deltas.len(), 2);
                let input_delta = &deltas[0];
                assert_eq!(input_delta.owner, order.owner);
                assert_eq!(input_delta.token, order.validInputs[0].token);
                assert_eq!(input_delta.vault_id, order.validInputs[0].vaultId);
                assert_eq!(input_delta.delta.format().unwrap(), "2");

                let output_delta = &deltas[1];
                assert_eq!(output_delta.owner, order.owner);
                assert_eq!(output_delta.token, order.validOutputs[0].token);
                assert_eq!(output_delta.vault_id, order.validOutputs[0].vaultId);
                assert_eq!(output_delta.delta.format().unwrap(), "-1.25");
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }

    #[test]
    fn clear_event_updates_both_orders_and_bounties() {
        let mut alice_order = sample_order();
        alice_order.owner = Address::repeat_byte(0x10);
        let mut bob_order = sample_order();
        bob_order.owner = Address::repeat_byte(0x20);
        bob_order.validInputs[0].token = Address::repeat_byte(0x33);
        bob_order.validOutputs[0].token = Address::repeat_byte(0x44);

        let clear_event = ClearV3 {
            sender: Address::repeat_byte(0x99),
            alice: alice_order.clone(),
            bob: bob_order.clone(),
            clearConfig: ClearConfigV2 {
                aliceInputIOIndex: U256::ZERO,
                aliceOutputIOIndex: U256::ZERO,
                bobInputIOIndex: U256::ZERO,
                bobOutputIOIndex: U256::ZERO,
                aliceBountyVaultId: B256::from([0xAA; 32]),
                bobBountyVaultId: B256::from([0xBB; 32]),
            },
        };

        let state_change = AfterClearV2 {
            sender: Address::repeat_byte(0x99),
            clearStateChange: ClearStateChangeV2 {
                aliceOutput: Float::parse("4".to_string()).unwrap().get_inner(),
                bobOutput: Float::parse("3".to_string()).unwrap().get_inner(),
                aliceInput: Float::parse("1.5".to_string()).unwrap().get_inner(),
                bobInput: Float::parse("2".to_string()).unwrap().get_inner(),
            },
        };

        let mutations = orderbook_event_to_mutations(OrderBookEvent::Clear {
            clear: &clear_event,
            state_change: &state_change,
        })
        .expect("conversion should succeed");

        assert_eq!(mutations.len(), 1);
        match &mutations[0] {
            RaindexMutation::VaultDeltas { deltas } => {
                // alice input +1.5, alice output -4, bob input +2, bob output -3, clearer bounties +2, +1
                assert_eq!(deltas.len(), 6);
                let find_delta = |owner: Address, token: Address, vault: B256| {
                    deltas
                        .iter()
                        .find(|delta| {
                            delta.owner == owner && delta.token == token && delta.vault_id == vault
                        })
                        .map(|delta| delta.delta.format().unwrap())
                        .expect("delta present")
                };

                assert_eq!(
                    find_delta(
                        alice_order.owner,
                        alice_order.validInputs[0].token,
                        alice_order.validInputs[0].vaultId
                    ),
                    "1.5"
                );
                assert_eq!(
                    find_delta(
                        alice_order.owner,
                        alice_order.validOutputs[0].token,
                        alice_order.validOutputs[0].vaultId
                    ),
                    "-4"
                );
                assert_eq!(
                    find_delta(
                        bob_order.owner,
                        bob_order.validInputs[0].token,
                        bob_order.validInputs[0].vaultId
                    ),
                    "2"
                );
                assert_eq!(
                    find_delta(
                        bob_order.owner,
                        bob_order.validOutputs[0].token,
                        bob_order.validOutputs[0].vaultId
                    ),
                    "-3"
                );
                assert_eq!(
                    find_delta(
                        clear_event.sender,
                        alice_order.validOutputs[0].token,
                        clear_event.clearConfig.aliceBountyVaultId
                    ),
                    "2"
                );
                assert_eq!(
                    find_delta(
                        clear_event.sender,
                        bob_order.validOutputs[0].token,
                        clear_event.clearConfig.bobBountyVaultId
                    ),
                    "1.5"
                );
            }
            other => panic!("unexpected mutation: {other:?}"),
        }
    }
}
