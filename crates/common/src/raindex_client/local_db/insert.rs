use super::decode::{DecodedEvent, DecodedEventData};
use alloy::sol_types::SolValue;
use alloy::{
    hex,
    primitives::{keccak256, FixedBytes, U256},
};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{
    AddOrderV3, AfterClearV2, ClearV3, DepositV2, OrderV4, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    IOV2,
};
use rain_orderbook_bindings::OrderBook::MetaV1_2;
use std::convert::TryInto;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("Failed to parse hex string: {hex_str}")]
    HexParseError { hex_str: String },
    #[error("IO index {field} exceeds u64 range")]
    IoIndexOverflow { field: &'static str },
    #[error("{side} {io_type} IO index {index} out of bounds (len {len})")]
    IoIndexOutOfBounds {
        side: &'static str,
        io_type: &'static str,
        index: usize,
        len: usize,
    },
    #[error("Float conversion failed: {0}")]
    FloatConversion(String),
}

fn encode_u256_prefixed(value: &U256) -> String {
    hex::encode_prefixed(value.to_be_bytes::<32>())
}

fn compute_order_hash(order: &OrderV4) -> String {
    let encoded = order.abi_encode();
    let hash = keccak256(encoded);
    hex::encode_prefixed(hash)
}

fn u256_to_u64(value: &U256, field: &'static str) -> Result<u64, InsertError> {
    value
        .try_into()
        .map_err(|_| InsertError::IoIndexOverflow { field })
}

fn u64_to_usize(value: u64, field: &'static str) -> Result<usize, InsertError> {
    usize::try_from(value).map_err(|_| InsertError::IoIndexOverflow { field })
}

fn vault_id_by_index<'a>(
    ios: &'a [IOV2],
    index: usize,
    side: &'static str,
    io_type: &'static str,
) -> Result<&'a FixedBytes<32>, InsertError> {
    ios.get(index)
        .map(|io| &io.vaultId)
        .ok_or(InsertError::IoIndexOutOfBounds {
            side,
            io_type,
            index,
            len: ios.len(),
        })
}

struct EventContext<'a> {
    block_number: u64,
    block_timestamp: u64,
    transaction_hash: &'a str,
    log_index: u64,
}

fn event_context<'a>(
    event: &'a DecodedEventData<DecodedEvent>,
) -> Result<EventContext<'a>, InsertError> {
    Ok(EventContext {
        block_number: hex_to_decimal(&event.block_number)?,
        block_timestamp: hex_to_decimal(&event.block_timestamp)?,
        transaction_hash: &event.transaction_hash,
        log_index: hex_to_decimal(&event.log_index)?,
    })
}

pub fn decoded_events_to_sql(
    events: &[DecodedEventData<DecodedEvent>],
    end_block: u64,
) -> Result<String, InsertError> {
    let mut sql = String::new();
    sql.push_str("BEGIN TRANSACTION;\n\n");

    for event in events {
        match &event.decoded_data {
            DecodedEvent::DepositV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_deposit_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::WithdrawV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_withdraw_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::AddOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_add_order_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::RemoveOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_remove_order_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::TakeOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_take_order_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::ClearV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_clear_v3_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::AfterClearV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_after_clear_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::MetaV1_2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_meta_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::Unknown(decoded) => {
                eprintln!(
                    "Warning: Unknown event type for transaction {}: {}",
                    event.transaction_hash, decoded.note
                );
            }
        }
    }

    sql.push_str(&format!(
        "\nUPDATE sync_status SET last_synced_block = {}, updated_at = CURRENT_TIMESTAMP WHERE id = 1;\n",
        end_block
    ));

    sql.push_str("\nCOMMIT;\n");

    Ok(sql)
}

fn generate_deposit_sql(
    context: &EventContext<'_>,
    decoded: &DepositV2,
) -> Result<String, InsertError> {
    let deposit_amount_float = Float::from_fixed_decimal(decoded.depositAmountUint256, 6)
        .map_err(|err| InsertError::FloatConversion(err.to_string()))?;

    Ok(format!(
        "INSERT INTO deposits (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount, deposit_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.token),
        hex::encode_prefixed(decoded.vaultId),
        deposit_amount_float.as_hex(),
        encode_u256_prefixed(&decoded.depositAmountUint256)
    ))
}

fn generate_withdraw_sql(
    context: &EventContext<'_>,
    decoded: &WithdrawV2,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO withdrawals (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, withdraw_amount, withdraw_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.token),
        hex::encode_prefixed(decoded.vaultId),
        hex::encode_prefixed(decoded.targetAmount),
        hex::encode_prefixed(decoded.withdrawAmount),
        encode_u256_prefixed(&decoded.withdrawAmountUint256)
    ))
}

