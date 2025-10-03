use super::decode::{DecodedEvent, DecodedEventData};
use alloy::primitives::Address;
use std::collections::BTreeSet;

fn collect_tokens_from_order(
    order: &rain_orderbook_bindings::IOrderBookV5::OrderV4,
    set: &mut BTreeSet<Address>,
) {
    for io in order.validInputs.iter().chain(order.validOutputs.iter()) {
        set.insert(io.token);
    }
}

pub fn collect_token_addresses(events: &[DecodedEventData<DecodedEvent>]) -> BTreeSet<Address> {
    let mut out = BTreeSet::new();

    for event in events {
        match &event.decoded_data {
            DecodedEvent::DepositV2(decoded) => {
                out.insert(decoded.token);
            }
            DecodedEvent::WithdrawV2(decoded) => {
                out.insert(decoded.token);
            }
            DecodedEvent::AddOrderV3(decoded) => {
                collect_tokens_from_order(&decoded.order, &mut out);
            }
            DecodedEvent::RemoveOrderV3(decoded) => {
                collect_tokens_from_order(&decoded.order, &mut out);
            }
            DecodedEvent::ClearV3(decoded) => {
                collect_tokens_from_order(&decoded.alice, &mut out);
                collect_tokens_from_order(&decoded.bob, &mut out);
            }
            DecodedEvent::MetaV1_2(_)
            | DecodedEvent::AfterClearV2(_)
            | DecodedEvent::TakeOrderV3(_)
            | DecodedEvent::Unknown(_) => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, Bytes, FixedBytes, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearConfigV2, ClearStateChangeV2, ClearV3, DepositV2,
        EvaluableV4, OrderV4, RemoveOrderV3, SignedContextV1, TakeOrderConfigV4, TakeOrderV3,
        WithdrawV2, IOV2,
    };
    use rain_orderbook_bindings::OrderBook::MetaV1_2;

    fn zero_address() -> Address {
        Address::from([0u8; 20])
    }

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: zero_address(),
            nonce: U256::from(1).into(),
            evaluable: EvaluableV4 {
                interpreter: zero_address(),
                store: zero_address(),
                bytecode: Bytes::from(vec![0x01, 0x02]),
            },
            validInputs: vec![IOV2 {
                token: Address::from([4u8; 20]),
                vaultId: Default::default(),
            }],
            validOutputs: vec![IOV2 {
                token: Address::from([5u8; 20]),
                vaultId: Default::default(),
            }],
        }
    }

    fn wrap_event(event: DecodedEvent) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: super::super::decode::EventType::Unknown,
            block_number: String::new(),
            block_timestamp: String::new(),
            transaction_hash: String::new(),
            log_index: String::new(),
            decoded_data: event,
        }
    }

    #[test]
    fn collects_from_deposit_and_withdraw() {
        let deposit = DepositV2 {
            sender: zero_address(),
            token: Address::from([0x11u8; 20]),
            vaultId: Default::default(),
            depositAmountUint256: Default::default(),
        };
        let withdraw = WithdrawV2 {
            sender: zero_address(),
            token: Address::from([0x22u8; 20]),
            vaultId: Default::default(),
            targetAmount: Default::default(),
            withdrawAmount: Default::default(),
            withdrawAmountUint256: Default::default(),
        };

        let events = vec![
            wrap_event(DecodedEvent::DepositV2(Box::new(deposit))),
            wrap_event(DecodedEvent::WithdrawV2(Box::new(withdraw))),
        ];

        let addrs = collect_token_addresses(&events);
        assert!(addrs.contains(&Address::from([0x11u8; 20])));
        assert!(addrs.contains(&Address::from([0x22u8; 20])));
    }

    #[test]
    fn collects_from_orders_and_clear_events() {
        let order = sample_order();
        let add_event = AddOrderV3 {
            sender: zero_address(),
            orderHash: Default::default(),
            order: order.clone(),
        };
        let remove_event = RemoveOrderV3 {
            sender: zero_address(),
            orderHash: Default::default(),
            order: order.clone(),
        };
        let clear_event = ClearV3 {
            sender: zero_address(),
            alice: order.clone(),
            bob: order,
            clearConfig: ClearConfigV2 {
                aliceInputIOIndex: Default::default(),
                aliceOutputIOIndex: Default::default(),
                bobInputIOIndex: Default::default(),
                bobOutputIOIndex: Default::default(),
                aliceBountyVaultId: Default::default(),
                bobBountyVaultId: Default::default(),
            },
        };

        let events = vec![
            wrap_event(DecodedEvent::AddOrderV3(Box::new(add_event))),
            wrap_event(DecodedEvent::RemoveOrderV3(Box::new(remove_event))),
            wrap_event(DecodedEvent::ClearV3(Box::new(clear_event))),
        ];

        let addrs = collect_token_addresses(&events);
        assert!(addrs.contains(&Address::from([4u8; 20])));
        assert!(addrs.contains(&Address::from([5u8; 20])));
    }

    #[test]
    fn ignores_unrelated_events() {
        let events = vec![
            wrap_event(DecodedEvent::MetaV1_2(Box::new(MetaV1_2 {
                sender: zero_address(),
                subject: FixedBytes::<32>::ZERO,
                meta: Bytes::from(vec![0x01]),
            }))),
            wrap_event(DecodedEvent::AfterClearV2(Box::new(AfterClearV2 {
                sender: zero_address(),
                clearStateChange: ClearStateChangeV2 {
                    aliceInput: Default::default(),
                    aliceOutput: Default::default(),
                    bobInput: Default::default(),
                    bobOutput: Default::default(),
                },
            }))),
            wrap_event(DecodedEvent::TakeOrderV3(Box::new(TakeOrderV3 {
                sender: zero_address(),
                config: TakeOrderConfigV4 {
                    order: sample_order(),
                    inputIOIndex: Default::default(),
                    outputIOIndex: Default::default(),
                    signedContext: vec![SignedContextV1 {
                        signer: zero_address(),
                        context: vec![],
                        signature: Bytes::from(vec![0xAA]),
                    }],
                },
                input: Default::default(),
                output: Default::default(),
            }))),
        ];

        let addrs = collect_token_addresses(&events);
        assert!(addrs.is_empty());
    }
}
