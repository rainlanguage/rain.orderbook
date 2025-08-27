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

pub fn decoded_events_to_sql(
    data: Value,
    end_block: u64,
) -> Result<String, InsertError> {
    let mut sql = String::new();

    sql.push_str("BEGIN TRANSACTION;\n\n");

    let events = data
        .as_array()
        .ok_or(InsertError::InvalidInputFormat)?;

    let mut max_block_number: Option<u64> = None;

    for event in events {
        if let Ok(block_number_str) = get_string_field(event, "block_number") {
            if let Ok(block_number) = hex_to_decimal(block_number_str) {
                max_block_number =
                    Some(max_block_number.map_or(block_number, |max| max.max(block_number)));
            }
        }

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

fn generate_deposit_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData { event_type: "DepositV2".to_string() })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let deposit_amount_uint256 = get_string_field(decoded_data, "deposit_amount_uint256")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO deposits (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount_uint256) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, deposit_amount_uint256
    ));

    Ok(sql)
}

fn generate_withdraw_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData { event_type: "WithdrawV2".to_string() })?;

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
        .ok_or(InsertError::MissingDecodedData { event_type: "AddOrderV3".to_string() })?;
    let order = decoded_data
        .get("order")
        .ok_or(InsertError::MissingEventField { field: "order".to_string(), event_type: "AddOrderV3".to_string() })?;

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

    sql.push_str(&generate_order_ios_sql(order, &transaction_hash, log_index)?);

    Ok(sql)
}

fn generate_remove_order_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData { event_type: "RemoveOrderV3".to_string() })?;
    let order = decoded_data
        .get("order")
        .ok_or(InsertError::MissingEventField { field: "order".to_string(), event_type: "RemoveOrderV3".to_string() })?;

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

    sql.push_str(&generate_order_ios_sql(order, &transaction_hash, log_index)?);

    Ok(sql)
}

fn generate_take_order_sql(event: &Value) -> Result<String, InsertError> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or(InsertError::MissingDecodedData { event_type: "TakeOrderV3".to_string() })?;
    let config = decoded_data
        .get("config")
        .ok_or(InsertError::MissingEventField { field: "config".to_string(), event_type: "TakeOrderV3".to_string() })?;
    let order = config
        .get("order")
        .ok_or(InsertError::MissingEventField { field: "order".to_string(), event_type: "TakeOrderV3 config".to_string() })?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;

    let order_owner = get_string_field(order, "owner")?;
    let order_nonce = get_string_field(order, "nonce")?;

    let input_io_index = hex_to_decimal(get_string_field(config, "input_io_index")?)?;
    let output_io_index = hex_to_decimal(get_string_field(config, "output_io_index")?)?;
    let input_amount = get_string_field(decoded_data, "input")?;
    let output_amount = get_string_field(decoded_data, "output")?;

    let mut sql = String::new();

    sql.push_str(&format!(
        "INSERT INTO take_orders (block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, input, output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}');\n",
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
        .ok_or(InsertError::MissingDecodedData { event_type: "ClearV3".to_string() })?;

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
        .ok_or(InsertError::MissingDecodedData { event_type: "AfterClearV2".to_string() })?;

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
        .ok_or(InsertError::MissingDecodedData { event_type: "MetaV1_2".to_string() })?;

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

fn get_string_field<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, InsertError> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| InsertError::MissingField { field: field.to_string() })
}

fn hex_to_decimal(hex_str: &str) -> Result<u64, InsertError> {
    let hex_str_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(hex_str_clean, 16)
        .map_err(|_| InsertError::HexParseError { hex_str: hex_str.to_string() })
}
