use super::LocalDb;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum InsertError {
    #[error("Input data is not a JSON array")]
    InvalidInputFormat,
    #[error("Missing or invalid {field} field")]
    MissingField { field: String },
    #[error("Failed to parse hex string: {hex_str}")]
    HexParseError { hex_str: String },
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

fn generate_withdraw_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "WithdrawV2".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let target_amount = get_string_field(decoded_data, "target_amount")?;
    let withdraw_amount = get_string_field(decoded_data, "withdraw_amount")?;
    let withdraw_amount_uint256 = get_string_field(decoded_data, "withdraw_amount_uint256")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO withdrawals (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, withdraw_amount, withdraw_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, withdraw_amount, withdraw_amount_uint256
    ));

    Ok(sql)
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
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'AddOrderV3', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_hash, owner, nonce
    ));

    sql.push_str(&generate_order_ios_sql(order, transaction_hash, log_index)?);

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
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, order_owner, order_nonce) VALUES ({}, {}, '{}', {}, 'RemoveOrderV3', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_hash, owner, nonce
    ));

    sql.push_str(&generate_order_ios_sql(order, transaction_hash, log_index)?);

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

    if let Some(signed_contexts) = config.get("signed_context").and_then(|v| v.as_array()) {
        for (context_index, context) in signed_contexts.iter().enumerate() {
            let signer = get_string_field(context, "signer")?;
            let signature = get_string_field(context, "signature")?;

            let context_value = format!("signer:{},signature:{}", signer, signature);
            sql.push_str(&format!(
                "INSERT INTO take_order_contexts (transaction_hash, log_index, context_index, context_value) VALUES ('{}', {}, {}, '{}');\n",
                transaction_hash, log_index, context_index, context_value
            ));

            if let Some(context_values) = context.get("context").and_then(|v| v.as_array()) {
                for (value_index, value) in context_values.iter().enumerate() {
                    let context_value = value.as_str().ok_or(InsertError::InvalidContextValue)?;
                    sql.push_str(&format!(
                        "INSERT INTO context_values (transaction_hash, log_index, context_index, value_index, value) VALUES ('{}', {}, {}, {}, '{}');\n",
                        transaction_hash, log_index, context_index, value_index, context_value
                    ));
                }
            }
        }
    }

    Ok(sql)
}

fn generate_clear_v3_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "ClearV3".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let alice_owner = get_string_field(decoded_data, "alice_owner")?;
    let bob_owner = get_string_field(decoded_data, "bob_owner")?;
    let alice_order_hash = get_string_field(decoded_data, "alice_order_hash")?;
    let bob_order_hash = get_string_field(decoded_data, "bob_order_hash")?;
    let alice_input_io_index = get_string_field(decoded_data, "alice_input_io_index")?;
    let alice_output_io_index = get_string_field(decoded_data, "alice_output_io_index")?;
    let alice_bounty_vault_id = get_string_field(decoded_data, "alice_bounty_vault_id")?;
    let alice_input_vault_id = get_string_field(decoded_data, "alice_input_vault_id")?;
    let alice_output_vault_id = get_string_field(decoded_data, "alice_output_vault_id")?;
    let bob_input_io_index = get_string_field(decoded_data, "bob_input_io_index")?;
    let bob_output_io_index = get_string_field(decoded_data, "bob_output_io_index")?;
    let bob_bounty_vault_id = get_string_field(decoded_data, "bob_bounty_vault_id")?;
    let bob_input_vault_id = get_string_field(decoded_data, "bob_input_vault_id")?;
    let bob_output_vault_id = get_string_field(decoded_data, "bob_output_vault_id")?;

    let mut sql = String::new();

    let alice_input_io_index_num = hex_to_decimal(alice_input_io_index)?;
    let alice_output_io_index_num = hex_to_decimal(alice_output_io_index)?;
    let bob_input_io_index_num = hex_to_decimal(bob_input_io_index)?;
    let bob_output_io_index_num = hex_to_decimal(bob_output_io_index)?;

    sql.push_str(&format!(
        "INSERT INTO clear_v3_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_order_hash, alice_order_owner, alice_input_io_index, alice_output_io_index, alice_bounty_vault_id, alice_input_vault_id, alice_output_vault_id, bob_order_hash, bob_order_owner, bob_input_io_index, bob_output_io_index, bob_bounty_vault_id, bob_input_vault_id, bob_output_vault_id) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, alice_order_hash, alice_owner, alice_input_io_index_num, alice_output_io_index_num, alice_bounty_vault_id, alice_input_vault_id, alice_output_vault_id, bob_order_hash, bob_owner, bob_input_io_index_num, bob_output_io_index_num, bob_bounty_vault_id, bob_input_vault_id, bob_output_vault_id
    ));

    Ok(sql)
}

fn generate_after_clear_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "AfterClearV2".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let alice_input = get_string_field(decoded_data, "alice_input")?;
    let alice_output = get_string_field(decoded_data, "alice_output")?;
    let bob_input = get_string_field(decoded_data, "bob_input")?;
    let bob_output = get_string_field(decoded_data, "bob_output")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO after_clear_v2_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output
    ));

    Ok(sql)
}

fn generate_meta_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData {
            event_type: "MetaV1_2".to_string(),
        })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let subject = get_string_field(decoded_data, "subject")?;
    let meta = get_string_field(decoded_data, "meta")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO meta_events (block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta
    ));

    Ok(sql)
}

