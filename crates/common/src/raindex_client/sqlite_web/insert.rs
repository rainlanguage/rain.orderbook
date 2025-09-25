use super::decode::{
    AddOrderV3Decoded, AfterClearV2Decoded, ClearV3Decoded, DecodedEvent, DecodedEventData,
    DepositV2Decoded, EventType, MetaV1_2Decoded, OrderDecoded, RemoveOrderV3Decoded,
    TakeOrderV3Decoded, UnknownEventDecoded, WithdrawV2Decoded,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("Failed to parse hex string: {hex_str}")]
    HexParseError { hex_str: String },
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
                sql.push_str(&generate_deposit_sql(&context, decoded)?);
            }
            DecodedEvent::WithdrawV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_withdraw_sql(&context, decoded)?);
            }
            DecodedEvent::AddOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_add_order_sql(&context, decoded)?);
            }
            DecodedEvent::RemoveOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_remove_order_sql(&context, decoded)?);
            }
            DecodedEvent::TakeOrderV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_take_order_sql(&context, decoded)?);
            }
            DecodedEvent::ClearV3(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_clear_v3_sql(&context, decoded)?);
            }
            DecodedEvent::AfterClearV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_after_clear_sql(&context, decoded)?);
            }
            DecodedEvent::MetaV1_2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_meta_sql(&context, decoded)?);
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
    decoded: &DepositV2Decoded,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO deposits (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.token,
        decoded.vault_id,
        decoded.deposit_amount_uint256
    ))
}

fn generate_withdraw_sql(
    context: &EventContext<'_>,
    decoded: &WithdrawV2Decoded,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO withdrawals (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, withdraw_amount, withdraw_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.token,
        decoded.vault_id,
        decoded.target_amount,
        decoded.withdraw_amount,
        decoded.withdraw_amount_uint256
    ))
}

fn generate_add_order_sql(
    context: &EventContext<'_>,
    decoded: &AddOrderV3Decoded,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'AddOrderV3', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.order_hash,
        decoded.order.owner,
        decoded.order.nonce
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_remove_order_sql(
    context: &EventContext<'_>,
    decoded: &RemoveOrderV3Decoded,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'RemoveOrderV3', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.order_hash,
        decoded.order.owner,
        decoded.order.nonce
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_take_order_sql(
    context: &EventContext<'_>,
    decoded: &TakeOrderV3Decoded,
) -> Result<String, InsertError> {
    let input_io_index = hex_to_decimal(&decoded.config.input_io_index)?;
    let output_io_index = hex_to_decimal(&decoded.config.output_io_index)?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO take_orders (block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, input, output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.config.order.owner,
        decoded.config.order.nonce,
        input_io_index,
        output_io_index,
        decoded.input,
        decoded.output
    ));

    for (context_index, signed_context) in decoded.config.signed_context.iter().enumerate() {
        let context_value = format!(
            "signer:{},signature:{}",
            signed_context.signer, signed_context.signature
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
                value
            ));
        }
    }

    Ok(sql)
}

fn generate_clear_v3_sql(
    context: &EventContext<'_>,
    decoded: &ClearV3Decoded,
) -> Result<String, InsertError> {
    let alice_input_io_index = hex_to_decimal(&decoded.alice_input_io_index)?;
    let alice_output_io_index = hex_to_decimal(&decoded.alice_output_io_index)?;
    let bob_input_io_index = hex_to_decimal(&decoded.bob_input_io_index)?;
    let bob_output_io_index = hex_to_decimal(&decoded.bob_output_io_index)?;

    Ok(format!(
        "INSERT INTO clear_v3_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_order_hash, alice_order_owner, alice_input_io_index, alice_output_io_index, alice_bounty_vault_id, alice_input_vault_id, alice_output_vault_id, bob_order_hash, bob_order_owner, bob_input_io_index, bob_output_io_index, bob_bounty_vault_id, bob_input_vault_id, bob_output_vault_id) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.alice_order_hash,
        decoded.alice_owner,
        alice_input_io_index,
        alice_output_io_index,
        decoded.alice_bounty_vault_id,
        decoded.alice_input_vault_id,
        decoded.alice_output_vault_id,
        decoded.bob_order_hash,
        decoded.bob_owner,
        bob_input_io_index,
        bob_output_io_index,
        decoded.bob_bounty_vault_id,
        decoded.bob_input_vault_id,
        decoded.bob_output_vault_id
    ))
}

