use super::decode::{DecodedEvent, DecodedEventData};
use alloy::primitives::Address;
use rain_orderbook_bindings::IOrderBookV5::OrderV4;
use std::collections::BTreeSet;

pub fn collect_token_addresses(
    decoded_events: &[DecodedEventData<DecodedEvent>],
) -> BTreeSet<Address> {
    let mut out = BTreeSet::new();

    for event in decoded_events {
        match &event.decoded_data {
            DecodedEvent::DepositV2(deposit) => {
                out.insert(deposit.token);
            }
            DecodedEvent::WithdrawV2(withdraw) => {
                out.insert(withdraw.token);
            }
            DecodedEvent::AddOrderV3(add) => {
                collect_order_tokens(&add.order, &mut out);
            }
            DecodedEvent::RemoveOrderV3(remove) => {
                collect_order_tokens(&remove.order, &mut out);
            }
            DecodedEvent::ClearV3(clear) => {
                collect_order_tokens(&clear.alice, &mut out);
                collect_order_tokens(&clear.bob, &mut out);
            }
            _ => {}
        }
    }

    out
}

fn collect_order_tokens(order: &OrderV4, out: &mut BTreeSet<Address>) {
    for input in &order.validInputs {
        out.insert(input.token);
    }
    for output in &order.validOutputs {
        out.insert(output.token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        AddOrderV3, DepositV2, OrderV4, RemoveOrderV3, WithdrawV2, IOV2,
    };

    fn build_event(event: DecodedEvent) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: super::super::decode::EventType::Unknown,
            block_number: "0x0".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0x0".into(),
            log_index: "0x0".into(),
            decoded_data: event,
        }
    }

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: Address::from([1u8; 20]),
            nonce: U256::from(1).into(),
            evaluable: rain_orderbook_bindings::IOrderBookV5::EvaluableV4 {
                interpreter: Address::from([2u8; 20]),
                store: Address::from([3u8; 20]),
                bytecode: alloy::primitives::Bytes::from(vec![]),
            },
            validInputs: vec![IOV2 {
                token: Address::from([4u8; 20]),
                vaultId: U256::from(10).into(),
            }],
            validOutputs: vec![IOV2 {
                token: Address::from([5u8; 20]),
                vaultId: U256::from(11).into(),
            }],
        }
    }

    #[test]
    fn collects_simple_transfers() {
        let deposit = DepositV2 {
            sender: Address::from([0u8; 20]),
            token: Address::from([1u8; 20]),
            vaultId: U256::from(1).into(),
            depositAmountUint256: U256::from(1),
        };
        let withdraw = WithdrawV2 {
            sender: Address::from([0u8; 20]),
            token: Address::from([2u8; 20]),
            vaultId: U256::from(1).into(),
            targetAmount: U256::from(0).into(),
            withdrawAmount: U256::from(0).into(),
            withdrawAmountUint256: U256::from(0),
        };

        let events = vec![
            build_event(DecodedEvent::DepositV2(Box::new(deposit))),
            build_event(DecodedEvent::WithdrawV2(Box::new(withdraw))),
        ];

        let tokens = collect_token_addresses(&events);
        assert!(tokens.contains(&Address::from([1u8; 20])));
        assert!(tokens.contains(&Address::from([2u8; 20])));
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn collects_orders() {
        let order = sample_order();
        let add_event = AddOrderV3 {
            sender: Address::from([6u8; 20]),
            orderHash: alloy::primitives::FixedBytes::from([0u8; 32]),
            order: order.clone(),
        };
        let remove_event = RemoveOrderV3 {
            sender: Address::from([7u8; 20]),
            orderHash: alloy::primitives::FixedBytes::from([1u8; 32]),
            order,
        };

        let events = vec![
            build_event(DecodedEvent::AddOrderV3(Box::new(add_event))),
            build_event(DecodedEvent::RemoveOrderV3(Box::new(remove_event))),
        ];

        let tokens = collect_token_addresses(&events);
        assert_eq!(tokens.len(), 2);
        assert!(tokens.contains(&Address::from([4u8; 20])));
        assert!(tokens.contains(&Address::from([5u8; 20])));
    }

    #[test]
    fn collects_from_clear() {
        let mut alice = sample_order();
        alice.validInputs.push(IOV2 {
            token: Address::from([8u8; 20]),
            vaultId: U256::from(0).into(),
        });
        let mut bob = sample_order();
        bob.validOutputs.push(IOV2 {
            token: Address::from([9u8; 20]),
            vaultId: U256::from(0).into(),
        });

        let clear = rain_orderbook_bindings::IOrderBookV5::ClearV3 {
            sender: Address::from([0u8; 20]),
            alice,
            bob,
            clearConfig: rain_orderbook_bindings::IOrderBookV5::ClearConfigV2 {
                aliceInputIOIndex: U256::from(0),
                aliceOutputIOIndex: U256::from(0),
                bobInputIOIndex: U256::from(0),
                bobOutputIOIndex: U256::from(0),
                aliceBountyVaultId: U256::from(0).into(),
                bobBountyVaultId: U256::from(0).into(),
            },
        };

        let events = vec![build_event(DecodedEvent::ClearV3(Box::new(clear)))];
        let tokens = collect_token_addresses(&events);
        assert!(tokens.contains(&Address::from([4u8; 20])));
        assert!(tokens.contains(&Address::from([5u8; 20])));
        assert!(tokens.contains(&Address::from([8u8; 20])));
        assert!(tokens.contains(&Address::from([9u8; 20])));
        assert_eq!(tokens.len(), 4);
    }
}
