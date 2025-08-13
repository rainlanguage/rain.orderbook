use anyhow::{Context, Result};
use clap::Args;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct EventsToSql {
    #[arg(
        short,
        long,
        default_value = "decoded_events.json",
        help = "Path to the decoded events JSON file"
    )]
    pub input: PathBuf,

    #[arg(
        short,
        long,
        default_value = "events.sql",
        help = "Path to output SQL file"
    )]
    pub output: PathBuf,
}

impl EventsToSql {
    pub async fn execute(self) -> Result<()> {
        let content = fs::read_to_string(&self.input)
            .with_context(|| format!("Failed to read input file: {:?}", self.input))?;

        let data: Value = serde_json::from_str(&content).context("Failed to parse JSON")?;

        let sql_statements = generate_sql(&data)?;

        fs::write(&self.output, sql_statements)
            .with_context(|| format!("Failed to write output file: {:?}", self.output))?;
        println!("SQL statements written to {:?}", self.output);

        Ok(())
    }
}

fn generate_sql(data: &Value) -> Result<String> {
    let mut sql = String::new();

    // Use SAVEPOINT for nested transaction safety
    sql.push_str("SAVEPOINT events_import;\n\n");

    let events = data
        .get("decoded_events")
        .and_then(|v| v.as_array())
        .context("No decoded_events array found")?;

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
                sql.push_str(&generate_clear_sql(event)?);
            }
            Some(event_type) => {
                eprintln!("Warning: Unknown event type: {}", event_type);
            }
            None => {
                eprintln!("Warning: Event missing event_type field");
            }
        }
    }

    sql.push_str("RELEASE SAVEPOINT events_import;\n");

    Ok(sql)
}

fn generate_deposit_sql(event: &Value) -> Result<String> {
    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in Deposit event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let amount = get_string_field(decoded_data, "amount")?;

    Ok(format!(
        "INSERT OR IGNORE INTO deposits (transaction_hash, block_number, log_index, sender, token, vault_id, amount, timestamp) VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', {});\n",
        transaction_hash, block_number, log_index, sender, token, vault_id, amount, timestamp
    ))
}

fn generate_withdraw_sql(event: &Value) -> Result<String> {
    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in Withdraw event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let token = get_string_field(decoded_data, "token")?;
    let vault_id = get_string_field(decoded_data, "vault_id")?;
    let target_amount = get_string_field(decoded_data, "target_amount")?;
    let amount = get_string_field(decoded_data, "amount")?;

    Ok(format!(
        "INSERT OR IGNORE INTO withdraws (transaction_hash, block_number, log_index, sender, token, vault_id, target_amount, amount, timestamp) VALUES ('{}', {}, {}, '{}', '{}', '{}', '{}', '{}', {});\n",
        transaction_hash, block_number, log_index, sender, token, vault_id, target_amount, amount, timestamp
    ))
}

fn generate_add_order_sql(event: &Value) -> Result<String> {
    let mut sql = String::new();

    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in AddOrderV2 event")?;
    let order = decoded_data
        .get("order")
        .context("Missing order in AddOrderV2 event")?;
    let evaluable = order
        .get("evaluable")
        .context("Missing evaluable in order")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;

    let interpreter = get_string_field(evaluable, "interpreter")?;
    let store = get_string_field(evaluable, "store")?;
    let bytecode = get_string_field(evaluable, "bytecode")?;

    // Insert evaluable
    sql.push_str(&format!(
        "INSERT OR IGNORE INTO evaluables (interpreter, store, bytecode, created_at) VALUES ('{}', '{}', '{}', {});\n",
        interpreter, store, bytecode, timestamp
    ));

    // Insert order details
    sql.push_str(&format!(
        "INSERT OR IGNORE INTO order_details (order_hash, owner, nonce, evaluable_id, created_at) VALUES ('{}', '{}', '{}', (SELECT id FROM evaluables WHERE interpreter = '{}' AND store = '{}' AND bytecode = '{}'), {});\n",
        order_hash, owner, nonce, interpreter, store, bytecode, timestamp
    ));

    // Insert order event
    sql.push_str(&format!(
        "INSERT OR IGNORE INTO orders (transaction_hash, block_number, log_index, event_type, sender, order_hash, timestamp) VALUES ('{}', {}, {}, 'ADD', '{}', '{}', {});\n",
        transaction_hash, block_number, log_index, sender, order_hash, timestamp
    ));

    // Insert IOs
    sql.push_str(&generate_ios_sql(order, order_hash)?);

    Ok(sql)
}

