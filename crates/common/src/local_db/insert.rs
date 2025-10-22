use super::decode::{DecodedEvent, DecodedEventData, InterpreterStoreSetEvent};
use super::query::{SqlStatement, SqlStatementBatch, SqlValue};
use crate::{erc20::TokenInfo, rpc_client::LogEntryResponse};
use alloy::sol_types::SolValue;
use alloy::{
    hex,
    primitives::{keccak256, Address, FixedBytes, U256},
};
use itertools::Itertools;
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
    #[error("Failed to serialize raw event payload: {0}")]
    RawEventSerialization(String),
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

pub fn decoded_events_to_statement(
    events: &[DecodedEventData<DecodedEvent>],
    end_block: u64,
    decimals_by_token: &HashMap<Address, u8>,
    prefix_sql: Option<&str>,
) -> Result<SqlStatementBatch, InsertError> {
    let mut batch = SqlStatementBatch::new();

    if let Some(prefix) = prefix_sql {
        if !prefix.is_empty() {
            batch.add(SqlStatement::new(prefix));
        }
    }

    for event in events {
        let context = event_context(event)?;
        match &event.decoded_data {
            DecodedEvent::DepositV2(decoded) => {
                batch.add(generate_deposit_statement(
                    &context,
                    decoded.as_ref(),
                    decimals_by_token,
                )?);
            }
            DecodedEvent::WithdrawV2(decoded) => {
                batch.add(generate_withdraw_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::AddOrderV3(decoded) => {
                let add_event = decoded.as_ref();
                batch.add(generate_add_order_sql(&context, add_event)?);
                batch.extend(generate_order_ios_sql(&context, &add_event.order));
            }
            DecodedEvent::RemoveOrderV3(decoded) => {
                let remove_event = decoded.as_ref();
                batch.add(generate_remove_order_sql(&context, remove_event)?);
                batch.extend(generate_order_ios_sql(&context, &remove_event.order));
            }
            DecodedEvent::TakeOrderV3(decoded) => {
                let take_event = decoded.as_ref();
                batch.add(generate_take_order_sql(&context, take_event)?);
                batch.extend(generate_take_order_contexts(&context, take_event));
                batch.extend(generate_take_order_context_values(&context, take_event));
            }
            DecodedEvent::ClearV3(decoded) => {
                batch.add(generate_clear_v3_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::AfterClearV2(decoded) => {
                batch.add(generate_after_clear_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::MetaV1_2(decoded) => {
                batch.add(generate_meta_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::InterpreterStoreSet(decoded) => {
                batch.add(generate_store_set_sql(&context, decoded.as_ref())?);
            }
            DecodedEvent::Unknown(decoded) => {
                eprintln!(
                    "Warning: Unknown event type for transaction {}: {}",
                    event.transaction_hash, decoded.note
                );
            }
        }
    }

    batch.add(SqlStatement::new(format!(
        "UPDATE sync_status SET last_synced_block = {}, updated_at = CURRENT_TIMESTAMP WHERE id = 1;",
        end_block
    )));

    Ok(batch)
}

pub fn raw_events_to_sql(raw_events: &[LogEntryResponse]) -> Result<String, InsertError> {
    struct RawEventRow<'a> {
        block_number: u64,
        log_index: u64,
        block_timestamp: Option<u64>,
        event: &'a LogEntryResponse,
        topics_json: String,
        raw_json: String,
    }

    let rows = raw_events
        .iter()
        .map(|event| {
            let block_number = hex_to_decimal(&event.block_number)?;
            let log_index = hex_to_decimal(&event.log_index)?;
            let block_timestamp = event
                .block_timestamp
                .as_deref()
                .map(hex_to_decimal)
                .transpose()?;
            let topics_json = serde_json::to_string(&event.topics)
                .map_err(|err| InsertError::RawEventSerialization(err.to_string()))?;
            let raw_json = serde_json::to_string(&event)
                .map_err(|err| InsertError::RawEventSerialization(err.to_string()))?;
            Ok(RawEventRow {
                block_number,
                log_index,
                block_timestamp,
                event,
                topics_json,
                raw_json,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let sql = rows
        .iter()
        .sorted_by(|a, b| {
            a.block_number
                .cmp(&b.block_number)
                .then_with(|| a.log_index.cmp(&b.log_index))
        })
        .map(|row| {
            let timestamp_sql = row
                .block_timestamp
                .map_or_else(|| "NULL".to_string(), |ts| ts.to_string());
            format!(
                r#"INSERT INTO raw_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    address,
    topics,
    data,
    raw_json
) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}');
"#,
                row.block_number,
                timestamp_sql,
                escape_sql_text(&row.event.transaction_hash),
                row.log_index,
                escape_sql_text(&row.event.address),
                escape_sql_text(&row.topics_json),
                escape_sql_text(&row.event.data),
                escape_sql_text(&row.raw_json),
            )
        })
        .collect::<String>();

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

fn generate_deposit_statement(
    context: &EventContext<'_>,
    decoded: &DepositV2,
    decimals_by_token: &HashMap<Address, u8>,
) -> Result<SqlStatement, InsertError> {
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

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(token),
            SqlValue::from(vault_id),
            SqlValue::from(deposit_amount),
            SqlValue::from(deposit_amount_uint256),
        ],
    ))
}

fn generate_withdraw_sql(
    context: &EventContext<'_>,
    decoded: &WithdrawV2,
) -> Result<SqlStatement, InsertError> {
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

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9,
    ?10
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(token),
            SqlValue::from(vault_id),
            SqlValue::from(target_amount),
            SqlValue::from(withdraw_amount),
            SqlValue::from(withdraw_amount_uint256),
        ],
    ))
}

fn generate_add_order_sql(
    context: &EventContext<'_>,
    decoded: &AddOrderV3,
) -> Result<SqlStatement, InsertError> {
    let order_bytes = hex::encode_prefixed(decoded.order.abi_encode());
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_hash = hex::encode_prefixed(decoded.orderHash);
    let order_owner = hex::encode_prefixed(decoded.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.order.nonce);

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    'AddOrderV3',
    ?5,
    ?6,
    ?7,
    ?8,
    ?9
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(order_hash),
            SqlValue::from(order_owner),
            SqlValue::from(order_nonce),
            SqlValue::from(order_bytes),
        ],
    ))
}

fn generate_remove_order_sql(
    context: &EventContext<'_>,
    decoded: &RemoveOrderV3,
) -> Result<SqlStatement, InsertError> {
    let order_bytes = hex::encode_prefixed(decoded.order.abi_encode());
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_hash = hex::encode_prefixed(decoded.orderHash);
    let order_owner = hex::encode_prefixed(decoded.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.order.nonce);

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    'RemoveOrderV3',
    ?5,
    ?6,
    ?7,
    ?8,
    ?9
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(order_hash),
            SqlValue::from(order_owner),
            SqlValue::from(order_nonce),
            SqlValue::from(order_bytes),
        ],
    ))
}

