use super::*;

const QUERY: &str = include_str!("query.sql");

impl LocalDbQuery {
    pub async fn clear_tables(db_callback: &js_sys::Function) -> Result<(), LocalDbQueryError> {
        LocalDbQuery::execute_query_text(db_callback, QUERY)
            .await
            .map(|_| ())
    }
}

#[wasm_export]
impl LocalDb {
    #[wasm_export(js_name = "clearTables", unchecked_return_type = "void")]
    pub async fn clear_tables(&self, db_callback: js_sys::Function) -> Result<(), RaindexError> {
        LocalDbQuery::clear_tables(&db_callback)
            .await
            .map_err(|e| RaindexError::LocalDbError(LocalDbError::LocalDbQueryError(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::create_tables::REQUIRED_TABLES;
        use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_clear_tables_generates_expected_sql() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("null", captured_sql.clone());

            let result = LocalDbQuery::clear_tables(&callback).await;
            assert!(result.is_ok());

            let sql = captured_sql.borrow();
            assert!(sql.contains("BEGIN TRANSACTION"));
            assert!(sql.contains("COMMIT"));
            assert!(sql.contains("VACUUM"));
            assert!(sql.contains("DROP TABLE IF EXISTS"));

            for table_name in REQUIRED_TABLES {
                assert!(
                    sql.contains(table_name),
                    "SQL should contain drop for table: {}",
                    table_name
                );
            }
        }
    }
}
