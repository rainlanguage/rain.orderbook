use crate::local_db::query::fetch_order_trades_count::{
    build_fetch_trade_count_query, extract_trade_count, LocalDbTradeCountRow,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn fetch_order_trades_count<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    order_hash: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<u64, LocalDbQueryError> {
    let sql = build_fetch_trade_count_query(order_hash, start_timestamp, end_timestamp);
    let rows: Vec<LocalDbTradeCountRow> = exec.query_json(&sql).await?;
    Ok(extract_trade_count(&rows))
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
    async fn wrapper_uses_builder_sql_and_extracts_count() {
        let order_hash = " 0xAbC ' ";
        let start = Some(10);
        let end = Some(20);

        let expected_sql = build_fetch_trade_count_query(order_hash, start, end);

        // Return one row with count 5
        let response = r#"[{"trade_count":5}]"#;
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback(response, store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_order_trades_count(&exec, order_hash, start, end).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 5);

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_extracts_zero_on_empty_rows() {
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);
        let res = super::fetch_order_trades_count(&exec, "hash", None, None).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);
    }
}
