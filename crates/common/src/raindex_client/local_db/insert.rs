use super::decode::{DecodedEvent, DecodedEventData};
use crate::erc20::TokenInfo;
use alloy::sol_types::SolValue;
use alloy::{
    hex,
    primitives::{keccak256, Address, FixedBytes, U256},
};
use rain_math_float::Float;
use rain_orderbook_bindings::IOrderBookV5::{
    AddOrderV3, AfterClearV2, ClearV3, DepositV2, OrderV4, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    IOV2,
};
use rain_orderbook_bindings::OrderBook::MetaV1_2;
use std::collections::HashMap;
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
    #[error("Missing decimals for token {token}")]
    MissingTokenDecimals { token: String },
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
    decimals_by_token: &HashMap<Address, u8>,
    prefix_sql: Option<&str>,
) -> Result<String, InsertError> {
    let mut sql = String::new();
    sql.push_str("BEGIN TRANSACTION;\n\n");

    if let Some(prefix) = prefix_sql {
        if !prefix.is_empty() {
            sql.push_str(prefix);
            if !prefix.ends_with('\n') {
                sql.push('\n');
            }
        }
    }

    for event in events {
        match &event.decoded_data {
            DecodedEvent::DepositV2(decoded) => {
                let context = event_context(event)?;
                sql.push_str(&generate_deposit_sql(
                    &context,
                    decoded.as_ref(),
                    decimals_by_token,
                )?);
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

/// Build upsert SQL for erc20_tokens. Only include successfully fetched tokens.
pub fn generate_erc20_tokens_sql(chain_id: u32, tokens: &[(Address, TokenInfo)]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let mut sql = String::new();
    sql.push_str(
        r#"INSERT INTO erc20_tokens (
    chain_id,
    address,
    name,
    symbol,
    decimals
) VALUES "#,
    );

    let mut first = true;
    for (addr, info) in tokens.iter() {
        let address_str = format!("0x{:x}", addr);
        let address_literal = sql_string_literal(&address_str);
        let name_literal = sql_string_literal(&info.name);
        let symbol_literal = sql_string_literal(&info.symbol);
        let decimals = info.decimals as u32; // store as INTEGER
        if !first {
            sql.push_str(", ");
        }
        first = false;
        sql.push_str(&format!(
            "({}, {}, {}, {}, {})",
            chain_id, address_literal, name_literal, symbol_literal, decimals
        ));
    }

    sql.push_str(
        " ON CONFLICT(chain_id, address) DO UPDATE SET decimals = excluded.decimals, name = excluded.name, symbol = excluded.symbol;\n",
    );
    sql
}

fn sql_string_literal(value: &str) -> String {
    let mut literal = String::with_capacity(value.len() + 2);
    literal.push('\'');
    for ch in value.chars() {
        match ch {
            '\'' => literal.push_str("''"),
            _ => literal.push(ch),
        }
    }
    literal.push('\'');
    literal
}

fn generate_deposit_sql(
    context: &EventContext<'_>,
    decoded: &DepositV2,
    decimals_by_token: &HashMap<Address, u8>,
) -> Result<String, InsertError> {
    let decimals = decimals_by_token
        .get(&decoded.token)
        .copied()
        .ok_or_else(|| InsertError::MissingTokenDecimals {
            token: hex::encode_prefixed(decoded.token),
        })?;

    let deposit_amount_float = Float::from_fixed_decimal(decoded.depositAmountUint256, decimals)
        .map_err(|err| InsertError::FloatConversion(err.to_string()))?;

    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let token = hex::encode_prefixed(decoded.token);
    let vault_id = hex::encode_prefixed(decoded.vaultId);
    let deposit_amount = deposit_amount_float.as_hex();
    let deposit_amount_uint256 = encode_u256_prefixed(&decoded.depositAmountUint256);

    Ok(format!(
        r#"INSERT INTO deposits (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    token,
    vault_id,
    deposit_amount,
    deposit_amount_uint256
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{token}',
    '{vault_id}',
    '{deposit_amount}',
    '{deposit_amount_uint256}'
);
"#,
    ))
}

fn generate_withdraw_sql(
    context: &EventContext<'_>,
    decoded: &WithdrawV2,
) -> Result<String, InsertError> {
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let token = hex::encode_prefixed(decoded.token);
    let vault_id = hex::encode_prefixed(decoded.vaultId);
    let target_amount = hex::encode_prefixed(decoded.targetAmount);
    let withdraw_amount = hex::encode_prefixed(decoded.withdrawAmount);
    let withdraw_amount_uint256 = encode_u256_prefixed(&decoded.withdrawAmountUint256);

    Ok(format!(
        r#"INSERT INTO withdrawals (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    token,
    vault_id,
    target_amount,
    withdraw_amount,
    withdraw_amount_uint256
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{token}',
    '{vault_id}',
    '{target_amount}',
    '{withdraw_amount}',
    '{withdraw_amount_uint256}'
);
"#,
    ))
}

fn generate_add_order_sql(
    context: &EventContext<'_>,
    decoded: &AddOrderV3,
) -> Result<String, InsertError> {
    let mut sql = String::new();
    let order_bytes = hex::encode_prefixed(decoded.order.abi_encode());
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_hash = hex::encode_prefixed(decoded.orderHash);
    let order_owner = hex::encode_prefixed(decoded.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.order.nonce);

    sql.push_str(&format!(
        r#"INSERT INTO order_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    event_type,
    sender,
    order_hash,
    order_owner,
    order_nonce,
    order_bytes
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    'AddOrderV3',
    '{sender}',
    '{order_hash}',
    '{order_owner}',
    '{order_nonce}',
    '{order_bytes}'
);
"#,
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
    let order_bytes = hex::encode_prefixed(decoded.order.abi_encode());
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_hash = hex::encode_prefixed(decoded.orderHash);
    let order_owner = hex::encode_prefixed(decoded.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.order.nonce);

    sql.push_str(&format!(
        r#"INSERT INTO order_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    event_type,
    sender,
    order_hash,
    order_owner,
    order_nonce,
    order_bytes
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    'RemoveOrderV3',
    '{sender}',
    '{order_hash}',
    '{order_owner}',
    '{order_nonce}',
    '{order_bytes}'
);
"#,
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
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_owner = hex::encode_prefixed(decoded.config.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.config.order.nonce);
    let taker_input = hex::encode_prefixed(decoded.input);
    let taker_output = hex::encode_prefixed(decoded.output);
    let input_io_index = input_io_index_u64;
    let output_io_index = output_io_index_u64;

    sql.push_str(&format!(
        r#"INSERT INTO take_orders (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    order_owner,
    order_nonce,
    input_io_index,
    output_io_index,
    taker_input,
    taker_output
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{order_owner}',
    '{order_nonce}',
    {input_io_index},
    {output_io_index},
    '{taker_input}',
    '{taker_output}'
);
"#,
    ));

    for (context_index, signed_context) in decoded.config.signedContext.iter().enumerate() {
        let context_value = format!(
            "signer:{},signature:{}",
            hex::encode_prefixed(signed_context.signer),
            hex::encode_prefixed(&signed_context.signature)
        );

        sql.push_str(&format!(
            r#"INSERT INTO take_order_contexts (
    transaction_hash,
    log_index,
    context_index,
    context_value
) VALUES (
    '{transaction_hash}',
    {log_index},
    {context_index},
    '{context_value}'
);
"#,
        ));

        for (value_index, value) in signed_context.context.iter().enumerate() {
            let value_hex = hex::encode_prefixed(value);
            sql.push_str(&format!(
                r#"INSERT INTO context_values (
    transaction_hash,
    log_index,
    context_index,
    value_index,
    value
) VALUES (
    '{transaction_hash}',
    {log_index},
    {context_index},
    {value_index},
    '{value_hex}'
);
"#,
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
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let alice_order_owner = hex::encode_prefixed(decoded.alice.owner);
    let alice_input_io_index = alice_input_io_index_u64;
    let alice_output_io_index = alice_output_io_index_u64;
    let alice_bounty_vault_id = hex::encode_prefixed(decoded.clearConfig.aliceBountyVaultId);
    let alice_input_vault_id_hex = hex::encode_prefixed(alice_input_vault_id);
    let alice_output_vault_id_hex = hex::encode_prefixed(alice_output_vault_id);
    let bob_order_owner = hex::encode_prefixed(decoded.bob.owner);
    let bob_input_io_index = bob_input_io_index_u64;
    let bob_output_io_index = bob_output_io_index_u64;
    let bob_bounty_vault_id = hex::encode_prefixed(decoded.clearConfig.bobBountyVaultId);
    let bob_input_vault_id_hex = hex::encode_prefixed(bob_input_vault_id);
    let bob_output_vault_id_hex = hex::encode_prefixed(bob_output_vault_id);

    Ok(format!(
        r#"INSERT INTO clear_v3_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    alice_order_hash,
    alice_order_owner,
    alice_input_io_index,
    alice_output_io_index,
    alice_bounty_vault_id,
    alice_input_vault_id,
    alice_output_vault_id,
    bob_order_hash,
    bob_order_owner,
    bob_input_io_index,
    bob_output_io_index,
    bob_bounty_vault_id,
    bob_input_vault_id,
    bob_output_vault_id
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{alice_order_hash}',
    '{alice_order_owner}',
    {alice_input_io_index},
    {alice_output_io_index},
    '{alice_bounty_vault_id}',
    '{alice_input_vault_id_hex}',
    '{alice_output_vault_id_hex}',
    '{bob_order_hash}',
    '{bob_order_owner}',
    {bob_input_io_index},
    {bob_output_io_index},
    '{bob_bounty_vault_id}',
    '{bob_input_vault_id_hex}',
    '{bob_output_vault_id_hex}'
);
"#,
    ))
}

fn generate_after_clear_sql(
    context: &EventContext<'_>,
    decoded: &AfterClearV2,
) -> Result<String, InsertError> {
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let alice_input = hex::encode_prefixed(decoded.clearStateChange.aliceInput);
    let alice_output = hex::encode_prefixed(decoded.clearStateChange.aliceOutput);
    let bob_input = hex::encode_prefixed(decoded.clearStateChange.bobInput);
    let bob_output = hex::encode_prefixed(decoded.clearStateChange.bobOutput);

    Ok(format!(
        r#"INSERT INTO after_clear_v2_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    alice_input,
    alice_output,
    bob_input,
    bob_output
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{alice_input}',
    '{alice_output}',
    '{bob_input}',
    '{bob_output}'
);
"#,
    ))
}

fn generate_meta_sql(
    context: &EventContext<'_>,
    decoded: &MetaV1_2,
) -> Result<String, InsertError> {
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let subject = hex::encode_prefixed(decoded.subject);
    let meta = hex::encode_prefixed(&decoded.meta);

    Ok(format!(
        r#"INSERT INTO meta_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    subject,
    meta
) VALUES (
    {block_number},
    {block_timestamp},
    '{transaction_hash}',
    {log_index},
    '{sender}',
    '{subject}',
    '{meta}'
);
"#,
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
        r#"INSERT INTO order_ios (
    transaction_hash,
    log_index,
    io_index,
    io_type,
    token,
    vault_id
) VALUES {};\n"#,
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
    use crate::raindex_client::local_db::LocalDb;
    use alloy::primitives::{Address, Bytes, FixedBytes, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        ClearConfigV2, EvaluableV4, SignedContextV1, TakeOrderConfigV4,
    };
    use std::collections::HashMap;

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

    fn sample_add_event() -> DecodedEventData<DecodedEvent> {
        let add = AddOrderV3 {
            sender: Address::from([0x07; 20]),
            orderHash: FixedBytes::<32>::from([0x08; 32]),
            order: sample_order(),
        };

        build_event(
            EventType::AddOrderV3,
            "0x100",
            "0x200",
            "0xaaa",
            "0x1",
            DecodedEvent::AddOrderV3(Box::new(add)),
        )
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
        let deposit = decoded.as_ref();
        let mut decimals = HashMap::new();
        decimals.insert(deposit.token, 6);
        let sql = generate_deposit_sql(&context, deposit, &decimals).unwrap();
        assert!(sql.contains("INSERT INTO deposits"));
        assert!(sql.contains("0x0000000000000000000000000000000000000000000000000000000000000fa0"));
    }

    #[test]
    fn add_order_sql_includes_order_bytes() {
        let event = sample_add_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::AddOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let sql = generate_add_order_sql(&context, decoded).unwrap();
        assert!(sql.contains("order_bytes"));
        let expected_bytes = hex::encode_prefixed(decoded.order.abi_encode());
        assert!(sql.contains(&expected_bytes));
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
        let mut decimals = HashMap::new();
        if let DecodedEvent::DepositV2(deposit) = &deposit_event.decoded_data {
            decimals.insert(deposit.token, 6);
        }
        let sql =
            decoded_events_to_sql(&[deposit_event, clear_event], 0x200, &decimals, None).unwrap();
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
        let sql = decoded_events_to_sql(&[unknown_event], 0, &HashMap::new(), None).unwrap();
        assert!(sql.contains("BEGIN TRANSACTION"));
    }

    #[test]
    fn test_generate_erc20_tokens_sql_builder() {
        let addr1 = Address::from([1u8; 20]);
        let addr2 = Address::from([2u8; 20]);
        let tokens = vec![
            (
                addr1,
                TokenInfo {
                    decimals: 18,
                    name: "Foo Token".to_string(),
                    symbol: "FOO".to_string(),
                },
            ),
            (
                addr2,
                TokenInfo {
                    decimals: 6,
                    name: "Bar's Token".to_string(),
                    symbol: "B'AR".to_string(),
                },
            ),
        ];

        let sql = generate_erc20_tokens_sql(1, &tokens);
        assert!(sql.starts_with("INSERT INTO erc20_tokens"));
        assert!(sql.contains("ON CONFLICT(chain_id, address) DO UPDATE"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("0x0202020202020202020202020202020202020202"));
        // Apostrophes should be doubled
        assert!(sql.contains("Bar''s Token"));
        assert!(sql.contains("B''AR"));
    }

    fn assert_literal_round_trip(literal: &str, expected: &str) {
        assert!(literal.starts_with('\'') && literal.ends_with('\''));
        let mut chars = literal[1..literal.len() - 1].chars().peekable();
        let mut reconstructed = String::new();
        while let Some(ch) = chars.next() {
            if ch == '\'' {
                match chars.next() {
                    Some('\'') => reconstructed.push('\''),
                    _ => panic!("found unescaped single quote in literal: {}", literal),
                }
            } else {
                reconstructed.push(ch);
            }
        }
        assert_eq!(reconstructed, expected);
    }

    #[test]
    fn sql_string_literal_round_trip_special_characters() {
        let value = "Alice's \"Token\" \\ Backslash\nNewline\rCarriage\tTab; DROP TABLE tokens; --";
        let literal = super::sql_string_literal(value);
        assert!(literal.contains("''"));
        assert!(literal.contains('"'));
        assert!(literal.contains('\\'));
        assert_literal_round_trip(&literal, value);
    }

    #[test]
    fn generate_erc20_tokens_sql_escapes_special_characters() {
        let addr = Address::from([3u8; 20]);
        let name =
            "Dangerous 'Name' \"Quotes\" \\ Backslash\nNewline; DROP TABLE tokens; --".to_string();
        let symbol = "SYM'\"\\\n;--".to_string();
        let tokens = vec![(
            addr,
            TokenInfo {
                decimals: 8,
                name: name.clone(),
                symbol: symbol.clone(),
            },
        )];

        let sql = generate_erc20_tokens_sql(5, &tokens);

        let expected_tuple = format!(
            "({}, {}, {}, {}, {})",
            5,
            super::sql_string_literal(&format!("0x{:x}", addr)),
            super::sql_string_literal(&name),
            super::sql_string_literal(&symbol),
            8u32
        );

        assert!(sql.starts_with("INSERT INTO erc20_tokens"));
        assert!(sql.contains(&expected_tuple));
        assert!(sql.ends_with(
            " ON CONFLICT(chain_id, address) DO UPDATE SET decimals = excluded.decimals, name = excluded.name, symbol = excluded.symbol;\n"
        ));

        let address_literal = super::sql_string_literal(&format!("0x{:x}", addr));
        let name_literal = super::sql_string_literal(&name);
        let symbol_literal = super::sql_string_literal(&symbol);

        assert!(sql.contains(&address_literal));
        assert!(sql.contains(&name_literal));
        assert!(sql.contains(&symbol_literal));

        assert_literal_round_trip(&address_literal, &format!("0x{:x}", addr));
        assert_literal_round_trip(&name_literal, &name);
        assert_literal_round_trip(&symbol_literal, &symbol);
    }

    #[test]
    fn test_decoded_events_to_sql_with_prefix_injection() {
        let events: Vec<DecodedEventData<DecodedEvent>> = Vec::new();
        let base = LocalDb::default()
            .decoded_events_to_sql(&events, 0, &HashMap::new(), None)
            .unwrap();
        assert!(base.starts_with("BEGIN TRANSACTION;\n\n"));

        let prefix = "-- prefix sql\n";
        let prefixed = LocalDb::default()
            .decoded_events_to_sql(&events, 0, &HashMap::new(), Some(prefix))
            .unwrap();
        let expected = format!("BEGIN TRANSACTION;\n\n{}", prefix);
        assert!(prefixed.starts_with(&expected));
    }
}