fn generate_remove_order_sql(event: &Value) -> Result<String> {
    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in RemoveOrderV2 event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let order_hash = get_string_field(decoded_data, "order_hash")?;

    Ok(format!(
        "INSERT OR IGNORE INTO orders (transaction_hash, block_number, log_index, event_type, sender, order_hash, timestamp) VALUES ('{}', {}, {}, 'REMOVE', '{}', '{}', {});\n",
        transaction_hash, block_number, log_index, sender, order_hash, timestamp
    ))
}

fn generate_take_order_sql(event: &Value) -> Result<String> {
    let mut sql = String::new();

    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in TakeOrderV2 event")?;
    let config = decoded_data
        .get("config")
        .context("Missing config in TakeOrderV2 event")?;
    let order = config
        .get("order")
        .context("Missing order in TakeOrderV2 config")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;

    // For TakeOrderV2, we need to reference an existing order by owner and nonce
    // Since we don't have the actual order_hash in TakeOrderV2 events, we'll use a placeholder
    // and rely on the foreign key constraint to link to existing orders
    let owner = get_string_field(order, "owner")?;
    let nonce = get_string_field(order, "nonce")?;

    let input_io_index = hex_to_decimal(get_string_field(config, "input_io_index")?)?;
    let output_io_index = hex_to_decimal(get_string_field(config, "output_io_index")?)?;
    let input_amount = get_string_field(decoded_data, "input")?;
    let output_amount = get_string_field(decoded_data, "output")?;

    // Insert take order - use a subquery to find the order_hash by owner and nonce
    sql.push_str(&format!(
        "INSERT OR IGNORE INTO take_orders (transaction_hash, block_number, log_index, sender, order_hash, input_io_index, output_io_index, input_amount, output_amount, timestamp) VALUES ('{}', {}, {}, '{}', (SELECT order_hash FROM order_details WHERE owner = '{}' AND nonce = '{}' LIMIT 1), {}, {}, '{}', '{}', {});\n",
        transaction_hash, block_number, log_index, sender, owner, nonce, input_io_index, output_io_index, input_amount, output_amount, timestamp
    ));

    // Insert signed contexts if present
    if let Some(signed_contexts) = config.get("signed_context").and_then(|v| v.as_array()) {
        for (context_index, context) in signed_contexts.iter().enumerate() {
            let signer = get_string_field(context, "signer")?;
            let signature = get_string_field(context, "signature")?;

            sql.push_str(&format!(
                "INSERT OR IGNORE INTO signed_contexts (take_order_id, signer, signature, context_index) VALUES ((SELECT id FROM take_orders WHERE transaction_hash = '{}' AND log_index = {}), '{}', '{}', {});\n",
                transaction_hash, log_index, signer, signature, context_index
            ));

            // Insert context values
            if let Some(context_values) = context.get("context").and_then(|v| v.as_array()) {
                for (value_index, value) in context_values.iter().enumerate() {
                    let context_value = value.as_str().context("Context value must be string")?;
                    sql.push_str(&format!(
                        "INSERT OR IGNORE INTO signed_context_values (signed_context_id, value, value_index) VALUES ((SELECT id FROM signed_contexts WHERE take_order_id = (SELECT id FROM take_orders WHERE transaction_hash = '{}' AND log_index = {}) AND context_index = {}), '{}', {});\n",
                        transaction_hash, log_index, context_index, context_value, value_index
                    ));
                }
            }
        }
    }

    Ok(sql)
}

