use super::LocalDb;
use crate::erc20::TokenInfo;
use alloy::primitives::Address;
use serde_json::Value;

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("Failed to parse hex string: {hex_str}")]
    HexParseError { hex_str: String },
    #[error("Raw event must be a JSON object")]
    InvalidRawEventFormat,
    #[error("Topics field must be an array of strings")]
    InvalidTopicsFormat,
    #[error("Failed to serialize raw event payload")]
    RawEventSerialization,
    #[error("Missing decoded_data in {event_type} event")]
    MissingDecodedData { event_type: String },
    #[error("Missing {field} in {event_type} event")]
    MissingEventField { field: String, event_type: String },
    #[error("Context value must be string")]
    InvalidContextValue,
}

impl LocalDb {
    pub fn decoded_events_to_sql(
        &self,
        data: Value,
        end_block: u64,
    ) -> Result<String, InsertError> {
        let mut sql = String::new();

        sql.push_str("BEGIN TRANSACTION;\n\n");

        let events = data.as_array().ok_or(InsertError::InvalidInputFormat)?;

        for event in events {
            match event.get("event_type").and_then(|v| v.as_str()) {
                Some("DepositV2") => {
                    sql.push_str(&generate_deposit_sql(event)?);
                }
                Some("WithdrawV2") => {
                    sql.push_str(&generate_withdraw_sql(event)?);
                }
                Some("AddOrderV3") => {
                    sql.push_str(&generate_add_order_sql(event)?);
                }
                Some("RemoveOrderV3") => {
                    sql.push_str(&generate_remove_order_sql(event)?);
                }
                Some("TakeOrderV3") => {
                    sql.push_str(&generate_take_order_sql(event)?);
                }
                Some("ClearV3") => {
                    sql.push_str(&generate_clear_v3_sql(event)?);
                }
                Some("AfterClearV2") => {
                    sql.push_str(&generate_after_clear_sql(event)?);
                }
                Some("MetaV1_2") => {
                    sql.push_str(&generate_meta_sql(event)?);
                }
                Some("Set") => {
                    sql.push_str(&generate_set_sql(event)?);
                }
                Some(event_type) => {
                    eprintln!("Warning: Unknown event type: {}", event_type);
                }
                None => {
                    eprintln!("Warning: Event missing event_type field");
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

    /// Build a SQL transaction that optionally injects `prefix_sql` immediately after BEGIN TRANSACTION.
    pub fn decoded_events_to_sql_with_prefix(
        &self,
        data: Value,
        end_block: u64,
        prefix_sql: &str,
    ) -> Result<String, InsertError> {
        let base = self.decoded_events_to_sql(data, end_block)?;
        if prefix_sql.is_empty() {
            return Ok(base);
        }

        let marker = "BEGIN TRANSACTION;\n\n";
        if let Some(pos) = base.find(marker) {
            let mut out = String::with_capacity(base.len() + prefix_sql.len() + 1);
            out.push_str(&base[..pos]);
            out.push_str(marker);
            out.push_str(prefix_sql);
            if !prefix_sql.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&base[pos + marker.len()..]);
            Ok(out)
        } else {
            // Fallback: prepend prefix at the beginning
            Ok(format!("{}{}", prefix_sql, base))
        }
    }

    pub fn raw_events_to_sql(&self, raw_events: &[Value]) -> Result<String, InsertError> {
        if raw_events.is_empty() {
            return Ok(String::new());
        }

        struct RawEventRow {
            block_number: u64,
            log_index: u64,
            block_timestamp: Option<u64>,
            transaction_hash: String,
            address: String,
            data: String,
            topics_json: String,
            raw_json: String,
        }

        let mut rows: Vec<RawEventRow> = Vec::with_capacity(raw_events.len());

        for event in raw_events {
            if !event.is_object() {
                return Err(InsertError::InvalidRawEventFormat);
            }

            let transaction_hash = get_string_field(event, "transactionHash")?.to_string();
            let log_index = hex_to_decimal(get_string_field(event, "logIndex")?)?;
            let block_number = hex_to_decimal(get_string_field(event, "blockNumber")?)?;
            let block_timestamp = event
                .get("blockTimestamp")
                .and_then(|v| v.as_str())
                .map(hex_to_decimal)
                .transpose()?;
            let address = get_string_field(event, "address")?.to_string();
            let data = get_string_field(event, "data")?.to_string();

            let topics = event
                .get("topics")
                .and_then(|v| v.as_array())
                .ok_or(InsertError::InvalidTopicsFormat)?;
            let mut topics_vec = Vec::with_capacity(topics.len());
            for topic in topics {
                let topic_str = topic
                    .as_str()
                    .ok_or(InsertError::InvalidTopicsFormat)?
                    .to_string();
                topics_vec.push(topic_str);
            }
            let topics_json = serde_json::to_string(&topics_vec)
                .map_err(|_| InsertError::RawEventSerialization)?;

            let raw_json =
                serde_json::to_string(event).map_err(|_| InsertError::RawEventSerialization)?;

            rows.push(RawEventRow {
                block_number,
                log_index,
                block_timestamp,
                transaction_hash,
                address,
                data,
                topics_json,
                raw_json,
            });
        }

        rows.sort_by(|a, b| {
            a.block_number
                .cmp(&b.block_number)
                .then_with(|| a.log_index.cmp(&b.log_index))
        });

        let mut sql = String::new();
        for row in rows {
            let timestamp_sql = row
                .block_timestamp
                .map(|ts| ts.to_string())
                .unwrap_or_else(|| "NULL".to_string());

            sql.push_str(&format!(
                "INSERT INTO raw_events (block_number, block_timestamp, transaction_hash, log_index, address, topics, data, raw_json) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}');\n",
                row.block_number,
                timestamp_sql,
                escape_sql_text(&row.transaction_hash),
                row.log_index,
                escape_sql_text(&row.address),
                escape_sql_text(&row.topics_json),
                escape_sql_text(&row.data),
                escape_sql_text(&row.raw_json),
            ));
        }

        Ok(sql)
    }
}

/// Build upsert SQL for erc20_tokens. Only include successfully fetched tokens.
pub fn generate_erc20_tokens_sql(chain_id: u32, tokens: &[(Address, TokenInfo)]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let mut sql = String::new();
    sql.push_str("INSERT INTO erc20_tokens (chain_id, address, name, symbol, decimals) VALUES ");

    let mut first = true;
    for (addr, info) in tokens.iter() {
        let address_str = format!("0x{:x}", addr);
        let name = info.name.replace('\'', "''");
        let symbol = info.symbol.replace('\'', "''");
        let decimals = info.decimals as u32; // store as INTEGER
        if !first {
            sql.push_str(", ");
        }
        first = false;
        sql.push_str(&format!(
            "({}, '{}', '{}', '{}', {})",
            chain_id, address_str, name, symbol, decimals
        ));
    }

    sql.push_str(
        " ON CONFLICT(chain_id, address) DO UPDATE SET decimals = excluded.decimals, name = excluded.name, symbol = excluded.symbol;\n",
    );
    sql
}

fn generate_deposit_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "DepositV2".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let deposit_amount = get_string_field(decoded_data, "deposit_amount")?;
    let deposit_amount_uint256 = get_string_field(decoded_data, "deposit_amount_uint256")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO deposits (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount, deposit_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount, deposit_amount_uint256
    ));

    Ok(sql)
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

fn generate_add_order_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "AddOrderV3".to_string(),
        })?;
    let order = decoded_data
        .get("order")
        .ok_or(InsertError::MissingEventField {
            field: "order".to_string(),
            event_type: "AddOrderV3".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;
    let order_bytes = get_string_field(decoded_data, "order_bytes")?;
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;
    let evaluable = order
        .get("evaluable")
        .ok_or(InsertError::MissingEventField {
            field: "order.evaluable".to_string(),
            event_type: "AddOrderV3".to_string(),
        })?;
    let interpreter_address = get_string_field(evaluable, "interpreter")?;
    let store_address = get_string_field(evaluable, "store")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, interpreter_address, store_address, order_hash, order_owner, order_nonce, order_bytes) VALUES ({}, {}, '{}', {}, 'AddOrderV3', '{}', '{}', '{}', '{}', '{}', '{}', '{}');\n",
        block_number,
        block_timestamp,
        transaction_hash,
        log_index,
        sender,
        interpreter_address,
        store_address,
        order_hash,
        owner,
        nonce,
        order_bytes
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_remove_order_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "RemoveOrderV3".to_string(),
        })?;
    let order = decoded_data
        .get("order")
        .ok_or(InsertError::MissingEventField {
            field: "order".to_string(),
            event_type: "RemoveOrderV3".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;
    let order_bytes = get_string_field(decoded_data, "order_bytes")?;
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;
    let evaluable = order
        .get("evaluable")
        .ok_or(InsertError::MissingEventField {
            field: "order.evaluable".to_string(),
            event_type: "RemoveOrderV3".to_string(),
        })?;
    let interpreter_address = get_string_field(evaluable, "interpreter")?;
    let store_address = get_string_field(evaluable, "store")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, interpreter_address, store_address, order_hash, order_owner, order_nonce, order_bytes) VALUES ({}, {}, '{}', {}, 'RemoveOrderV3', '{}', '{}', '{}', '{}', '{}', '{}', '{}');\n",
        block_number,
        block_timestamp,
        transaction_hash,
        log_index,
        sender,
        interpreter_address,
        store_address,
        order_hash,
        owner,
        nonce,
        order_bytes
    ));

    let ios_sql = generate_order_ios_sql(context, &decoded.order);
    if !ios_sql.is_empty() {
        sql.push_str(&ios_sql);
    }

    Ok(sql)
}

