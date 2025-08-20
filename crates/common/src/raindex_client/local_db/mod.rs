use super::*;
use crate::raindex_client::local_db::decode::decode_events;
use crate::raindex_client::local_db::fetch::fetch_events;
use crate::raindex_client::local_db::insert::decoded_events_to_sql;
use crate::raindex_client::local_db::query::execute_query_no_result;

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
    ) -> Result<(), RaindexError> {
        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Starting database sync process"),
        );

        let _ = status_callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str("Fetching events..."),
        );
        let events =
            match fetch_events("0xd2938e7c9fe3597f78832ce780feb61945c377d7", 19033330u64).await {
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
        let sql_commands = match decoded_events_to_sql(decoded_result) {
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