fn generate_clear_sql(event: &Value) -> Result<String> {
    let mut sql = String::new();

    let decoded_data = event
        .get("decoded_data")
        .context("Missing decoded_data in ClearV2 event")?;
    let clear_config = decoded_data
        .get("clear_config")
        .context("Missing clear_config in ClearV2 event")?;

    let transaction_hash = get_string_field(event, "transaction_hash")?;
    let block_number = hex_to_decimal(get_string_field(event, "block_number")?)?;
    let log_index = hex_to_decimal(get_string_field(event, "log_index")?)?;
    let timestamp = hex_to_decimal(get_string_field(event, "block_timestamp")?)?;

    let sender = get_string_field(decoded_data, "sender")?;
    let alice_order_hash = get_string_field(decoded_data, "alice_order_hash")?;
    let bob_order_hash = get_string_field(decoded_data, "bob_order_hash")?;

    let alice_input_io_index = clear_config
        .get("alice_input_io_index")
        .and_then(|v| v.as_u64())
        .context("Missing alice_input_io_index")?;
    let alice_output_io_index = clear_config
        .get("alice_output_io_index")
        .and_then(|v| v.as_u64())
        .context("Missing alice_output_io_index")?;
    let bob_input_io_index = clear_config
        .get("bob_input_io_index")
        .and_then(|v| v.as_u64())
        .context("Missing bob_input_io_index")?;
    let bob_output_io_index = clear_config
        .get("bob_output_io_index")
        .and_then(|v| v.as_u64())
        .context("Missing bob_output_io_index")?;
    let alice_bounty_vault_id = get_string_field(clear_config, "alice_bounty_vault_id")?;
    let bob_bounty_vault_id = get_string_field(clear_config, "bob_bounty_vault_id")?;

    // Insert clear config
    sql.push_str(&format!(
        "INSERT INTO clear_configs (alice_input_io_index, alice_output_io_index, bob_input_io_index, bob_output_io_index, alice_bounty_vault_id, bob_bounty_vault_id) VALUES ({}, {}, {}, {}, '{}', '{}');\n",
        alice_input_io_index, alice_output_io_index, bob_input_io_index, bob_output_io_index, alice_bounty_vault_id, bob_bounty_vault_id
    ));

    // Insert clear event (get the ID of the config we just inserted)
    sql.push_str(&format!(
        "INSERT OR IGNORE INTO clears (transaction_hash, block_number, log_index, sender, alice_order_hash, bob_order_hash, clear_config_id, timestamp) VALUES ('{}', {}, {}, '{}', '{}', '{}', (SELECT id FROM clear_configs WHERE alice_input_io_index = {} AND alice_output_io_index = {} AND bob_input_io_index = {} AND bob_output_io_index = {} AND alice_bounty_vault_id = '{}' AND bob_bounty_vault_id = '{}'), {});\n",
        transaction_hash, block_number, log_index, sender, alice_order_hash, bob_order_hash, alice_input_io_index, alice_output_io_index, bob_input_io_index, bob_output_io_index, alice_bounty_vault_id, bob_bounty_vault_id, timestamp
    ));

    Ok(sql)
}

fn generate_ios_sql(order: &Value, order_hash: &str) -> Result<String> {
    let mut sql = String::new();

    // Process valid_inputs
    if let Some(inputs) = order.get("valid_inputs").and_then(|v| v.as_array()) {
        for (index, input) in inputs.iter().enumerate() {
            let token = get_string_field(input, "token")?;
            let decimals = input
                .get("decimals")
                .and_then(|v| v.as_u64())
                .context("Missing decimals in input IO")?;
            let vault_id = get_string_field(input, "vault_id")?;

            // Insert IO
            sql.push_str(&format!(
                "INSERT OR IGNORE INTO ios (token, decimals, vault_id) VALUES ('{}', {}, '{}');\n",
                token, decimals, vault_id
            ));

            // Link to order
            sql.push_str(&format!(
                "INSERT OR IGNORE INTO order_ios (order_hash, io_id, io_type, io_index) VALUES ('{}', (SELECT id FROM ios WHERE token = '{}' AND decimals = {} AND vault_id = '{}'), 'INPUT', {});\n",
                order_hash, token, decimals, vault_id, index
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
                .context("Missing decimals in output IO")?;
            let vault_id = get_string_field(output, "vault_id")?;

            // Insert IO
            sql.push_str(&format!(
                "INSERT OR IGNORE INTO ios (token, decimals, vault_id) VALUES ('{}', {}, '{}');\n",
                token, decimals, vault_id
            ));

            // Link to order
            sql.push_str(&format!(
                "INSERT OR IGNORE INTO order_ios (order_hash, io_id, io_type, io_index) VALUES ('{}', (SELECT id FROM ios WHERE token = '{}' AND decimals = {} AND vault_id = '{}'), 'OUTPUT', {});\n",
                order_hash, token, decimals, vault_id, index
            ));
        }
    }

    Ok(sql)
}

fn get_string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .with_context(|| format!("Missing or invalid {} field", field))
}

fn hex_to_decimal(hex_str: &str) -> Result<u64> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(hex_str, 16)
        .with_context(|| format!("Failed to parse hex string: {}", hex_str))
}
