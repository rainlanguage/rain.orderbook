use super::*;
use crate::raindex_client::local_db::decode::decode_events;
use crate::raindex_client::local_db::fetch::{fetch_events, HyperRpcClient};
use crate::raindex_client::local_db::insert::decoded_events_to_sql;
use crate::raindex_client::local_db::query::{execute_query_no_result, get_last_synced_block};

pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "syncDatabase", unchecked_return_type = "void")]
    pub async fn sync_database(
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
        #[wasm_export(param_description = "JavaScript function called with status updates")]
        status_callback: js_sys::Function,
        #[wasm_export(param_description = "The contract address to sync events for")]
        contract_address: String,
        #[wasm_export(param_description = "The default start block number (used when no sync history exists)")]
        default_start_block: u64,
    ) -> Result<(), RaindexError> {
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Starting database sync process"),
        );

        // Read the last synced block from sync_status table
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Reading sync status..."),
        );
        let last_synced_block = match get_last_synced_block(&db_callback).await {
            Ok(block) => block,
            Err(e) => {
                let _ = status_callback.call1(
                    &wasm_bindgen::JsValue::NULL,
                    &wasm_bindgen::JsValue::from_str(&format!("Failed to read sync status: {}", e)),
                );
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };

        // Get the latest block number
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Getting latest block number..."),
        );
        let client = HyperRpcClient {};
        let latest_block = match client.get_latest_block_number().await {
            Ok(block) => block,
            Err(e) => {
                let _ = status_callback.call1(
                    &wasm_bindgen::JsValue::NULL,
                    &wasm_bindgen::JsValue::from_str(&format!("Failed to get latest block: {}", e)),
                );
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };

        // Determine the start block based on sync status
        let start_block = if last_synced_block == 0 {
            // No sync history, use the default start block
            let _ = status_callback.call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(&format!("No sync history found, starting from block {}", default_start_block)),
            );
            default_start_block
        } else {
            // Resume from the next block after the last synced one
            let resume_block = last_synced_block + 1;
            let _ = status_callback.call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(&format!("Resuming sync from block {} (last synced: {})", resume_block, last_synced_block)),
            );
            resume_block
        };

        // Check if we're already up to date
        if start_block > latest_block {
            let _ = status_callback.call1(
                &wasm_bindgen::JsValue::NULL,
                &wasm_bindgen::JsValue::from_str(&format!("Already up to date: start block {} > latest block {}", start_block, latest_block)),
            );
            return Ok(());
        }

        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(&format!("Fetching events from block {} to {}", start_block, latest_block)),
        );
        let events =
            match fetch_events(&contract_address, start_block, latest_block).await {
                Ok(result) => result,
                Err(e) => {
                    let _ = status_callback.call1(
                        &wasm_bindgen::JsValue::NULL,
                        &wasm_bindgen::JsValue::from_str(&format!("Fetch error: {}", e)),
                    );
                    return Err(RaindexError::CustomError(e.to_string()));
                }
            };
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(&format!(
                "Fetched events: {} events",
                events.as_array().map_or(0, |arr| arr.len())
            )),
        );

        // Decode the events
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Decoding events..."),
        );
        let decoded_result = match decode_events(events) {
            Ok(result) => result,
            Err(e) => {
                let _ = status_callback.call1(
                    &wasm_bindgen::JsValue::NULL,
                    &wasm_bindgen::JsValue::from_str(&format!("Decode error: {}", e)),
                );
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Events decoded successfully"),
        );

        // Generate SQL from decoded events
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Generating SQL commands..."),
        );
        let sql_commands = match decoded_events_to_sql(decoded_result, latest_block) {
            Ok(result) => result,
            Err(e) => {
                let _ = status_callback.call1(
                    &wasm_bindgen::JsValue::NULL,
                    &wasm_bindgen::JsValue::from_str(&format!("SQL generation error: {}", e)),
                );
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(&format!(
                "Generated SQL commands: {} characters",
                sql_commands.len()
            )),
        );

        // Execute the SQL commands locally
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Executing SQL commands locally"),
        );
        execute_query_no_result(&db_callback, &sql_commands).await?;

        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Database sync process completed successfully"),
        );
        Ok(())
    }
}