fn generate_add_order_sql(
    context: &EventContext<'_>,
    decoded: &AddOrderV3,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'AddOrderV3', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.orderHash),
        hex::encode_prefixed(decoded.order.owner),
        hex::encode_prefixed(decoded.order.nonce)
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_remove_order_sql(
    context: &EventContext<'_>,
    decoded: &RemoveOrderV3,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'RemoveOrderV3', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.orderHash),
        hex::encode_prefixed(decoded.order.owner),
        hex::encode_prefixed(decoded.order.nonce)
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_take_order_sql(
    context: &EventContext<'_>,
    decoded: &TakeOrderV3,
) -> Result<String, InsertError> {
    let input_io_index_u64 = u256_to_u64(&decoded.config.inputIOIndex, "inputIOIndex")?;
    let output_io_index_u64 = u256_to_u64(&decoded.config.outputIOIndex, "outputIOIndex")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO take_orders (block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, taker_input, taker_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.config.order.owner),
        hex::encode_prefixed(decoded.config.order.nonce),
        input_io_index_u64,
        output_io_index_u64,
        hex::encode_prefixed(decoded.input),
        hex::encode_prefixed(decoded.output)
    ));

    for (context_index, signed_context) in decoded.config.signedContext.iter().enumerate() {
        let context_value = format!(
            "signer:{},signature:{}",
            hex::encode_prefixed(signed_context.signer),
            hex::encode_prefixed(&signed_context.signature)
        );

        sql.push_str(&format!(
            "INSERT INTO take_order_contexts (transaction_hash, log_index, context_index, context_value) VALUES ('{}', {}, {}, '{}');\n",
            context.transaction_hash,
            context.log_index,
            context_index,
            context_value
        ));

        for (value_index, value) in signed_context.context.iter().enumerate() {
            sql.push_str(&format!(
                "INSERT INTO context_values (transaction_hash, log_index, context_index, value_index, value) VALUES ('{}', {}, {}, {}, '{}');\n",
                context.transaction_hash,
                context.log_index,
                context_index,
                value_index,
                hex::encode_prefixed(value)
            ));
        }
    }

    Ok(sql)
}

fn generate_clear_v3_sql(
    context: &EventContext<'_>,
    decoded: &ClearV3,
) -> Result<String, InsertError> {
    let alice_input_io_index_u64 =
        u256_to_u64(&decoded.clearConfig.aliceInputIOIndex, "aliceInputIOIndex")?;
    let alice_output_io_index_u64 = u256_to_u64(
        &decoded.clearConfig.aliceOutputIOIndex,
        "aliceOutputIOIndex",
    )?;
    let bob_input_io_index_u64 =
        u256_to_u64(&decoded.clearConfig.bobInputIOIndex, "bobInputIOIndex")?;
    let bob_output_io_index_u64 =
        u256_to_u64(&decoded.clearConfig.bobOutputIOIndex, "bobOutputIOIndex")?;

    let alice_input_index = u64_to_usize(alice_input_io_index_u64, "aliceInputIOIndex")?;
    let alice_output_index = u64_to_usize(alice_output_io_index_u64, "aliceOutputIOIndex")?;
    let bob_input_index = u64_to_usize(bob_input_io_index_u64, "bobInputIOIndex")?;
    let bob_output_index = u64_to_usize(bob_output_io_index_u64, "bobOutputIOIndex")?;

    let alice_input_vault_id = vault_id_by_index(
        &decoded.alice.validInputs,
        alice_input_index,
        "alice",
        "input",
    )?;
    let alice_output_vault_id = vault_id_by_index(
        &decoded.alice.validOutputs,
        alice_output_index,
        "alice",
        "output",
    )?;
    let bob_input_vault_id =
        vault_id_by_index(&decoded.bob.validInputs, bob_input_index, "bob", "input")?;
    let bob_output_vault_id =
        vault_id_by_index(&decoded.bob.validOutputs, bob_output_index, "bob", "output")?;

    let alice_order_hash = compute_order_hash(&decoded.alice);
    let bob_order_hash = compute_order_hash(&decoded.bob);

    Ok(format!(
        "INSERT INTO clear_v3_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_order_hash, alice_order_owner, alice_input_io_index, alice_output_io_index, alice_bounty_vault_id, alice_input_vault_id, alice_output_vault_id, bob_order_hash, bob_order_owner, bob_input_io_index, bob_output_io_index, bob_bounty_vault_id, bob_input_vault_id, bob_output_vault_id) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        alice_order_hash,
        hex::encode_prefixed(decoded.alice.owner),
        alice_input_io_index_u64,
        alice_output_io_index_u64,
        hex::encode_prefixed(decoded.clearConfig.aliceBountyVaultId),
        hex::encode_prefixed(alice_input_vault_id),
        hex::encode_prefixed(alice_output_vault_id),
        bob_order_hash,
        hex::encode_prefixed(decoded.bob.owner),
        bob_input_io_index_u64,
        bob_output_io_index_u64,
        hex::encode_prefixed(decoded.clearConfig.bobBountyVaultId),
        hex::encode_prefixed(bob_input_vault_id),
        hex::encode_prefixed(bob_output_vault_id)
    ))
}

