use super::*;
use crate::local_db::query::fetch_order_trades_count::{
    build_fetch_trade_count_query, extract_trade_count, LocalDbTradeCountRow,
};

impl LocalDbQuery {
    pub async fn fetch_order_trades_count(
        db_callback: &js_sys::Function,
        order_hash: &str,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<u64, LocalDbQueryError> {
        let sql = build_fetch_trade_count_query(order_hash, start_timestamp, end_timestamp);
        let rows: Vec<LocalDbTradeCountRow> =
            LocalDbQuery::execute_query_json(db_callback, &sql).await?;
        Ok(extract_trade_count(&rows))
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
    async fn wrapper_uses_builder_sql_and_extracts_count() {
        let order_hash = " 0xAbC ' ";
        let start = Some(10);
        let end = Some(20);

        let expected_sql = build_fetch_trade_count_query(order_hash, start, end);

        // Return one row with count 5
        let response = r#"[{"trade_count":5}]"#;
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback(response, store.clone());

        let res = LocalDbQuery::fetch_order_trades_count(&callback, order_hash, start, end).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 5);

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_extracts_zero_on_empty_rows() {
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let res = LocalDbQuery::fetch_order_trades_count(&callback, "hash", None, None).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);
    }
}