fn generate_after_clear_sql(
    context: &EventContext<'_>,
    decoded: &AfterClearV2Decoded,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO after_clear_v2_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.alice_input,
        decoded.alice_output,
        decoded.bob_input,
        decoded.bob_output
    ))
}

fn generate_meta_sql(
    context: &EventContext<'_>,
    decoded: &MetaV1_2Decoded,
) -> Result<String, InsertError> {
    Ok(format!(
        "INSERT INTO meta_events (block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}');\n",
        context.block_number,
        context.block_timestamp,
        context.transaction_hash,
        context.log_index,
        decoded.sender,
        decoded.subject,
        decoded.meta
    ))
}

fn generate_order_ios_sql(context: &EventContext<'_>, order: &OrderDecoded) -> String {
    let mut rows = Vec::new();

    for (index, input) in order.valid_inputs.iter().enumerate() {
        rows.push(format!(
            "('{}', {}, {}, 'input', '{}', '{}')",
            context.transaction_hash, context.log_index, index, input.token, input.vault_id
        ));
    }

    for (index, output) in order.valid_outputs.iter().enumerate() {
        rows.push(format!(
            "('{}', {}, {}, 'output', '{}', '{}')",
            context.transaction_hash, context.log_index, index, output.token, output.vault_id
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
    use super::super::decode::{
        OrderEvaluableDecoded, OrderIoDecoded, SignedContextDecoded, TakeOrderConfigDecoded,
    };
    use super::*;

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

    fn sample_order() -> OrderDecoded {
        OrderDecoded {
            owner: "0x0101010101010101010101010101010101010101".to_string(),
            nonce: "0x1".to_string(),
            evaluable: OrderEvaluableDecoded {
                interpreter: "0x0202020202020202020202020202020202020202".to_string(),
                store: "0x0303030303030303030303030303030303030303".to_string(),
                bytecode: "0x01020304".to_string(),
            },
            valid_inputs: vec![
                OrderIoDecoded {
                    token: "0x0404040404040404040404040404040404040404".to_string(),
                    vault_id: "0x64".to_string(),
                },
                OrderIoDecoded {
                    token: "0x0505050505050505050505050505050505050505".to_string(),
                    vault_id: "0xc8".to_string(),
                },
            ],
            valid_outputs: vec![OrderIoDecoded {
                token: "0x0606060606060606060606060606060606060606".to_string(),
                vault_id: "0x12c".to_string(),
            }],
        }
    }

    fn sample_deposit_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::DepositV2,
            "0x123456",
            "0x64b8c123",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x1",
            DecodedEvent::DepositV2(DepositV2Decoded {
                sender: "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d".to_string(),
                token: "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e".to_string(),
                vault_id: "0x258".to_string(),
                deposit_amount_uint256: "0xfa0".to_string(),
            }),
        )
    }

    fn sample_withdraw_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::WithdrawV2,
            "0x123457",
            "0x64b8c124",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891",
            "0x2",
            DecodedEvent::WithdrawV2(WithdrawV2Decoded {
                sender: "0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b".to_string(),
                token: "0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c".to_string(),
                vault_id: "0x1f4".to_string(),
                target_amount: "0xbb8".to_string(),
                withdraw_amount: "0x9c4".to_string(),
                withdraw_amount_uint256: "0x9c4".to_string(),
            }),
        )
    }

    fn sample_add_order_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::AddOrderV3,
            "0x123458",
            "0x64b8c125",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892",
            "0x3",
            DecodedEvent::AddOrderV3(AddOrderV3Decoded {
                sender: "0x0707070707070707070707070707070707070707".to_string(),
                order_hash: "0x0808080808080808080808080808080808080808080808080808080808080808"
                    .to_string(),
                order: sample_order(),
            }),
        )
    }

    fn sample_remove_order_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::RemoveOrderV3,
            "0x123459",
            "0x64b8c125",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567897",
            "0x4",
            DecodedEvent::RemoveOrderV3(RemoveOrderV3Decoded {
                sender: "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a".to_string(),
                order_hash: "0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"
                    .to_string(),
                order: OrderDecoded {
                    owner: "0x0101010101010101010101010101010101010101".to_string(),
                    nonce: "0x2".to_string(),
                    evaluable: OrderEvaluableDecoded {
                        interpreter: "0x0202020202020202020202020202020202020202".to_string(),
                        store: "0x0303030303030303030303030303030303030303".to_string(),
                        bytecode: "0x05060708".to_string(),
                    },
                    valid_inputs: vec![OrderIoDecoded {
                        token: "0x0404040404040404040404040404040404040404".to_string(),
                        vault_id: "0x96".to_string(),
                    }],
                    valid_outputs: vec![
                        OrderIoDecoded {
                            token: "0x0505050505050505050505050505050505050505".to_string(),
                            vault_id: "0x12c".to_string(),
                        },
                        OrderIoDecoded {
                            token: "0x0606060606060606060606060606060606060606".to_string(),
                            vault_id: "0x190".to_string(),
                        },
                    ],
                },
            }),
        )
    }

    fn sample_take_order_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::TakeOrderV3,
            "0x123459",
            "0x64b8c126",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "0x4",
            DecodedEvent::TakeOrderV3(TakeOrderV3Decoded {
                sender: "0x0909090909090909090909090909090909090909".to_string(),
                config: TakeOrderConfigDecoded {
                    order: OrderDecoded {
                        owner: "0x0101010101010101010101010101010101010101".to_string(),
                        nonce: "0x1".to_string(),
                        evaluable: OrderEvaluableDecoded {
                            interpreter: "0x0202020202020202020202020202020202020202".to_string(),
                            store: "0x0303030303030303030303030303030303030303".to_string(),
                            bytecode: "0x01020304".to_string(),
                        },
                        valid_inputs: vec![],
                        valid_outputs: vec![],
                    },
                    input_io_index: "0x0".to_string(),
                    output_io_index: "0x0".to_string(),
                    signed_context: vec![SignedContextDecoded {
                        signer: "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a".to_string(),
                        context: vec!["0x2a".to_string(), "0x2b".to_string()],
                        signature: "0x112233".to_string(),
                    }],
                },
                input: "0x3e8".to_string(),
                output: "0x7d0".to_string(),
            }),
        )
    }

    fn sample_clear_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::ClearV3,
            "0x12345a",
            "0x64b8c127",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567894",
            "0x5",
            DecodedEvent::ClearV3(ClearV3Decoded {
                sender: "0x1111111111111111111111111111111111111111".to_string(),
                alice_owner: "0x0101010101010101010101010101010101010101".to_string(),
                bob_owner: "0x1212121212121212121212121212121212121212".to_string(),
                alice_order_hash:
                    "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                bob_order_hash:
                    "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                alice_input_io_index: "0x0".to_string(),
                alice_output_io_index: "0x0".to_string(),
                bob_input_io_index: "0x0".to_string(),
                bob_output_io_index: "0x0".to_string(),
                alice_bounty_vault_id: "0x0".to_string(),
                bob_bounty_vault_id: "0x0".to_string(),
                alice_input_vault_id: "0x64".to_string(),
                alice_output_vault_id: "0x12c".to_string(),
                bob_input_vault_id: "0x2bc".to_string(),
                bob_output_vault_id: "0x320".to_string(),
            }),
        )
    }

    fn sample_after_clear_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::AfterClearV2,
            "0x12345b",
            "0x64b8c128",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567895",
            "0x6",
            DecodedEvent::AfterClearV2(AfterClearV2Decoded {
                sender: "0x1717171717171717171717171717171717171717".to_string(),
                alice_input: "0x1388".to_string(),
                alice_output: "0x1770".to_string(),
                bob_input: "0x1b58".to_string(),
                bob_output: "0x1f40".to_string(),
            }),
        )
    }

    fn sample_meta_event() -> DecodedEventData<DecodedEvent> {
        build_event(
            EventType::MetaV1_2,
            "0x12345c",
            "0x64b8c129",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567896",
            "0x7",
            DecodedEvent::MetaV1_2(MetaV1_2Decoded {
                sender: "0x1818181818181818181818181818181818181818".to_string(),
                subject: "0x1919191919191919191919191919191919191919".to_string(),
                meta: "0x090a0b0c0d".to_string(),
            }),
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
        assert!(sql.contains("1193046"));
        assert!(sql.contains("1689829667"));
        assert!(sql.contains("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"));
        assert!(sql.contains("1"));
        assert!(sql.contains("0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d"));
        assert!(sql.contains("0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e"));
        assert!(sql.contains("0x258"));
        assert!(sql.contains("0xfa0"));
    }

    #[test]
    fn withdraw_sql_generation() {
        let event = sample_withdraw_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::WithdrawV2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_withdraw_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO withdrawals"));
        assert!(sql.contains("1193047"));
        assert!(sql.contains("1689829668"));
        assert!(sql.contains("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891"));
        assert!(sql.contains("2"));
        assert!(sql.contains("0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"));
        assert!(sql.contains("0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c"));
        assert!(sql.contains("0x1f4"));
        assert!(sql.contains("0xbb8"));
        assert!(sql.contains("0x9c4"));
    }

    #[test]
    fn add_order_sql_generation() {
        let event = sample_add_order_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::AddOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_add_order_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("'AddOrderV3'"));
        assert!(sql.contains("1193048"));
        assert!(sql.contains("1689829669"));
        assert!(sql.contains("0x0707070707070707070707070707070707070707"));
        assert!(sql.contains("0x0808080808080808080808080808080808080808080808080808080808080808"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("0x1"));

        assert!(sql.contains("INSERT INTO order_ios"));
        assert!(sql.contains("'input'"));
        assert!(sql.contains("'output'"));
        assert!(sql.contains("0x0404040404040404040404040404040404040404"));
        assert!(sql.contains("0x64"));
        assert!(sql.contains("0x0505050505050505050505050505050505050505"));
        assert!(sql.contains("0xc8"));
        assert!(sql.contains("0x0606060606060606060606060606060606060606"));
        assert!(sql.contains("0x12c"));
    }

    #[test]
    fn remove_order_sql_generation() {
        let event = sample_remove_order_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::RemoveOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_remove_order_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("'RemoveOrderV3'"));
        assert!(sql.contains("1193049"));
        assert!(sql.contains("1689829669"));
        assert!(sql.contains("0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"));
        assert!(sql.contains("0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"));
        assert!(sql.contains("0x2"));
        assert!(sql.contains("INSERT INTO order_ios"));
        assert!(sql.contains("'input'"));
        assert!(sql.contains("'output'"));
        assert!(sql.contains("0x0404040404040404040404040404040404040404"));
        assert!(sql.contains("0x96"));
        assert!(sql.contains("0x0505050505050505050505050505050505050505"));
        assert!(sql.contains("0x12c"));
        assert!(sql.contains("0x0606060606060606060606060606060606060606"));
        assert!(sql.contains("0x190"));
    }

    #[test]
    fn take_order_sql_generation() {
        let event = sample_take_order_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::TakeOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_take_order_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO take_orders"));
        assert!(sql.contains("1193049"));
        assert!(sql.contains("1689829670"));
        assert!(sql.contains("0x0909090909090909090909090909090909090909"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("0"));
        assert!(sql.contains("0x3e8"));
        assert!(sql.contains("0x7d0"));
        assert!(sql.contains("INSERT INTO take_order_contexts"));
        assert!(
            sql.contains("signer:0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a,signature:0x112233")
        );
        assert!(sql.contains("INSERT INTO context_values"));
        assert!(sql.contains("'0x2a'"));
        assert!(sql.contains("'0x2b'"));
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
        assert!(sql.contains("1193050"));
        assert!(sql.contains("1689829671"));
        assert!(sql.contains("0x1111111111111111111111111111111111111111"));
        assert!(sql.contains("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(sql.contains("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("0x1212121212121212121212121212121212121212"));
        assert!(sql.contains("0x64"));
        assert!(sql.contains("0x12c"));
        assert!(sql.contains("0x2bc"));
        assert!(sql.contains("0x320"));
    }

    #[test]
    fn after_clear_sql_generation() {
        let event = sample_after_clear_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::AfterClearV2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_after_clear_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO after_clear_v2_events"));
        assert!(sql.contains("1193051"));
        assert!(sql.contains("1689829672"));
        assert!(sql.contains("0x1717171717171717171717171717171717171717"));
        assert!(sql.contains("0x1388"));
        assert!(sql.contains("0x1770"));
        assert!(sql.contains("0x1b58"));
        assert!(sql.contains("0x1f40"));
    }

    #[test]
    fn meta_sql_generation() {
        let event = sample_meta_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::MetaV1_2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_meta_sql(&context, decoded).unwrap();

        assert!(sql.contains("INSERT INTO meta_events"));
        assert!(sql.contains("1193052"));
        assert!(sql.contains("1689829673"));
        assert!(sql.contains("0x1818181818181818181818181818181818181818"));
        assert!(sql.contains("0x1919191919191919191919191919191919191919"));
        assert!(sql.contains("0x090a0b0c0d"));
    }

    #[test]
    fn decoded_events_to_sql_complete() {
        let events = vec![
            sample_deposit_event(),
            sample_withdraw_event(),
            sample_add_order_event(),
            sample_take_order_event(),
            sample_clear_event(),
            sample_after_clear_event(),
            sample_meta_event(),
        ];

        let sql = decoded_events_to_sql(&events, 5_000_000).unwrap();

        assert!(sql.starts_with("BEGIN TRANSACTION;"));
        assert!(sql.ends_with("COMMIT;\n"));
        assert!(sql.contains("UPDATE sync_status SET last_synced_block = 5000000"));
        assert!(sql.contains("INSERT INTO deposits"));
        assert!(sql.contains("INSERT INTO withdrawals"));
        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("INSERT INTO order_ios"));
        assert!(sql.contains("INSERT INTO take_orders"));
        assert!(sql.contains("INSERT INTO take_order_contexts"));
        assert!(sql.contains("INSERT INTO context_values"));
        assert!(sql.contains("INSERT INTO clear_v3_events"));
        assert!(sql.contains("INSERT INTO after_clear_v2_events"));
        assert!(sql.contains("INSERT INTO meta_events"));
    }

    #[test]
    fn decoded_events_handles_unknown_event() {
        let events = vec![
            sample_meta_event(),
            build_event(
                EventType::Unknown,
                "0x0",
                "0x0",
                "0xdeadbeef",
                "0x0",
                DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".to_string(),
                    note: "unknown".to_string(),
                }),
            ),
        ];

        let sql = decoded_events_to_sql(&events, 42).unwrap();
        assert!(sql.contains("UPDATE sync_status SET last_synced_block = 42"));
    }

    #[test]
    fn hex_parse_error_propagates() {
        let mut event = sample_deposit_event();
        event.block_number = "invalid".to_string();
        let events = vec![event];
        let result = decoded_events_to_sql(&events, 1000);
        assert!(matches!(result, Err(InsertError::HexParseError { .. })));
    }

    #[test]
    fn order_ios_generation_empty() {
        let context = EventContext {
            block_number: 1,
            block_timestamp: 2,
            transaction_hash: "0xhash",
            log_index: 3,
        };
        let order = OrderDecoded {
            owner: String::new(),
            nonce: String::new(),
            evaluable: OrderEvaluableDecoded {
                interpreter: String::new(),
                store: String::new(),
                bytecode: String::new(),
            },
            valid_inputs: vec![],
            valid_outputs: vec![],
        };

        let sql = generate_order_ios_sql(&context, &order);
        assert!(sql.is_empty());
    }

    #[test]
    fn order_ios_generation_with_inputs_only() {
        let context = EventContext {
            block_number: 1,
            block_timestamp: 2,
            transaction_hash: "0xhash",
            log_index: 3,
        };
        let order = OrderDecoded {
            owner: String::new(),
            nonce: String::new(),
            evaluable: OrderEvaluableDecoded {
                interpreter: String::new(),
                store: String::new(),
                bytecode: String::new(),
            },
            valid_inputs: vec![OrderIoDecoded {
                token: "0xaaa".to_string(),
                vault_id: "0x1".to_string(),
            }],
            valid_outputs: vec![],
        };

        let sql = generate_order_ios_sql(&context, &order);
        assert!(sql.contains("'input'"));
        assert!(!sql.contains("'output'"));
    }
}
