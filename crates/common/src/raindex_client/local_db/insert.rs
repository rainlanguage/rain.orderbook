use serde_json::Value;

pub fn decoded_events_to_sql(data: Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut sql = String::new();

    // Start transaction for all events
    sql.push_str("BEGIN TRANSACTION;\n\n");

    let events = data
        .get("decoded_events")
        .and_then(|v| v.as_array())
        .ok_or("No decoded_events array found")?;

    for event in events {
        match event.get("event_type").and_then(|v| v.as_str()) {
            Some("Deposit") => {
                sql.push_str(&generate_deposit_sql(event)?);
            }
            Some("Withdraw") => {
                sql.push_str(&generate_withdraw_sql(event)?);
            }
            Some("AddOrderV2") => {
                sql.push_str(&generate_add_order_sql(event)?);
            }
            Some("RemoveOrderV2") => {
                sql.push_str(&generate_remove_order_sql(event)?);
            }
            Some("TakeOrderV2") => {
                sql.push_str(&generate_take_order_sql(event)?);
            }
            Some("ClearV2") => {
                sql.push_str(&generate_clear_v2_sql(event)?);
            }
            Some("AfterClear") => {
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

    // Commit the transaction for all events
    sql.push_str("\nCOMMIT;\n");

    Ok(sql)
}

fn generate_deposit_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in Deposit event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let amount = get_string_field(decoded_data, "amount")?;

    let mut sql = String::new();

    // Insert directly into deposits table with metadata
    sql.push_str(&format!(
        "INSERT INTO deposits (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, amount) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, amount
    ));

    Ok(sql)
}

fn generate_withdraw_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in Withdraw event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let target_amount = get_string_field(decoded_data, "target_amount")?;
    let amount = get_string_field(decoded_data, "amount")?;

    let mut sql = String::new();

    // Insert directly into withdrawals table with metadata
    sql.push_str(&format!(
        "INSERT INTO withdrawals (block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, amount) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, token, vault_id, target_amount, amount
    ));

    Ok(sql)
}

fn generate_add_order_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in AddOrderV2 event")?;
    let order = decoded_data
        .get("order")
        .ok_or("Missing order in AddOrderV2 event")?;
    let evaluable = order.get("evaluable").ok_or("Missing evaluable in order")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;

    let interpreter = get_string_field(evaluable, "interpreter")?;
    let store = get_string_field(evaluable, "store")?;
    let bytecode = get_string_field(evaluable, "bytecode")?;

    let mut sql = String::new();

    // Insert directly into order_events table with metadata and full order data
    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash, owner, nonce, interpreter, store, bytecode) VALUES ({}, {}, '{}', {}, 'add', '{}', '{}', '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_hash, owner, nonce, interpreter, store, bytecode
    ));

    // Insert IOs for this order using the specific transaction hash and log index
    let order_event_id_query = &format!(
        "(SELECT id FROM order_events WHERE transaction_hash = '{}' AND log_index = {})",
        transaction_hash, log_index
    );
    sql.push_str(&generate_order_ios_sql(order, order_event_id_query)?);

    Ok(sql)
}

fn generate_remove_order_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in RemoveOrderV2 event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;

    let mut sql = String::new();

    // Insert directly into order_events table with metadata (remove event doesn't have order data)
    sql.push_str(&format!(
        "INSERT INTO order_events (block_number, block_timestamp, transaction_hash, log_index, event_type, sender, order_hash) VALUES ({}, {}, '{}', {}, 'remove', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_hash
    ));

    Ok(sql)
}

fn generate_take_order_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in TakeOrderV2 event")?;
    let config = decoded_data
        .get("config")
        .ok_or("Missing config in TakeOrderV2 event")?;
    let order = config
        .get("order")
        .ok_or("Missing order in TakeOrderV2 config")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;

    // Get order details from the config
    let order_owner = get_string_field(order, "owner")?;
    let order_nonce = get_string_field(order, "nonce")?;

    let input_io_index = hex_to_decimal(get_string_field(config, "input_io_index")?)?;
    let output_io_index = hex_to_decimal(get_string_field(config, "output_io_index")?)?;
    let input_amount = get_string_field(decoded_data, "input")?;
    let output_amount = get_string_field(decoded_data, "output")?;

    let mut sql = String::new();

    // Insert directly into take_orders table with metadata
    sql.push_str(&format!(
        "INSERT INTO take_orders (block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, input_amount, output_amount) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', {}, {}, '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, order_owner, order_nonce, input_io_index, output_io_index, input_amount, output_amount
    ));

    // Insert signed contexts if present - use specific transaction hash and log index to find the take_order
    if let Some(signed_contexts) = config.get("signed_context").and_then(|v| v.as_array()) {
        for (context_index, context) in signed_contexts.iter().enumerate() {
            let signer = get_string_field(context, "signer")?;
            let signature = get_string_field(context, "signature")?;

            // Insert context using the specific transaction hash and log index
            sql.push_str(&format!(
                "INSERT INTO take_order_contexts (take_order_id, context_index, signer, signature) VALUES ((SELECT id FROM take_orders WHERE transaction_hash = '{}' AND log_index = {}), {}, '{}', '{}');\n",
                transaction_hash, log_index, context_index, signer, signature
            ));

            // Insert context values for this specific context
            if let Some(context_values) = context.get("context").and_then(|v| v.as_array()) {
                for (value_index, value) in context_values.iter().enumerate() {
                    let context_value = value.as_str().ok_or("Context value must be string")?;
                    sql.push_str(&format!(
                        "INSERT INTO context_values (context_id, value_index, value) VALUES (last_insert_rowid(), {}, '{}');\n",
                        value_index, context_value
                    ));
                }
            }
        }
    }

    Ok(sql)
}