fn generate_take_order_sql(
    context: &EventContext<'_>,
    decoded: &TakeOrderV3,
) -> Result<SqlStatement, InsertError> {
    let input_io_index_u64 = u256_to_u64(&decoded.config.inputIOIndex, "inputIOIndex")?;
    let output_io_index_u64 = u256_to_u64(&decoded.config.outputIOIndex, "outputIOIndex")?;

    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let order_owner = hex::encode_prefixed(decoded.config.order.owner);
    let order_nonce = hex::encode_prefixed(decoded.config.order.nonce);
    let taker_input = hex::encode_prefixed(decoded.input);
    let taker_output = hex::encode_prefixed(decoded.output);

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9,
    ?10,
    ?11
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(order_owner),
            SqlValue::from(order_nonce),
            SqlValue::from(input_io_index_u64),
            SqlValue::from(output_io_index_u64),
            SqlValue::from(taker_input),
            SqlValue::from(taker_output),
        ],
    ))
}

fn generate_take_order_contexts(
    context: &EventContext<'_>,
    decoded: &TakeOrderV3,
) -> SqlStatementBatch {
    const INSERT_CONTEXT_SQL: &str = r#"INSERT INTO take_order_contexts (
    transaction_hash,
    log_index,
    context_index,
    context_value
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4
);
"#;

    let mut batch = SqlStatementBatch::new();
    let transaction_hash = context.transaction_hash.to_owned();
    let log_index = context.log_index;

    for (context_index, signed_context) in decoded.config.signedContext.iter().enumerate() {
        let context_value = format!(
            "signer:{},signature:{}",
            hex::encode_prefixed(signed_context.signer),
            hex::encode_prefixed(&signed_context.signature)
        );

        batch.add(SqlStatement::new_with_params(
            INSERT_CONTEXT_SQL,
            vec![
                SqlValue::from(transaction_hash.clone()),
                SqlValue::from(log_index),
                SqlValue::from(context_index as u64),
                SqlValue::from(context_value),
            ],
        ));
    }

    batch
}