fn generate_after_clear_sql(
    context: &EventContext<'_>,
    decoded: &AfterClearV2,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO after_clear_v2_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.clearStateChange.aliceInput),
        hex::encode_prefixed(decoded.clearStateChange.aliceOutput),
        hex::encode_prefixed(decoded.clearStateChange.bobInput),
        hex::encode_prefixed(decoded.clearStateChange.bobOutput)
    ))
}

fn generate_meta_sql(
    context: &EventContext<'_>,
    decoded: &MetaV1_2,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO meta_events (block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        hex::encode_prefixed(decoded.sender),
        hex::encode_prefixed(decoded.subject),
        hex::encode_prefixed(&decoded.meta)
    ))
}

fn generate_order_ios_sql(context: &EventContext<'_>, order: &OrderV4) -> String {
    let mut rows = Vec::new();

    for (index, input) in order.validInputs.iter().enumerate() {
        rows.push(format!(
            "('{}', {}, {}, 'input', '{}', '{}')",
            context.transaction_hash,
            context.log_index,
            index,
            hex::encode_prefixed(input.token),
            hex::encode_prefixed(input.vaultId),
        ));
    }

    for (index, output) in order.validOutputs.iter().enumerate() {
        rows.push(format!(
            "('{}', {}, {}, 'output', '{}', '{}')",
            context.transaction_hash,
            context.log_index,
            index,
            hex::encode_prefixed(output.token),
            hex::encode_prefixed(output.vaultId),
        ));
    }

    if rows.is_empty() {
        return String::new();
    }

    format!(
        "INSERT INTO order_ios (transaction_hash, log_index, io_index, io_type, token, vault_id) VALUES {};\n",
        rows.join(", ")
    )
}