fn generate_order_ios_sql(
    order: &Value,
    transaction_hash: &str,
    log_index: u64,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    let mut all_ios = Vec::new();

    if let Some(inputs) = order.get("valid_inputs").and_then(|v| v.as_array()) {
        for (index, input) in inputs.iter().enumerate() {
            let token = get_string_field(input, "token")?;
            let vault_id = get_string_field(input, "vault_id")?;

            all_ios.push(format!(
                "('{}', {}, {}, 'input', '{}', '{}')",
                transaction_hash, log_index, index, token, vault_id
            ));
        }
    }

    if let Some(outputs) = order.get("valid_outputs").and_then(|v| v.as_array()) {
        for (index, output) in outputs.iter().enumerate() {
            let token = get_string_field(output, "token")?;
            let vault_id = get_string_field(output, "vault_id")?;

            all_ios.push(format!(
                "('{}', {}, {}, 'output', '{}', '{}')",
                transaction_hash, log_index, index, token, vault_id
            ));
        }
    }

    if !all_ios.is_empty() {
        sql.push_str("INSERT INTO order_ios (transaction_hash, log_index, io_index, io_type, token, vault_id) VALUES ");
        sql.push_str(&all_ios.join(", "));
        sql.push_str(";\n");
    }

    Ok(sql)
}

fn get_string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, InsertError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| InsertError::MissingField {
            field: field.to_string(),
        })
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
    use serde_json::json;

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

    fn create_sample_clear_v3_event() -> serde_json::Value {
        json!({
            "event_type": "ClearV3",
            "block_number": "0x12345a",
            "block_timestamp": "0x64b8c127",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567894",
            "log_index": "0x5",
            "decoded_data": {
                "sender": "0x1111111111111111111111111111111111111111",
                "alice_owner": "0x0101010101010101010101010101010101010101",
                "bob_owner": "0x1212121212121212121212121212121212121212",
                "alice_order_hash": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bob_order_hash": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "alice_input_io_index": "0x0",
                "alice_output_io_index": "0x0",
                "alice_bounty_vault_id": "0x0",
                "alice_input_vault_id": "0x64",
                "alice_output_vault_id": "0x12c",
                "bob_input_io_index": "0x0",
                "bob_output_io_index": "0x0",
                "bob_bounty_vault_id": "0x0",
                "bob_input_vault_id": "0x2bc",
                "bob_output_vault_id": "0x320"
            }
        })
    }

    fn create_sample_after_clear_event() -> serde_json::Value {
        json!({
            "event_type": "AfterClearV2",
            "block_number": "0x12345b",
            "block_timestamp": "0x64b8c128",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567895",
            "log_index": "0x6",
            "decoded_data": {
                "sender": "0x1717171717171717171717171717171717171717",
                "alice_input": "0x1388",
                "alice_output": "0x1770",
                "bob_input": "0x1b58",
                "bob_output": "0x1f40"
            }
        })
    }

    fn create_sample_meta_event() -> serde_json::Value {
        json!({
            "event_type": "MetaV1_2",
            "block_number": "0x12345c",
            "block_timestamp": "0x64b8c129",
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567896",
            "log_index": "0x7",
            "decoded_data": {
                "sender": "0x1818181818181818181818181818181818181818",
                "subject": "0x1919191919191919191919191919191919191919",
                "meta": "0x090a0b0c0d"
            }
        })
    }

    #[test]
    fn test_deposit_sql_generation() {
        let deposit_event = create_sample_deposit_event();
        let result = generate_deposit_sql(&deposit_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
    fn test_withdraw_sql_generation() {
        let withdraw_event = create_sample_withdraw_event();
        let result = generate_withdraw_sql(&withdraw_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
    fn test_add_order_sql_generation() {
        let add_order_event = create_sample_add_order_event();
        let result = generate_add_order_sql(&add_order_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
    fn test_take_order_sql_generation() {
        let take_order_event = create_sample_take_order_event();
        let result = generate_take_order_sql(&take_order_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

        assert!(sql.contains("INSERT INTO take_orders"));
        assert!(sql.contains("1193049"));
        assert!(sql.contains("1689829670"));
        assert!(sql.contains("0x0909090909090909090909090909090909090909"));
        assert!(sql.contains("0x0101010101010101010101010101010101010101"));
        assert!(sql.contains("0x1"));
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
    fn test_clear_v3_sql_generation() {
        let clear_event = create_sample_clear_v3_event();
        let result = generate_clear_v3_sql(&clear_event);

        assert!(result.is_ok());
        let sql = result.unwrap();

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
        assert_eq!(hex_to_decimal("10").unwrap(), 16);
        assert_eq!(hex_to_decimal("0xFF").unwrap(), 255);
        assert_eq!(hex_to_decimal("ff").unwrap(), 255);

        assert!(hex_to_decimal("invalid").is_err());
        assert!(matches!(
            hex_to_decimal("invalid").unwrap_err(),
            InsertError::HexParseError { .. }
        ));
    }

    #[test]
    fn test_get_string_field() {
        let test_value = json!({
            "string_field": "test_value",
            "number_field": 42,
            "null_field": null
        });

        assert_eq!(
            get_string_field(&test_value, "string_field").unwrap(),
            "test_value"
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
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1",
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
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1",
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
                "order": {
                    "owner": "0x0101010101010101010101010101010101010101",
                    "nonce": "0x1"
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
}