fn generate_take_order_context_values(
    context: &EventContext<'_>,
    decoded: &TakeOrderV3,
) -> SqlStatementBatch {
    const INSERT_VALUE_SQL: &str = r#"INSERT INTO context_values (
    transaction_hash,
    log_index,
    context_index,
    value_index,
    value
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5
);
"#;

    let mut batch = SqlStatementBatch::new();
    let transaction_hash = context.transaction_hash.to_owned();
    let log_index = context.log_index;

    for (context_index, signed_context) in decoded.config.signedContext.iter().enumerate() {
        for (value_index, value) in signed_context.context.iter().enumerate() {
            batch.add(SqlStatement::new_with_params(
                INSERT_VALUE_SQL,
                vec![
                    SqlValue::from(transaction_hash.clone()),
                    SqlValue::from(log_index),
                    SqlValue::from(context_index as u64),
                    SqlValue::from(value_index as u64),
                    SqlValue::from(hex::encode_prefixed(value)),
                ],
            ));
        }
    }

    batch
}

fn generate_clear_v3_sql(
    context: &EventContext<'_>,
    decoded: &ClearV3,
) -> Result<SqlStatement, InsertError> {
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

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9,
    ?10,
    ?11,
    ?12,
    ?13,
    ?14,
    ?15,
    ?16,
    ?17,
    ?18,
    ?19
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(alice_order_hash),
            SqlValue::from(alice_order_owner),
            SqlValue::from(alice_input_io_index),
            SqlValue::from(alice_output_io_index),
            SqlValue::from(alice_bounty_vault_id),
            SqlValue::from(alice_input_vault_id_hex),
            SqlValue::from(alice_output_vault_id_hex),
            SqlValue::from(bob_order_hash),
            SqlValue::from(bob_order_owner),
            SqlValue::from(bob_input_io_index),
            SqlValue::from(bob_output_io_index),
            SqlValue::from(bob_bounty_vault_id),
            SqlValue::from(bob_input_vault_id_hex),
            SqlValue::from(bob_output_vault_id_hex),
        ],
    ))
}

fn generate_after_clear_sql(
    context: &EventContext<'_>,
    decoded: &AfterClearV2,
) -> Result<SqlStatement, InsertError> {
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let alice_input = hex::encode_prefixed(decoded.clearStateChange.aliceInput);
    let alice_output = hex::encode_prefixed(decoded.clearStateChange.aliceOutput);
    let bob_input = hex::encode_prefixed(decoded.clearStateChange.bobInput);
    let bob_output = hex::encode_prefixed(decoded.clearStateChange.bobOutput);

    Ok(SqlStatement::new_with_params(
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
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8,
    ?9
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(alice_input),
            SqlValue::from(alice_output),
            SqlValue::from(bob_input),
            SqlValue::from(bob_output),
        ],
    ))
}

fn generate_meta_sql(
    context: &EventContext<'_>,
    decoded: &MetaV1_2,
) -> Result<SqlStatement, InsertError> {
    let block_number = context.block_number;
    let block_timestamp = context.block_timestamp;
    let transaction_hash = context.transaction_hash;
    let log_index = context.log_index;
    let sender = hex::encode_prefixed(decoded.sender);
    let subject = hex::encode_prefixed(decoded.subject);
    let meta = hex::encode_prefixed(&decoded.meta);

    Ok(SqlStatement::new_with_params(
        r#"INSERT INTO meta_events (
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    sender,
    subject,
    meta
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7
);
"#,
        vec![
            SqlValue::from(block_number),
            SqlValue::from(block_timestamp),
            SqlValue::from(transaction_hash.to_owned()),
            SqlValue::from(log_index),
            SqlValue::from(sender),
            SqlValue::from(subject),
            SqlValue::from(meta),
        ],
    ))
}

