use super::*;
use crate::local_db::query::update_last_synced_block::build_update_last_synced_block_query;

impl LocalDbQuery {
    pub async fn update_last_synced_block(
        db_callback: &js_sys::Function,
        block_number: u64,
    ) -> Result<(), LocalDbQueryError> {
        let sql = build_update_last_synced_block_query(block_number);
        LocalDbQuery::execute_query_text(db_callback, &sql)
            .await
            .map(|_| ())
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let expected_sql = build_update_last_synced_block_query(999);
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("OK", store.clone());

        let res = LocalDbQuery::update_last_synced_block(&callback, 999).await;
        assert!(res.is_ok());
        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }
}
