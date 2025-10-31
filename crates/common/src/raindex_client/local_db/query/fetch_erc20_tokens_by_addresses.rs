use crate::local_db::query::fetch_erc20_tokens_by_addresses::{build_fetch_stmt, Erc20TokenRow};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use alloy::primitives::Address;

pub async fn fetch_erc20_tokens_by_addresses<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    orderbook_address: Address,
    addresses: &[Address],
) -> Result<Vec<Erc20TokenRow>, LocalDbQueryError> {
    if let Some(stmt) = build_fetch_stmt(chain_id, orderbook_address, addresses)? {
        exec.query_json(&stmt).await
    } else {
        Ok(vec![])
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::str::FromStr;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn empty_addresses_short_circuits_and_executes_no_sql() {
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);
        let res = super::fetch_erc20_tokens_by_addresses(&exec, 1, Address::ZERO, &[]).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
        assert!(store.borrow().0.is_empty());
    }

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let addrs = vec![
            Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
            Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
        ];
        let expected_stmt = build_fetch_stmt(10, Address::ZERO, &addrs)
            .unwrap()
            .unwrap();

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_erc20_tokens_by_addresses(&exec, 10, Address::ZERO, &addrs).await;
        assert!(res.is_ok());

        let (captured_sql, captured_params) = store.borrow().clone();
        assert_eq!(captured_sql, expected_stmt.sql);

        // Also assert parameters are encoded and passed as expected
        // The executor passes `undefined` when empty, otherwise an array via serde_wasm_bindgen
        if expected_stmt.params().is_empty() {
            assert!(captured_params.is_undefined());
        } else {
            // Encode expected params to JsValue the same way the callback receives them
            let expected_js_params = wasm_bindgen_utils::prelude::serde_wasm_bindgen::to_value(
                &expected_stmt.as_js_params(),
            )
            .expect("encode expected params");

            // Compare as JSON strings to ensure deep structural equality
            let expected_json = js_sys::JSON::stringify(&expected_js_params)
                .unwrap()
                .as_string()
                .unwrap();
            let captured_json = js_sys::JSON::stringify(&captured_params)
                .unwrap()
                .as_string()
                .unwrap();
            assert_eq!(captured_json, expected_json);
        }
    }
}
