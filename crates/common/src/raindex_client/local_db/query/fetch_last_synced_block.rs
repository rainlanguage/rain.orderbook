use std::str::FromStr;

use crate::local_db::query::fetch_last_synced_block::{
    fetch_last_synced_block_stmt, SyncStatusResponse,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::{LocalDb, LocalDbError};
use crate::raindex_client::local_db::executor::JsCallbackExecutor;
use alloy::primitives::Address;
use wasm_bindgen_utils::{prelude::*, wasm_export};

pub async fn fetch_last_synced_block<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    orderbook_address: Address,
) -> Result<Vec<SyncStatusResponse>, LocalDbQueryError> {
    exec.query_json(&fetch_last_synced_block_stmt(chain_id, orderbook_address))
        .await
}

#[wasm_export]
impl LocalDb {
    /// Returns the sync status rows from the local DB (via provided JS query callback)
    #[wasm_export(
        js_name = "getSyncStatus",
        unchecked_return_type = "SyncStatusResponse[]"
    )]
    pub async fn get_sync_status(
        &self,
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
        #[wasm_export(js_name = "chainId")] chain_id: u32,
        #[wasm_export(js_name = "orderbookAddress", unchecked_param_type = "Address")]
        orderbook_address: String,
    ) -> Result<Vec<SyncStatusResponse>, LocalDbError> {
        let exec = JsCallbackExecutor::from_ref(&db_callback);
        fetch_last_synced_block(&exec, chain_id, Address::from_str(&orderbook_address)?)
            .await
            .map_err(LocalDbError::from)
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_stmt = fetch_last_synced_block_stmt(0, Address::ZERO);
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);
        let res = super::fetch_last_synced_block(&exec, 0, Address::ZERO).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }
}
