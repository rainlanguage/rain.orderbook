use super::*;
use crate::local_db::query::fetch_erc20_tokens_by_addresses::{build_fetch_query, Erc20TokenRow};

impl LocalDbQuery {
    pub async fn fetch_erc20_tokens_by_addresses(
        db_callback: &js_sys::Function,
        chain_id: u32,
        addresses: &[String],
    ) -> Result<Vec<Erc20TokenRow>, LocalDbQueryError> {
        if let Some(sql) = build_fetch_query(chain_id, addresses) {
            LocalDbQuery::execute_query_json(db_callback, &sql).await
        } else {
            Ok(vec![])
        }
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
    async fn empty_addresses_short_circuits_and_executes_no_sql() {
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let res = LocalDbQuery::fetch_erc20_tokens_by_addresses(&callback, 1, &[]).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
        assert!(store.borrow().is_empty());
    }

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let addrs = vec!["0xA".to_string(), "B".to_string()];
        let expected_sql = build_fetch_query(10, &addrs).unwrap();

        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());

        let res = LocalDbQuery::fetch_erc20_tokens_by_addresses(&callback, 10, &addrs).await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }
}
