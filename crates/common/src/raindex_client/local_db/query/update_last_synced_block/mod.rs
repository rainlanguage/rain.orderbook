use super::*;

const QUERY: &str = include_str!("query.sql");

impl LocalDbQuery {
    pub async fn update_last_synced_block(
        db_callback: &js_sys::Function,
        block_number: u64,
    ) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_text(
            db_callback,
            &QUERY.replace("?block_number", &block_number.to_string()),
        )
        .await
        .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        async fn test_update_last_synced_block() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("null", captured_sql.clone());
            let test_block_number = 54321u64;

            let result = LocalDbQuery::update_last_synced_block(&callback, test_block_number).await;

            assert!(result.is_ok());

            let sql = captured_sql.borrow();
            assert!(sql.contains("UPDATE sync_status"));
            assert!(sql.contains("SET last_synced_block ="));
            assert!(sql.contains("WHERE id = 1"));
            assert!(sql.contains("updated_at = CURRENT_TIMESTAMP"));

            assert!(
                sql.contains(&test_block_number.to_string()),
                "SQL should contain the block number {}: {}",
                test_block_number,
                sql
            );

            assert!(
                !sql.contains("?block_number"),
                "SQL should not contain placeholder ?block_number: {}",
                sql
            );
        }
    }
}
