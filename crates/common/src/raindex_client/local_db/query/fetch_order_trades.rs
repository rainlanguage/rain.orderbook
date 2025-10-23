use crate::local_db::query::fetch_order_trades::{
    build_fetch_order_trades_query, LocalDbOrderTrade,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn fetch_order_trades<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    order_hash: &str,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
    let sql = build_fetch_order_trades_query(chain_id, order_hash, start_timestamp, end_timestamp);
    exec.query_json(&sql).await
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
    async fn wrapper_uses_builder_sql_exactly() {
        let chain_id = 111;
        let order_hash = " 0xAbc'D  ";
        let start = Some(100);
        let end = Some(200);

        let expected_sql = build_fetch_order_trades_query(chain_id, order_hash, start, end);

        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_order_trades(&exec, chain_id, order_hash, start, end).await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }
}