fn generate_clear_v2_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in ClearV2 event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let alice_owner = get_string_field(decoded_data, "alice_owner")?;
    let bob_owner = get_string_field(decoded_data, "bob_owner")?;
    let alice_order_hash = get_string_field(decoded_data, "alice_order_hash")?;
    let bob_order_hash = get_string_field(decoded_data, "bob_order_hash")?;
    let alice_input_vault_id = get_string_field(decoded_data, "alice_input_vault_id")?;
    let alice_output_vault_id = get_string_field(decoded_data, "alice_output_vault_id")?;
    let bob_input_vault_id = get_string_field(decoded_data, "bob_input_vault_id")?;
    let bob_output_vault_id = get_string_field(decoded_data, "bob_output_vault_id")?;

    let mut sql = String::new();

    // Insert into clear_v2_events table
    sql.push_str(&format!(
        "INSERT INTO clear_v2_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_owner, bob_owner, alice_order_hash, bob_order_hash, alice_input_vault_id, alice_output_vault_id, bob_input_vault_id, bob_output_vault_id) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, alice_owner, bob_owner, alice_order_hash, bob_order_hash, alice_input_vault_id, alice_output_vault_id, bob_input_vault_id, bob_output_vault_id
    ));

    Ok(sql)
}

fn generate_after_clear_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in AfterClear event")?;

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

    // Insert into after_clear_events table
    sql.push_str(&format!(
        "INSERT INTO after_clear_events (block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, alice_input, alice_output, bob_input, bob_output
    ));

    Ok(sql)
}


fn generate_meta_sql(event: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_data = event
        .get("decoded_data")
        .ok_or("Missing decoded_data in MetaV1_2 event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let block_timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let subject = get_string_field(decoded_data, "subject")?;
    let meta = get_string_field(decoded_data, "meta")?;

    let mut sql = String::new();

    // Insert directly into meta_events table with metadata
    sql.push_str(&format!(
        "INSERT INTO meta_events (block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta) VALUES ({}, {}, '{}', {}, '{}', '{}', '{}');\n",
        block_number, block_timestamp, transaction_hash, log_index, sender, subject, meta
    ));

    Ok(sql)
}

fn generate_order_ios_sql(
    order: &Value,
    order_event_id_query: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut sql = String::new();

    // Collect all IOs first
    let mut all_ios = Vec::new();

    // Process valid_inputs
    if let Some(inputs) = order.get("valid_inputs").and_then(|v| v.as_array()) {
        for (index, input) in inputs.iter().enumerate() {
            let token = get_string_field(input, "token")?;
            let decimals = input
                .get("decimals")
                .and_then(|v| v.as_u64())
                .ok_or("Missing decimals in input IO")?;
            let vault_id = get_string_field(input, "vault_id")?;

            all_ios.push(format!(
                "({}, 'input', {}, '{}', {}, '{}')",
                order_event_id_query, index, token, decimals, vault_id
            ));
        }
    }

    // Process valid_outputs
    if let Some(outputs) = order.get("valid_outputs").and_then(|v| v.as_array()) {
        for (index, output) in outputs.iter().enumerate() {
            let token = get_string_field(output, "token")?;
            let decimals = output
                .get("decimals")
                .and_then(|v| v.as_u64())
                .ok_or("Missing decimals in output IO")?;
            let vault_id = get_string_field(output, "vault_id")?;

            all_ios.push(format!(
                "({}, 'output', {}, '{}', {}, '{}')",
                order_event_id_query, index, token, decimals, vault_id
            ));
        }
    }

    // Insert all IOs in a single INSERT statement to avoid multiple evaluations of last_insert_rowid()
    if !all_ios.is_empty() {
        sql.push_str("INSERT INTO order_ios (order_event_id, io_type, io_index, token, decimals, vault_id) VALUES ");
        sql.push_str(&all_ios.join(", "));
        sql.push_str(";\n");
    }

    Ok(sql)
}

fn get_string_field<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing or invalid {} field", field).into())
}

fn hex_to_decimal(hex_str: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(hex_str, 16)
        .map_err(|_| format!("Failed to parse hex string: {}", hex_str).into())
}