fn generate_store_set_sql(
    context: &EventContext<'_>,
    decoded: &InterpreterStoreSetEvent,
) -> Result<SqlStatement, InsertError> {
    Ok(SqlStatement::new_with_params(
        r#"INSERT INTO interpreter_store_sets (
    store_address,
    block_number,
    block_timestamp,
    transaction_hash,
    log_index,
    namespace,
    key,
    value
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8
) ON CONFLICT(transaction_hash, log_index) DO UPDATE SET
    store_address = excluded.store_address,
    block_number = excluded.block_number,
    block_timestamp = excluded.block_timestamp,
    namespace = excluded.namespace,
    key = excluded.key,
    value = excluded.value;
"#,
        vec![
            SqlValue::from(hex::encode_prefixed(decoded.store_address)),
            SqlValue::from(context.block_number),
            SqlValue::from(context.block_timestamp),
            SqlValue::from(context.transaction_hash.to_owned()),
            SqlValue::from(context.log_index),
            SqlValue::from(hex::encode_prefixed(decoded.namespace)),
            SqlValue::from(hex::encode_prefixed(decoded.key)),
            SqlValue::from(hex::encode_prefixed(decoded.value)),
        ],
    ))
}

fn generate_order_ios_sql(context: &EventContext<'_>, order: &OrderV4) -> SqlStatementBatch {
    const INSERT_IO_SQL: &str = r#"INSERT INTO order_ios (
    transaction_hash,
    log_index,
    io_index,
    io_type,
    token,
    vault_id
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6
);
"#;

    let mut batch = SqlStatementBatch::new();
    let transaction_hash = context.transaction_hash.to_owned();
    let log_index = context.log_index;

    for (index, input) in order.validInputs.iter().enumerate() {
        batch.add(SqlStatement::new_with_params(
            INSERT_IO_SQL,
            vec![
                SqlValue::from(transaction_hash.clone()),
                SqlValue::from(log_index),
                SqlValue::from(index as u64),
                SqlValue::from("input"),
                SqlValue::from(hex::encode_prefixed(input.token)),
                SqlValue::from(hex::encode_prefixed(input.vaultId)),
            ],
        ));
    }

    for (index, output) in order.validOutputs.iter().enumerate() {
        batch.add(SqlStatement::new_with_params(
            INSERT_IO_SQL,
            vec![
                SqlValue::from(transaction_hash.clone()),
                SqlValue::from(log_index),
                SqlValue::from(index as u64),
                SqlValue::from("output"),
                SqlValue::from(hex::encode_prefixed(output.token)),
                SqlValue::from(hex::encode_prefixed(output.vaultId)),
            ],
        ));
    }

    batch
}

fn hex_to_decimal(hex_str: &str) -> Result<u64, InsertError> {
    let hex_str_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(hex_str_clean, 16).map_err(|_| InsertError::HexParseError {
        hex_str: hex_str.to_string(),
    })
}

