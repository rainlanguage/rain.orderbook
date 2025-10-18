use super::*;
use crate::local_db::query::fetch_store_addresses::{fetch_store_addresses_sql, StoreAddressRow};

impl LocalDbQuery {
    pub async fn fetch_store_addresses(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<StoreAddressRow>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json(db_callback, fetch_store_addresses_sql()).await
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
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = fetch_store_addresses_sql();
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let res = LocalDbQuery::fetch_store_addresses(&callback).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone(), expected_sql);
    }
}
