use super::*;
use crate::raindex_client::local_db::decode::decode_events;
use crate::raindex_client::local_db::fetch::fetch_events;
use crate::raindex_client::local_db::insert::decoded_events_to_sql;
use web_sys::console;

pub mod decode;
pub mod fetch;
pub mod insert;
pub mod query;

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "syncDatabase", unchecked_return_type = "void")]
    pub async fn sync_database(
        #[wasm_export(param_description = "JavaScript function called with the SQL commands")]
        callback: js_sys::Function,
    ) -> Result<(), RaindexError> {
        console::log_1(&"Starting database sync process".into());

        console::log_1(&"Fetching events...".into());
        let events = match fetch_events().await {
            Ok(result) => result,
            Err(e) => {
                console::log_1(&format!("Fetch error: {}", e).into());
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };
        console::log_2(
            &"Fetched events:".into(),
            &format!("{} events", events.as_array().map_or(0, |arr| arr.len())).into(),
        );

        // Decode the events
        console::log_1(&"Decoding events...".into());
        let decoded_result = match decode_events(events) {
            Ok(result) => result,
            Err(e) => {
                console::log_1(&format!("Decode error: {}", e).into());
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };
        console::log_1(&"Events decoded successfully".into());

        // Generate SQL from decoded events
        console::log_1(&"Generating SQL commands...".into());
        let sql_commands = match decoded_events_to_sql(decoded_result) {
            Ok(result) => result,
            Err(e) => {
                console::log_1(&format!("SQL generation error: {}", e).into());
                return Err(RaindexError::CustomError(e.to_string()));
            }
        };
        console::log_2(
            &"Generated SQL commands:".into(),
            &format!("{} characters", sql_commands.len()).into(),
        );

        // Call the callback with the SQL commands
        console::log_1(&"Calling callback with SQL commands".into());
        let _ = callback.call1(
            &wasm_bindgen::JsValue::NULL,
            &wasm_bindgen::JsValue::from_str(&sql_commands),
        );

        console::log_1(&"Database sync process completed successfully".into());
        Ok(())
    }
}