fn escape_sql_text(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::decode::{EventType, UnknownEventDecoded};
    use crate::local_db::query::SqlValue;
    use crate::local_db::LocalDb;
    use crate::rpc_client::LogEntryResponse;
    use alloy::hex;
    use alloy::primitives::{Address, Bytes, FixedBytes, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        ClearConfigV2, ClearStateChangeV2, EvaluableV4, SignedContextV1, TakeOrderConfigV4,
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

    fn sample_after_clear_event() -> DecodedEventData<DecodedEvent> {
        let after_clear = AfterClearV2 {
            sender: Address::from([0x17; 20]),
            clearStateChange: ClearStateChangeV2 {
                aliceInput: U256::from(0x1f4).into(),
                aliceOutput: U256::from(0x258).into(),
                bobInput: U256::from(0x2bc).into(),
                bobOutput: U256::from(0x320).into(),
            },
        };

        build_event(
            EventType::AfterClearV2,
            "0x103",
            "0x203",
            "0x2468",
            "0x5",
            DecodedEvent::AfterClearV2(Box::new(after_clear)),
        )
    }

    fn sample_meta_event() -> DecodedEventData<DecodedEvent> {
        let meta = MetaV1_2 {
            sender: Address::from([0x18; 20]),
            subject: FixedBytes::<32>::from([0x19; 32]),
            meta: Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]),
        };

        build_event(
            EventType::MetaV1_2,
            "0x104",
            "0x204",
            "0x1122",
            "0x6",
            DecodedEvent::MetaV1_2(Box::new(meta)),
        )
    }

    fn sample_store_set_event() -> DecodedEventData<DecodedEvent> {
        let store = InterpreterStoreSetEvent {
            store_address: Address::from([0x30; 20]),
            namespace: FixedBytes::<32>::from([0xaa; 32]),
            key: FixedBytes::<32>::from([0xbb; 32]),
            value: FixedBytes::<32>::from([0xcc; 32]),
        };

        build_event(
            EventType::InterpreterStoreSet,
            "0x200",
            "0x300",
            "0xfeed",
            "0x4",
            DecodedEvent::InterpreterStoreSet(Box::new(store)),
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
    fn store_set_sql_generation() {
        let event = sample_store_set_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::InterpreterStoreSet(decoded) = &event.decoded_data else {
            unreachable!()
        };

        let statement = generate_store_set_sql(&context, decoded.as_ref()).unwrap();
        assert!(statement
            .sql()
            .contains("INSERT INTO interpreter_store_sets"));
        assert!(statement.sql().contains("?8"));
        let params = statement.params();
        assert_eq!(params.len(), 8);
        assert!(
            matches!(params[0], SqlValue::Text(ref v) if v == &hex::encode_prefixed(decoded.store_address))
        );
        assert!(matches!(params[1], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[2], SqlValue::U64(v) if v == context.block_timestamp));
        assert!(matches!(params[3], SqlValue::Text(ref v) if v == context.transaction_hash));
        assert!(matches!(params[4], SqlValue::U64(v) if v == context.log_index));
        assert!(
            matches!(params[5], SqlValue::Text(ref v) if v == &hex::encode_prefixed(decoded.namespace))
        );
        assert!(
            matches!(params[6], SqlValue::Text(ref v) if v == &hex::encode_prefixed(decoded.key))
        );
        assert!(
            matches!(params[7], SqlValue::Text(ref v) if v == &hex::encode_prefixed(decoded.value))
        );
    }

    #[test]
    fn deposit_statement_generation() {
        let event = sample_deposit_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::DepositV2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let deposit = decoded.as_ref();
        let mut decimals = HashMap::new();
        decimals.insert(deposit.token, 6);
        let statement = generate_deposit_statement(&context, deposit, &decimals).unwrap();
        assert!(statement.sql().contains("INSERT INTO deposits"));
        assert!(statement.sql().contains("?9"));
        let params = statement.params();
        assert_eq!(params.len(), 9);
        assert!(matches!(params[0], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[1], SqlValue::U64(v) if v == context.block_timestamp));
        assert!(matches!(params[3], SqlValue::U64(v) if v == context.log_index));
        let expected_uint256 = encode_u256_prefixed(&deposit.depositAmountUint256);
        assert!(matches!(params[8], SqlValue::Text(ref v) if v == &expected_uint256));
    }

    #[test]
    fn add_order_sql_includes_order_bytes() {
        let event = sample_add_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::AddOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let statement = generate_add_order_sql(&context, decoded).unwrap();
        assert!(statement.sql().contains("order_bytes"));
        assert!(statement.sql().contains("?9"));
        let params = statement.params();
        assert_eq!(params.len(), 9);
        let expected_bytes = hex::encode_prefixed(decoded.order.abi_encode());
        assert!(matches!(params[8], SqlValue::Text(ref v) if v == &expected_bytes));
        let ios_batch = generate_order_ios_sql(&context, &decoded.order);
        let expected_ios_len = decoded.order.validInputs.len() + decoded.order.validOutputs.len();
        assert_eq!(ios_batch.len(), expected_ios_len);
        assert!(ios_batch
            .statements()
            .iter()
            .all(|stmt| stmt.sql().contains("INSERT INTO order_ios")));
    }

    #[test]
    fn take_order_sql_generation() {
        let event = sample_take_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::TakeOrderV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let statement = generate_take_order_sql(&context, decoded).unwrap();
        assert!(statement.sql().contains("INSERT INTO take_orders"));
        assert!(statement.sql().contains("?11"));
        let params = statement.params();
        assert_eq!(params.len(), 11);
        assert!(matches!(params[0], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[3], SqlValue::U64(v) if v == context.log_index));
        assert!(
            matches!(params[4], SqlValue::Text(ref v) if v == "0x0909090909090909090909090909090909090909")
        );

        let contexts = generate_take_order_contexts(&context, decoded);
        assert_eq!(contexts.len(), decoded.config.signedContext.len());
        assert!(contexts
            .statements()
            .iter()
            .all(|stmt| stmt.sql().contains("INSERT INTO take_order_contexts")));

        let context_values = generate_take_order_context_values(&context, decoded);
        let expected_values: usize = decoded
            .config
            .signedContext
            .iter()
            .map(|ctx| ctx.context.len())
            .sum();
        assert_eq!(context_values.len(), expected_values);
        assert!(context_values
            .statements()
            .iter()
            .all(|stmt| stmt.sql().contains("INSERT INTO context_values")));
    }

    #[test]
    fn after_clear_sql_generation() {
        let event = sample_after_clear_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::AfterClearV2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let statement = generate_after_clear_sql(&context, decoded).unwrap();
        assert!(statement
            .sql()
            .contains("INSERT INTO after_clear_v2_events"));
        assert!(statement.sql().contains("?9"));
        let params = statement.params();
        assert_eq!(params.len(), 9);
        assert!(matches!(params[0], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[3], SqlValue::U64(v) if v == context.log_index));
        let expected_alice_input = hex::encode_prefixed(decoded.clearStateChange.aliceInput);
        assert!(matches!(params[5], SqlValue::Text(ref v) if v == &expected_alice_input));
        let expected_bob_output = hex::encode_prefixed(decoded.clearStateChange.bobOutput);
        assert!(matches!(params[8], SqlValue::Text(ref v) if v == &expected_bob_output));
    }

    #[test]
    fn meta_sql_generation() {
        let event = sample_meta_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::MetaV1_2(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let statement = generate_meta_sql(&context, decoded).unwrap();
        assert!(statement.sql().contains("INSERT INTO meta_events"));
        assert!(statement.sql().contains("?7"));
        let params = statement.params();
        assert_eq!(params.len(), 7);
        assert!(matches!(params[0], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[3], SqlValue::U64(v) if v == context.log_index));
        let expected_sender = hex::encode_prefixed(decoded.sender);
        assert!(matches!(params[4], SqlValue::Text(ref v) if v == &expected_sender));
        let expected_subject = hex::encode_prefixed(decoded.subject);
        assert!(matches!(params[5], SqlValue::Text(ref v) if v == &expected_subject));
        let expected_meta = hex::encode_prefixed(&decoded.meta);
        assert!(matches!(params[6], SqlValue::Text(ref v) if v == &expected_meta));
    }

    #[test]
    fn clear_v3_sql_generation() {
        let event = sample_clear_event();
        let context = event_context(&event).unwrap();
        let DecodedEvent::ClearV3(decoded) = &event.decoded_data else {
            unreachable!()
        };
        let statement = generate_clear_v3_sql(&context, decoded).unwrap();
        assert!(statement.sql().contains("INSERT INTO clear_v3_events"));
        assert!(statement.sql().contains("?19"));
        let params = statement.params();
        assert_eq!(params.len(), 19);
        assert!(matches!(params[0], SqlValue::U64(v) if v == context.block_number));
        assert!(matches!(params[3], SqlValue::U64(v) if v == context.log_index));
        let expected_alice_input_vault = hex::encode_prefixed(decoded.alice.validInputs[0].vaultId);
        assert!(matches!(params[10], SqlValue::Text(ref v) if v == &expected_alice_input_vault));
        let expected_bob_input_vault = hex::encode_prefixed(decoded.bob.validInputs[0].vaultId);
        assert!(matches!(params[17], SqlValue::Text(ref v) if v == &expected_bob_input_vault));
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
    fn decoded_events_to_statement_multiple_events() {
        let clear_event = sample_clear_event();
        let deposit_event = sample_deposit_event();
        let mut decimals = HashMap::new();
        if let DecodedEvent::DepositV2(deposit) = &deposit_event.decoded_data {
            decimals.insert(deposit.token, 6);
        }
        let batch =
            decoded_events_to_statement(&[deposit_event, clear_event], 0x200, &decimals, None)
                .unwrap()
                .into_transaction()
                .unwrap();
        let sql = batch.statements().iter().map(|stmt| stmt.sql()).join("\n");
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
        let batch = decoded_events_to_statement(&[unknown_event], 0, &HashMap::new(), None)
            .unwrap()
            .into_transaction()
            .unwrap();
        let sql = batch.statements().iter().map(|stmt| stmt.sql()).join("\n");
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
    fn test_decoded_events_to_statement_with_prefix_injection() {
        let events: Vec<DecodedEventData<DecodedEvent>> = Vec::new();
        let base_batch = LocalDb::default()
            .decoded_events_to_statement(&events, 0, &HashMap::new(), None)
            .unwrap()
            .into_transaction()
            .unwrap();
        let base = base_batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql())
            .join("\n");
        assert!(base.starts_with("BEGIN TRANSACTION"));

        let prefix = "-- prefix sql\n";
        let prefixed_batch = LocalDb::default()
            .decoded_events_to_statement(&events, 0, &HashMap::new(), Some(prefix))
            .unwrap()
            .into_transaction()
            .unwrap();
        let prefixed = prefixed_batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql())
            .join("\n");
        let expected = format!("BEGIN TRANSACTION\n{}", prefix.trim_end_matches('\n'));
        assert!(prefixed.starts_with(&expected));
    }

    #[test]
    fn test_raw_events_sql_sorted_and_handles_null_timestamp() {
        let events = vec![
            LogEntryResponse {
                address: "0x2222222222222222222222222222222222222222".to_string(),
                topics: vec!["0x01".to_string(), "0x02".to_string()],
                data: "0xdeadbeef".to_string(),
                block_number: "0x2".to_string(),
                block_timestamp: Some("0x64b8c125".to_string()),
                transaction_hash: "0xbbb".to_string(),
                transaction_index: "0x0".to_string(),
                block_hash: "0x0".to_string(),
                log_index: "0x1".to_string(),
                removed: false,
            },
            LogEntryResponse {
                address: "0x1111111111111111111111111111111111111111".to_string(),
                topics: vec!["0x01".to_string()],
                data: "0xbead".to_string(),
                block_number: "0x1".to_string(),
                block_timestamp: Some("0x64b8c124".to_string()),
                transaction_hash: "0xaaa".to_string(),
                transaction_index: "0x0".to_string(),
                block_hash: "0x0".to_string(),
                log_index: "0x0".to_string(),
                removed: false,
            },
            LogEntryResponse {
                address: "0x3333333333333333333333333333333333333333".to_string(),
                topics: vec!["0x01".to_string()],
                data: "0xfeed".to_string(),
                block_number: "0x3".to_string(),
                block_timestamp: None,
                transaction_hash: "0xccc".to_string(),
                transaction_index: "0x0".to_string(),
                block_hash: "0x0".to_string(),
                log_index: "0x0".to_string(),
                removed: false,
            },
        ];

        let sql = raw_events_to_sql(&events).unwrap();
        assert!(sql.contains("INSERT INTO raw_events"));

        let first_pos = sql.find("0xaaa").unwrap();
        let second_pos = sql.find("0xbbb").unwrap();
        let third_pos = sql.find("0xccc").unwrap();
        assert!(first_pos < second_pos && second_pos < third_pos);

        assert!(sql.contains("VALUES (3, NULL,"));
        assert!(sql.contains("[\"0x01\",\"0x02\"]"));
    }

    #[test]
    fn test_raw_events_sql_invalid_hex() {
        let events = vec![LogEntryResponse {
            address: "0x1111111111111111111111111111111111111111".to_string(),
            topics: vec!["0x01".to_string()],
            data: "0xbead".to_string(),
            block_number: "not-hex".to_string(),
            block_timestamp: Some("0x0".to_string()),
            transaction_hash: "0xaaa".to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0x0".to_string(),
            log_index: "0x0".to_string(),
            removed: false,
        }];

        let result = raw_events_to_sql(&events);
        assert!(matches!(result, Err(InsertError::HexParseError { .. })));
    }
}
