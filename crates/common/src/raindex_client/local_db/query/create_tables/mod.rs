use super::*;

const QUERY: &str = include_str!("query.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "sync_status",
    "deposits",
    "withdrawals",
    "order_events",
    "order_ios",
    "take_orders",
    "take_order_contexts",
    "context_values",
    "clear_v3_events",
    "after_clear_v2_events",
    "meta_events",
];

impl LocalDbQuery {
    pub async fn create_tables(db_callback: &js_sys::Function) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_with_callback::<()>(db_callback, QUERY).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
        use super::*;
        use std::rc::Rc;
        use std::cell::RefCell;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        async fn test_create_tables() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("null", captured_sql.clone());

            let result = LocalDbQuery::create_tables(&callback).await;

            assert!(result.is_ok());
            
            let sql = captured_sql.borrow();
            assert!(sql.contains("CREATE TABLE"));
            assert!(sql.contains("BEGIN TRANSACTION"));
            assert!(sql.contains("COMMIT"));
            
            for table_name in REQUIRED_TABLES {
                assert!(sql.contains(table_name), "SQL should contain table: {}", table_name);
            }
            
            assert!(sql.contains("INSERT OR IGNORE INTO sync_status (id, last_synced_block) VALUES (1, 0)"));
        }
    }
}
