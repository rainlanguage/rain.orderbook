use crate::local_db::query::fetch_store_addresses::{fetch_store_addresses_stmt, StoreAddressRow};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use alloy::primitives::Address;

pub async fn fetch_store_addresses<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    orderbook_address: Address,
) -> Result<Vec<StoreAddressRow>, LocalDbQueryError> {
    exec.query_json(&fetch_store_addresses_stmt(chain_id, orderbook_address))
        .await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use alloy::primitives::Address;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::wasm_bindgen;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let orderbook = Address::from([0x55; 20]);
        let expected_stmt = fetch_store_addresses_stmt(1, orderbook);
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);
        let res = super::fetch_store_addresses(&exec, 1, orderbook).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }
}
