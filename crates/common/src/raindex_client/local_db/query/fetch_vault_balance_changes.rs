use crate::local_db::query::fetch_vault_balance_changes::{
    build_fetch_balance_changes_stmt, LocalDbVaultBalanceChange,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn fetch_vault_balance_changes<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    vault_id: &str,
    token: &str,
) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
    let stmt = build_fetch_balance_changes_stmt(vault_id, token);
    exec.query_json(&stmt).await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let vault_id = "  V01'  ";
        let token = "  0xTo'ken  ";
        let expected_stmt = build_fetch_balance_changes_stmt(vault_id, token);

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_vault_balance_changes(&exec, vault_id, token).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_returns_rows_when_present() {
        let vault_id = "v01";
        let token = "0xtoken";
        let expected_stmt = build_fetch_balance_changes_stmt(vault_id, token);

        let row_json = r#"[{
            "transaction_hash":"0xabc",
            "log_index":1,
            "block_number":100,
            "block_timestamp":999,
            "owner":"0xowner",
            "change_type":"deposit",
            "token":"0xtoken",
            "vault_id":"v01",
            "delta":"0x1",
            "running_balance":"0x1"
        }]"#;

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback(row_json, store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_vault_balance_changes(&exec, vault_id, token).await;
        assert!(res.is_ok());
        let rows = res.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }
}