fn generate_set_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "Set".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let store_address_raw =
        event
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or(InsertError::MissingEventField {
                field: "address".to_string(),
                event_type: "Set".to_string(),
            })?;
    let store_address = store_address_raw.to_ascii_lowercase();

    let namespace = get_string_field(decoded_data, "namespace")?;
    let key = get_string_field(decoded_data, "key")?;
    let value = get_string_field(decoded_data, "value")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO interpreter_store_sets (store_address, block_number, block_timestamp, transaction_hash, log_index, namespace, key, value) VALUES ('{}', {}, {}, '{}', {}, '{}', '{}', '{}') ON CONFLICT(transaction_hash, log_index) DO UPDATE SET store_address = excluded.store_address, block_number = excluded.block_number, block_timestamp = excluded.block_timestamp, namespace = excluded.namespace, key = excluded.key, value = excluded.value;\n",
        store_address,
        block_number,
        block_timestamp,
        transaction_hash,
        log_index,
        namespace,
        key,
        value
    ));

    Ok(sql)
}

fn generate_take_order_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "TakeOrderV3".to_string(),
        })?;
    let config = decoded_data
        .get("config")
        .ok_or(InsertError::MissingEventField {
            field: "config".to_string(),
            event_type: "TakeOrderV3".to_string(),
        })?;
    let order = config.get("order").ok_or(InsertError::MissingEventField {
        field: "order".to_string(),
        event_type: "TakeOrderV3 config".to_string(),
    })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;

    let order_owner = get_string_field(order, "owner")?;
    let order_nonce = get_string_field(order, "nonce")?;

    let input_io_index = hex_to_decimal(get_string_field(config, "input_io_index")?)?;
    let output_io_index = hex_to_decimal(get_string_field(config, "output_io_index")?)?;
    let input_amount = get_string_field(decoded_data, "taker_input")?;
    let output_amount = get_string_field(decoded_data, "taker_output")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO take_orders (block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, taker_input, taker_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, input_amount, output_amount
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

fn escape_sql_text(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raindex_client::sqlite_web::decode::{EventType, UnknownEventDecoded};
    use alloy::primitives::{Address, Bytes, U256};
    use rain_orderbook_bindings::IOrderBookV5::{
        ClearConfigV2, EvaluableV4, SignedContextV1, TakeOrderConfigV4,
    };

    fn create_sample_deposit_event() -> serde_json::Value {
        json!({
            "event_type": "DepositV2",
            "block_number": "0x123456",
            "block_timestamp": "0x64b8c123",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "log_index": "0x1",
            "decoded_data": {
                "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                "vault_id": "0x258",
                "deposit_amount": "0x0",
                "deposit_amount_uint256": "0xfa0"
            }
        })
    }

    fn create_sample_withdraw_event() -> serde_json::Value {
        json!({
            "event_type": "WithdrawV2",
            "block_number": "0x123457",
            "block_timestamp": "0x64b8c124",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891",
            "log_index": "0x2",
            "decoded_data": {
                "sender": "0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
                "token": "0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c",
                "vault_id": "0x1f4",
                "target_amount": "0xbb8",
                "withdraw_amount": "0x9c4",
                "withdraw_amount_uint256": "0x9c4"
            }
        })
    }

    fn create_sample_add_order_event() -> serde_json::Value {
        json!({
            "event_type": "AddOrderV3",
            "block_number": "0x123458",
            "block_timestamp": "0x64b8c125",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892",
            "log_index": "0x3",
            "decoded_data": {
                "sender": "0x0707070707070707070707070707070707070707",
                "order_hash": "0x0808080808080808080808080808080808080808080808080808080808080808",
                "order_bytes": "0x112233",
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1",
                    "evaluable": {
                        "interpreter": "0x0202020202020202020202020202020202020202",
                        "store": "0x0303030303030303030303030303030303030303",
                        "bytecode": "0x01020304"
                    },
                    "valid_inputs": [
                        {
                            "token": "0x0404040404040404040404040404040404040404",
                            "vault_id": "0x64"
                        },
                        {
                            "token": "0x0505050505050505050505050505050505050505",
                            "vault_id": "0xc8"
                        }
                    ],
                    "valid_outputs": [
                        {
                            "token": "0x0606060606060606060606060606060606060606",
                            "vault_id": "0x12c"
                        }
                    ]
                }
            }
        })
    }

    fn create_sample_take_order_event() -> serde_json::Value {
        json!({
            "event_type": "TakeOrderV3",
            "block_number": "0x123459",
            "block_timestamp": "0x64b8c126",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "log_index": "0x4",
            "decoded_data": {
                "sender": "0x0909090909090909090909090909090909090909",
                "config": {
                    "order": {
                        "owner": "0x0101010101010101010101010101010101010101",
                        "nonce": "0x1"
                    },
                    "input_io_index": "0x0",
                    "output_io_index": "0x0",
                    "signed_context": [
                        {
                            "signer": "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a",
                            "signature": "0x112233",
                            "context": ["0x2a", "0x2b"]
                        }
                    ]
                },
                "taker_input": "0x3e8",
                "taker_output": "0x7d0"
            }
        })
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
    fn test_after_clear_sql_generation() {
        let after_clear_event = create_sample_after_clear_event();
        let result = generate_after_clear_sql(&after_clear_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
    fn test_meta_sql_generation() {
        let meta_event = create_sample_meta_event();
        let result = generate_meta_sql(&meta_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

        assert!(sql.contains("INSERT INTO meta_events"));
        assert!(sql.contains("1193052"));
        assert!(sql.contains("1689829673"));
        assert!(sql.contains("0x1818181818181818181818181818181818181818"));
        assert!(sql.contains("0x1919191919191919191919191919191919191919"));
        assert!(sql.contains("0x090a0b0c0d"));
    }

    #[test]
    fn test_decoded_events_to_sql_complete() {
        let events = json!([
            create_sample_deposit_event(),
            create_sample_withdraw_event(),
            create_sample_add_order_event(),
            create_sample_take_order_event(),
            create_sample_clear_v3_event(),
            create_sample_after_clear_event(),
            create_sample_meta_event()
        ]);

        let result = LocalDb::default().decoded_events_to_sql(events, 5000000);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
    fn test_missing_decoded_data_error() {
        let invalid_event = json!({
            "event_type": "DepositV2",
            "block_number": "0x123456",
            "block_timestamp": "0x64b8c123",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "log_index": "0x1"
        });

        let result = generate_deposit_sql(&invalid_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::MissingDecodedData { .. }
        ));
    }

    #[test]
    fn test_missing_field_error() {
        let invalid_event = json!({
            "event_type": "DepositV2",
            "block_number": "0x123456",
            "block_timestamp": "0x64b8c123",
            "log_index": "0x1",
            "decoded_data": {
                "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                "vault_id": "0x258",
                "deposit_amount_uint256": "0xfa0"
            }
        });

        let result = generate_deposit_sql(&invalid_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::MissingField { .. }
        ));
    }

    #[test]
    fn test_hex_parse_error() {
        let invalid_event = json!({
            "event_type": "DepositV2",
            "block_number": "invalid_hex",
            "block_timestamp": "0x64b8c123",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "log_index": "0x1",
            "decoded_data": {
                "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                "vault_id": "0x258",
                "deposit_amount_uint256": "0xfa0"
            }
        });

        let result = generate_deposit_sql(&invalid_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::HexParseError { .. }
        ));
    }

    #[test]
    fn test_invalid_input_format_error() {
        let not_array = json!({"not": "an_array"});
        let result = LocalDb::default().decoded_events_to_sql(not_array, 1000);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::InvalidInputFormat
        ));
    }

    #[test]
    fn test_unknown_event_type_warning() {
        let unknown_event = json!({
            "event_type": "UnknownEvent",
            "block_number": "0x123456",
            "block_timestamp": "0x64b8c123",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "log_index": "0x1",
            "decoded_data": {}
        });

        let events = json!([unknown_event]);
        let result = LocalDb::default().decoded_events_to_sql(events, 1000);

        assert!(result.is_ok());
        let sql = result.unwrap();
        assert!(sql.contains("BEGIN TRANSACTION;"));
        assert!(sql.contains("COMMIT;"));
        assert!(sql.contains("UPDATE sync_status"));
    }

    #[test]
    fn test_empty_events_array() {
        let empty_array = json!([]);
        let result = LocalDb::default().decoded_events_to_sql(empty_array, 1000);

        assert!(result.is_ok());
        let sql = result.unwrap();
        assert!(sql.contains("BEGIN TRANSACTION;"));
        assert!(sql.contains("COMMIT;"));
        assert!(sql.contains("UPDATE sync_status SET last_synced_block = 1000"));
    }

    #[test]
    fn test_hex_to_decimal_conversion() {
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

        assert!(get_string_field(&test_value, "number_field").is_err());
        assert!(matches!(
            get_string_field(&test_value, "number_field").unwrap_err(),
            InsertError::MissingField { .. }
        ));

        assert!(get_string_field(&test_value, "missing_field").is_err());
        assert!(matches!(
            get_string_field(&test_value, "missing_field").unwrap_err(),
            InsertError::MissingField { .. }
        ));

        assert!(get_string_field(&test_value, "null_field").is_err());
        assert!(matches!(
            get_string_field(&test_value, "null_field").unwrap_err(),
            InsertError::MissingField { .. }
        ));
    }

    #[test]
    fn test_order_ios_generation_empty_arrays() {
        let order_with_empty_ios = json!({
            "owner": "0x0101010101010101010101010101010101010101",
            "nonce": "0x1",
            "valid_inputs": [],
            "valid_outputs": []
        });

        let result = generate_order_ios_sql(&order_with_empty_ios, "0xtest", 1);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert_eq!(sql, "");
    }

    #[test]
    fn test_take_order_with_no_signed_context() {
        let take_order_event = json!({
            "event_type": "TakeOrderV3",
            "block_number": "0x123459",
            "block_timestamp": "0x64b8c126",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "log_index": "0x4",
            "decoded_data": {
                "sender": "0x0909090909090909090909090909090909090909",
                "config": {
                    "order": {
                        "owner": "0x0101010101010101010101010101010101010101",
                        "nonce": "0x1"
                    },
                    "input_io_index": "0x0",
                    "output_io_index": "0x0",
                    "signed_context": []
                },
                "taker_input": "0x3e8",
                "taker_output": "0x7d0"
            }
        });

        let result = generate_take_order_sql(&take_order_event);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert!(sql.contains("INSERT INTO take_orders"));
        assert!(!sql.contains("INSERT INTO take_order_contexts"));
        assert!(!sql.contains("INSERT INTO context_values"));
    }

    #[test]
    fn test_invalid_context_value_error() {
        let take_order_event = json!({
            "event_type": "TakeOrderV3",
            "block_number": "0x123459",
            "block_timestamp": "0x64b8c126",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "log_index": "0x4",
            "decoded_data": {
                "sender": "0x0909090909090909090909090909090909090909",
                "config": {
                    "order": {
                        "owner": "0x0101010101010101010101010101010101010101",
                        "nonce": "0x1"
                    },
                    "input_io_index": "0x0",
                    "output_io_index": "0x0",
                    "signed_context": [
                        {
                            "signer": "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a",
                            "signature": "0x112233",
                            "context": [123]
                        }
                    ]
                },
                "taker_input": "0x3e8",
                "taker_output": "0x7d0"
            }
        });

        let result = generate_take_order_sql(&take_order_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::InvalidContextValue
        ));
    }

    #[test]
    fn test_missing_order_field_in_add_order() {
        let invalid_event = json!({
            "event_type": "AddOrderV3",
            "block_number": "0x123458",
            "block_timestamp": "0x64b8c125",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892",
            "log_index": "0x3",
            "decoded_data": {
                "sender": "0x0707070707070707070707070707070707070707",
                "order_hash": "0x0808080808080808080808080808080808080808080808080808080808080808"
            }
        });

        let result = generate_add_order_sql(&invalid_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::MissingEventField { .. }
        ));
    }

    #[test]
    fn test_missing_config_field_in_take_order() {
        let invalid_event = json!({
            "event_type": "TakeOrderV3",
            "block_number": "0x123459",
            "block_timestamp": "0x64b8c126",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "log_index": "0x4",
            "decoded_data": {
                "sender": "0x0909090909090909090909090909090909090909",
                "taker_input": "0x3e8",
                "taker_output": "0x7d0"
            }
        });

        let result = generate_take_order_sql(&invalid_event);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::MissingEventField { .. }
        ));
    }

    fn create_sample_remove_order_event() -> serde_json::Value {
        json!({
            "event_type": "RemoveOrderV3",
            "block_number": "0x123459",
            "block_timestamp": "0x64b8c126",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "log_index": "0x4",
            "decoded_data": {
                "sender": "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a",
                "order_hash": "0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
                "order_bytes": "0xaabbcc",
                "order": {
                    "owner": "0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c",
                    "nonce": "0x2",
                    "evaluable": {
                        "interpreter": "0x0202020202020202020202020202020202020202",
                        "store": "0x0303030303030303030303030303030303030303",
                        "bytecode": "0x05060708"
                    },
                    "valid_inputs": [
                        {
                            "token": "0x0404040404040404040404040404040404040404",
                            "vault_id": "0x96"
                        }
                    ],
                    "valid_outputs": [
                        {
                            "token": "0x0505050505050505050505050505050505050505",
                            "vault_id": "0x12c"
                        },
                        {
                            "token": "0x0606060606060606060606060606060606060606",
                            "vault_id": "0x190"
                        }
                    ]
                }
            }
        })
    }

    #[test]
    fn test_remove_order_sql_generation() {
        let remove_order_event = create_sample_remove_order_event();
        let result = generate_remove_order_sql(&remove_order_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("'RemoveOrderV3'"));
        assert!(sql.contains("1193049"));
        assert!(sql.contains("1689829670"));
        assert!(sql.contains("0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"));
        assert!(sql.contains("0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"));
        assert!(sql.contains("0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c"));
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
    fn test_missing_valid_inputs_in_order() {
        let order_without_inputs = json!({
            "event_type": "AddOrderV3",
            "block_number": "0x123458",
            "block_timestamp": "0x64b8c125",
            "transaction_hash": "0xtest_missing_inputs",
            "log_index": "0x3",
            "decoded_data": {
                "sender": "0x0707070707070707070707070707070707070707",
                "order_hash": "0x0808080808080808080808080808080808080808080808080808080808080808",
                "order_bytes": "0x112233",
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1",
                    "evaluable": {
                        "interpreter": "0x0202020202020202020202020202020202020202",
                        "store": "0x0303030303030303030303030303030303030303",
                        "bytecode": "0x"
                    },
                    "valid_outputs": [
                        {
                            "token": "0x0606060606060606060606060606060606060606",
                            "vault_id": "0x12c"
                        }
                    ]
                }
            }
        });

        let result = generate_add_order_sql(&order_without_inputs);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("INSERT INTO order_ios"));
        assert!(sql.contains("'output'"));
        assert!(!sql.contains("'input'"));
    }

    #[test]
    fn test_missing_valid_outputs_in_order() {
        let order_without_outputs = json!({
            "event_type": "RemoveOrderV3",
            "block_number": "0x123458",
            "block_timestamp": "0x64b8c125",
            "transaction_hash": "0xtest_missing_outputs",
            "log_index": "0x3",
            "decoded_data": {
                "sender": "0x0707070707070707070707070707070707070707",
                "order_hash": "0x0808080808080808080808080808080808080808080808080808080808080808",
                "order_bytes": "0xaabbcc",
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1",
                    "evaluable": {
                        "interpreter": "0x0202020202020202020202020202020202020202",
                        "store": "0x0303030303030303030303030303030303030303",
                        "bytecode": "0x"
                    },
                    "valid_inputs": [
                        {
                            "token": "0x0404040404040404040404040404040404040404",
                            "vault_id": "0x64"
                        }
                    ]
                }
            }
        });

        let result = generate_remove_order_sql(&order_without_outputs);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert!(sql.contains("INSERT INTO order_events"));
        assert!(sql.contains("INSERT INTO order_ios"));
        assert!(sql.contains("'input'"));
        assert!(!sql.contains("'output'"));
    }

    #[test]
    fn test_missing_both_valid_inputs_and_outputs() {
        let order_without_ios = json!({
            "event_type": "AddOrderV3",
            "block_number": "0x123458",
            "block_timestamp": "0x64b8c125",
            "transaction_hash": "0xtest_no_ios",
            "log_index": "0x3",
            "decoded_data": {
                "sender": "0x0707070707070707070707070707070707070707",
                "order_hash": "0x0808080808080808080808080808080808080808080808080808080808080808",
                "order_bytes": "0x445566",
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1"
                    ,"evaluable": {
                        "interpreter": "0x0202020202020202020202020202020202020202",
                        "store": "0x0303030303030303030303030303030303030303",
                        "bytecode": "0x"
                    }
                }
            }
        });

        let result = generate_add_order_sql(&order_without_ios);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert!(sql.contains("INSERT INTO order_events"));
        assert!(!sql.contains("INSERT INTO order_ios"));
    }

    #[test]
    fn test_block_number_tracking() {
        let events = json!([
            {
                "event_type": "DepositV2",
                "block_number": "0x100",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0xtest1",
                "log_index": "0x1",
                "decoded_data": {
                    "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                    "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                    "vault_id": "0x258",
                    "deposit_amount": "0x0",
                    "deposit_amount_uint256": "0xfa0"
                }
            },
            {
                "event_type": "DepositV2",
                "block_number": "0x200",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0xtest2",
                "log_index": "0x2",
                "decoded_data": {
                    "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                    "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                    "vault_id": "0x258",
                    "deposit_amount": "0x0",
                    "deposit_amount_uint256": "0xfa0"
                }
            }
        ]);

        let result = LocalDb::default().decoded_events_to_sql(events, 1000);
        assert!(result.is_ok());

        let sql = result.unwrap();
        assert!(sql.contains("UPDATE sync_status SET last_synced_block = 1000"));
    }

    #[test]
    fn test_malformed_block_number_in_main_function() {
        let events_with_invalid_block = json!([
            {
                "event_type": "DepositV2",
                "block_number": "invalid_hex",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0xtest1",
                "log_index": "0x1",
                "decoded_data": {
                    "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                    "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                    "vault_id": "0x258",
                    "deposit_amount_uint256": "0xfa0"
                }
            },
            {
                "event_type": "DepositV2",
                "block_number": "0x200",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0xtest2",
                "log_index": "0x2",
                "decoded_data": {
                    "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                    "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                    "vault_id": "0x258",
                    "deposit_amount_uint256": "0xfa0"
                }
            }
        ]);

        let result = LocalDb::default().decoded_events_to_sql(events_with_invalid_block, 1000);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::HexParseError { .. }
        ));
    }

    #[test]
    fn test_missing_block_number_in_main_function() {
        let events_without_block_number = json!([
            {
                "event_type": "DepositV2",
                "block_timestamp": "0x64b8c123",
                "transaction_hash": "0xtest1",
                "log_index": "0x1",
                "decoded_data": {
                    "sender": "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d",
                    "token": "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e",
                    "vault_id": "0x258",
                    "deposit_amount_uint256": "0xfa0"
                }
            }
        ]);

        let result = LocalDb::default().decoded_events_to_sql(events_without_block_number, 1000);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InsertError::MissingField { .. }
        ));
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

    #[test]
    fn test_decoded_events_to_sql_with_prefix_injection() {
        let events = serde_json::json!([]);
        let base = LocalDb::default()
            .decoded_events_to_sql(events.clone(), 0)
            .unwrap();
        assert!(base.starts_with("BEGIN TRANSACTION;\n\n"));

        let prefixed = LocalDb::default()
            .decoded_events_to_sql_with_prefix(events, 0, "-- prefix sql\n")
            .unwrap();
        let expected = "BEGIN TRANSACTION;\n\n-- prefix sql\n";
        assert!(prefixed.starts_with(expected));
    }

    #[test]
    fn test_raw_events_sql_sorted_and_handles_null_timestamp() {
        let events = vec![
            serde_json::json!({
                "blockNumber": "0x2",
                "transactionHash": "0xbbb",
                "logIndex": "0x1",
                "address": "0x2222222222222222222222222222222222222222",
                "data": "0xdeadbeef",
                "topics": ["0x01", "0x02"],
                "blockTimestamp": "0x64b8c125"
            }),
            serde_json::json!({
                "blockNumber": "0x1",
                "transactionHash": "0xaaa",
                "logIndex": "0x0",
                "address": "0x1111111111111111111111111111111111111111",
                "data": "0xbead",
                "topics": ["0x01"],
                "blockTimestamp": "0x64b8c124"
            }),
            serde_json::json!({
                "blockNumber": "0x3",
                "transactionHash": "0xccc",
                "logIndex": "0x0",
                "address": "0x3333333333333333333333333333333333333333",
                "data": "0xfeed",
                "topics": ["0x01"],
                "blockTimestamp": null
            }),
        ];

        let sql = LocalDb::default().raw_events_to_sql(&events).unwrap();
        assert!(sql.contains("INSERT INTO raw_events"));

        let first_pos = sql.find("0xaaa").unwrap();
        let second_pos = sql.find("0xbbb").unwrap();
        let third_pos = sql.find("0xccc").unwrap();
        assert!(first_pos < second_pos && second_pos < third_pos);

        assert!(sql.contains("VALUES (3, NULL,"));
        assert!(sql.contains("[\"0x01\",\"0x02\"]"));
    }

    #[test]
    fn test_raw_events_sql_invalid_topics() {
        let events = vec![serde_json::json!({
            "blockNumber": "0x1",
            "transactionHash": "0xaaa",
            "logIndex": "0x0",
            "address": "0x1111111111111111111111111111111111111111",
            "data": "0xbead",
            "topics": "not-an-array"
        })];

        let result = LocalDb::default().raw_events_to_sql(&events);
        assert!(matches!(result, Err(InsertError::InvalidTopicsFormat)));
    }
}
