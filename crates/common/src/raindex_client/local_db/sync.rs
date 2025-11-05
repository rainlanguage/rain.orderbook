use crate::local_db::sync::{sync_database_with_services, StatusSink};
use crate::local_db::LocalDbError;
use crate::raindex_client::RaindexClient;
use wasm_bindgen_utils::prelude::*;
use wasm_bindgen_utils::wasm_export;

use super::executor::JsCallbackExecutor;

struct JsStatusReporter<'a> {
    callback: &'a js_sys::Function,
}

impl<'a> JsStatusReporter<'a> {
    fn new(callback: &'a js_sys::Function) -> Self {
        Self { callback }
    }
}

impl<'a> StatusSink for JsStatusReporter<'a> {
    fn send(&self, message: String) -> Result<(), LocalDbError> {
        send_status_message(self.callback, message)
    }
}

fn send_status_message(
    status_callback: &js_sys::Function,
    message: String,
) -> Result<(), LocalDbError> {
    // Call the JS status callback but intentionally swallow any errors so
    // that status reporting never aborts the sync process.
    let _ = status_callback.call1(
        &wasm_bindgen::JsValue::NULL,
        &wasm_bindgen::JsValue::from_str(&message),
    );
    Ok(())
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "syncLocalDatabase", unchecked_return_type = "void")]
    pub async fn sync_database(
        &self,
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
        #[wasm_export(param_description = "JavaScript function called with status updates")]
        status_callback: js_sys::Function,
        #[wasm_export(param_description = "The blockchain network ID to sync against")]
        chain_id: u32,
    ) -> Result<(), LocalDbError> {
        let db_bridge = JsCallbackExecutor::from_ref(&db_callback);
        let status_bridge = JsStatusReporter::new(&status_callback);
        let orderbooks =
            self.get_orderbooks_by_chain_id(chain_id)
                .map_err(|e| LocalDbError::Config {
                    message: format!("Failed to load orderbook configuration: {e}"),
                })?;

        let Some(orderbook_cfg) = orderbooks.first() else {
            return Err(LocalDbError::Config {
                message: format!("No orderbook configuration found for chain ID {chain_id}"),
            });
        };

        sync_database_with_services(orderbook_cfg, &db_bridge, &status_bridge).await
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::raindex_client::local_db::sync::send_status_message;
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::js_sys;

        #[wasm_bindgen_test]
        fn test_send_status_message_success() {
            let callback = js_sys::Function::new_no_args("return true;");
            let message = "Test status message".to_string();
            let result = send_status_message(&callback, message);
            assert!(result.is_ok());
        }

        #[wasm_bindgen_test]
        fn test_send_status_message_callback_error() {
            let callback = js_sys::Function::new_no_args("throw new Error('Callback failed');");
            let message = "Test status message".to_string();
            let result = send_status_message(&callback, message);
            // Errors from the JS callback are swallowed; should not propagate.
            assert!(result.is_ok());
        }
    }
}