fn hex_to_decimal(hex_str: &str) -> Result<u64, InsertError> {
    let hex_str_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(hex_str_clean, 16).map_err(|_| InsertError::HexParseError {
        hex_str: hex_str.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::local_db::decode::{EventType, UnknownEventDecoded};
    use alloy::primitives::{Address, Bytes, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        ClearConfigV2, EvaluableV4, SignedContextV1, TakeOrderConfigV4,
    };

    fn build_event(
        event_type: EventType,
        block_number: &str,
        block_timestamp: &str,
        transaction_hash: &str,
        log_index: &str,
        decoded: DecodedEvent,
    ) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type,
            block_number: block_number.to_string(),
            block_timestamp: block_timestamp.to_string(),
            transaction_hash: transaction_hash.to_string(),
            log_index: log_index.to_string(),
            decoded_data: decoded,
        }
    }

    fn sample_order() -> OrderV4 {
        OrderV4 {
            owner: Address::from([1u8; 20]),
            nonce: U256::from(1).into(),
            evaluable: EvaluableV4 {
                interpreter: Address::from([2u8; 20]),
                store: Address::from([3u8; 20]),
                bytecode: Bytes::from(vec![0x01, 0x02, 0x03, 0x04]),
            },
            validInputs: vec![
                IOV2 {
                    token: Address::from([4u8; 20]),
                    vaultId: U256::from(0x64).into(),
                },
                IOV2 {
                    token: Address::from([5u8; 20]),
                    vaultId: U256::from(0xc8).into(),
                },
            ],
            validOutputs: vec![IOV2 {
                token: Address::from([6u8; 20]),
                vaultId: U256::from(0x12c).into(),
            }],
        }
    }

    fn sample_clear_event() -> DecodedEventData<DecodedEvent> {
        let clear = ClearV3 {
            sender: Address::from([0x11; 20]),
            alice: sample_order(),
            bob: OrderV4 {
                owner: Address::from([0x12; 20]),
                nonce: U256::from(2).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([0x13; 20]),
                    store: Address::from([0x14; 20]),
                    bytecode: Bytes::from(vec![0x05, 0x06, 0x07, 0x08]),
                },
                validInputs: vec![IOV2 {
                    token: Address::from([0x15; 20]),
                    vaultId: U256::from(0x2bc).into(),
                }],
                validOutputs: vec![IOV2 {
                    token: Address::from([0x16; 20]),
                    vaultId: U256::from(0x320).into(),
                }],
            },
            clearConfig: ClearConfigV2 {
                aliceInputIOIndex: U256::from(0),
                aliceOutputIOIndex: U256::from(0),
                bobInputIOIndex: U256::from(0),
                bobOutputIOIndex: U256::from(0),
                aliceBountyVaultId: U256::from(0).into(),
                bobBountyVaultId: U256::from(0).into(),
            },
        };

        build_event(
            EventType::ClearV3,
            "0x100",
            "0x200",
            "0xabc",
            "0x1",
            DecodedEvent::ClearV3(Box::new(clear)),
        )
    }

    fn sample_take_event() -> DecodedEventData<DecodedEvent> {
        let take = TakeOrderV3 {
            sender: Address::from([0x09; 20]),
            config: TakeOrderConfigV4 {
                order: sample_order(),
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![SignedContextV1 {
                    signer: Address::from([0x0a; 20]),
                    context: vec![U256::from(0x2a).into()],
                    signature: Bytes::from(vec![0x11, 0x22, 0x33]),
                }],
            },
            input: U256::from(0x3e8).into(),
            output: U256::from(0x7d0).into(),
        };

        build_event(
            EventType::TakeOrderV3,
            "0x101",
            "0x201",
            "0xdef",
            "0x2",
            DecodedEvent::TakeOrderV3(Box::new(take)),
        )
    }

    fn sample_deposit_event() -> DecodedEventData<DecodedEvent> {
        let deposit = DepositV2 {
            sender: Address::from([0x0d; 20]),
            token: Address::from([0x0e; 20]),
            vaultId: U256::from(0x258).into(),
            depositAmountUint256: U256::from(0xfa0),
        };

        build_event(
            EventType::DepositV2,
            "0x102",
            "0x202",
            "0x123",
            "0x3",
            DecodedEvent::DepositV2(Box::new(deposit)),
        )
    }

    #[test]
    fn deposit_sql_generation() {
        let event = sample_deposit_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::DepositV2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_deposit_sql(&context, decoded).unwrap();
        assert!(sql.contains("INSERT INTO deposits"));
        assert!(sql.contains("0x0000000000000000000000000000000000000000000000000000000000000fa0"));
    }

    #[test]
    fn take_order_sql_generation() {
        let event = sample_take_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::TakeOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_take_order_sql(&context, decoded).unwrap();
        assert!(sql.contains("INSERT INTO take_orders"));
        assert!(sql.contains("signer:0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"));
        assert!(
            sql.contains("'0x000000000000000000000000000000000000000000000000000000000000002a'")
        );
    }

    #[test]
    fn clear_v3_sql_generation() {
        let event = sample_clear_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::ClearV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_clear_v3_sql(&context, decoded).unwrap();
        assert!(sql.contains("INSERT INTO clear_v3_events"));
        assert!(sql.contains("0x0000000000000000000000000000000000000000000000000000000000000064"));
        assert!(sql.contains("0x00000000000000000000000000000000000000000000000000000000000002bc"));
    }

    #[test]
    fn hex_to_decimal_roundtrip() {
        assert_eq!(hex_to_decimal("0x10").unwrap(), 16);
        assert!(matches!(
            hex_to_decimal("0xgg"),
            Err(InsertError::HexParseError { .. })
        ));
    }

    #[test]
    fn decoded_events_to_sql_multiple_events() {
        let clear_event = sample_clear_event();
        let deposit_event = sample_deposit_event();
        let sql = decoded_events_to_sql(&[deposit_event, clear_event], 0x200).unwrap();
        assert!(sql.contains("INSERT INTO deposits"));
        assert!(sql.contains("INSERT INTO clear_v3_events"));
        assert!(sql.contains("UPDATE sync_status SET last_synced_block = 512"));
    }

    #[test]
    fn unknown_event_is_logged() {
        let unknown_event = build_event(
            EventType::Unknown,
            "0x0",
            "0x0",
            "0xbeef",
            "0x0",
            DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0xdead".into(),
                note: "n/a".into(),
            }),
        );
        let sql = decoded_events_to_sql(&[unknown_event], 0).unwrap();
        assert!(sql.contains("BEGIN TRANSACTION"));
    }
}
