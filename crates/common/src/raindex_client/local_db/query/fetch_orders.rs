use crate::local_db::query::fetch_orders::{
    build_fetch_orders_stmt, FetchOrdersActiveFilter, FetchOrdersArgs, LocalDbOrder,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::raindex_client::orders::GetOrdersFilters;

impl From<GetOrdersFilters> for FetchOrdersArgs {
    fn from(filters: GetOrdersFilters) -> Self {
        let filter = match filters.active {
            Some(true) => FetchOrdersActiveFilter::Active,
            Some(false) => FetchOrdersActiveFilter::Inactive,
            None => FetchOrdersActiveFilter::All,
        };

        let owners = filters
            .owners
            .into_iter()
            .map(|owner| owner.to_string().to_lowercase())
            .collect();

        let order_hash = filters.order_hash.map(|hash| hash.to_string());

        let tokens = filters
            .tokens
            .unwrap_or_default()
            .into_iter()
            .map(|token| token.to_string().to_lowercase())
            .collect();

        FetchOrdersArgs {
            filter,
            owners,
            order_hash,
            tokens,
        }
    }
}

pub async fn fetch_orders<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    args: FetchOrdersArgs,
) -> Result<Vec<LocalDbOrder>, LocalDbQueryError> {
    let stmt = build_fetch_orders_stmt(&args)?;
    exec.query_json(&stmt).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, Bytes};
    use std::str::FromStr;

    #[test]
    fn from_get_orders_filters_builds_args() {
        let owner = Address::from_str("0x0123456789ABCDEF0123456789ABCDEF01234567").unwrap();
        let token = Address::from_str("0x89ABCDEF0123456789ABCDEF0123456789ABCDEF").unwrap();
        let filters = GetOrdersFilters {
            owners: vec![owner],
            active: Some(true),
            order_hash: Some(Bytes::from_str("0xdeadbeef").unwrap()),
            tokens: Some(vec![token]),
        };
        let args: FetchOrdersArgs = filters.into();
        // Active mapping
        assert!(matches!(args.filter, FetchOrdersActiveFilter::Active));
        // Owners lowered
        assert_eq!(
            args.owners,
            vec!["0x0123456789abcdef0123456789abcdef01234567".to_string()]
        );
        // Tokens lowered
        assert_eq!(
            args.tokens,
            vec!["0x89abcdef0123456789abcdef0123456789abcdef".to_string()]
        );
        // Order hash string preserved
        assert_eq!(args.order_hash.as_deref(), Some("0xdeadbeef"));
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
        use crate::raindex_client::local_db::executor::JsCallbackExecutor;
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::*;

        #[wasm_bindgen_test]
        async fn wrapper_uses_builder_sql_exactly() {
            // Arrange args with mixed case and whitespace to exercise builder sanitization
            let mut args = FetchOrdersArgs::default();
            args.filter = FetchOrdersActiveFilter::Active;
            args.owners = vec![" 0xAbC ".into(), "O'Owner".into()];
            args.tokens = vec![" Tok'A ".into()];
            args.order_hash = Some(" 0xHash ' ".into());

            let expected_stmt = build_fetch_orders_stmt(&args).unwrap();

            let store = Rc::new(RefCell::new((
                String::new(),
                wasm_bindgen::JsValue::UNDEFINED,
            )));
            let callback = create_sql_capturing_callback("[]", store.clone());
            let exec = JsCallbackExecutor::new(&callback);

            // Act
            let res = super::fetch_orders(&exec, args).await;

            // Assert
            assert!(res.is_ok());
            let (captured_sql, _captured_params) = store.borrow().clone();
            assert_eq!(captured_sql, expected_stmt.sql);
        }

        #[wasm_bindgen_test]
        async fn error_propagates_from_callback() {
            // Callback that returns WasmEncodedResult::Err
            let store = Rc::new(RefCell::new((
                String::new(),
                wasm_bindgen::JsValue::UNDEFINED,
            )));
            let store_clone = store.clone();
            let closure = Closure::wrap(Box::new(
                move |sql: String, params: wasm_bindgen::JsValue| -> wasm_bindgen::JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    let result: WasmEncodedResult<String> = WasmEncodedResult::Err {
                        value: None,
                        error: WasmEncodedError {
                            msg: "boom".to_string(),
                            readable_msg: "boom readable".to_string(),
                        },
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                },
            )
                as Box<dyn FnMut(String, wasm_bindgen::JsValue) -> wasm_bindgen::JsValue>);
            let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let args = FetchOrdersArgs::default();
            let exec = JsCallbackExecutor::new(&callback);
            let res = super::fetch_orders(&exec, args).await;
            assert!(res.is_err());
            let err = res.err().unwrap();
            let msg = err.to_string();
            assert!(msg.contains("boom readable"));
        }

        #[wasm_bindgen_test]
        async fn invalid_json_yields_deserialization_error() {
            // Build args and expected SQL
            let args = FetchOrdersArgs::default();
            let expected_stmt = build_fetch_orders_stmt(&args).unwrap();

            // Callback returns Success with invalid JSON payload
            let store = Rc::new(RefCell::new((
                String::new(),
                wasm_bindgen::JsValue::UNDEFINED,
            )));
            let callback = create_sql_capturing_callback("not-json", store.clone());

            let exec = JsCallbackExecutor::new(&callback);
            let res = super::fetch_orders(&exec, args).await;
            assert!(matches!(
                res,
                Err(LocalDbQueryError::Deserialization { .. })
            ));

            // Still should have executed with expected SQL
            assert_eq!(store.borrow().clone().0, expected_stmt.sql);
        }

        #[wasm_bindgen_test]
        async fn invalid_response_yields_invalid_response_error() {
            // Return a raw JsValue string instead of WasmEncodedResult
            let store = Rc::new(RefCell::new((
                String::new(),
                wasm_bindgen::JsValue::UNDEFINED,
            )));
            let store_clone = store.clone();
            let closure = Closure::wrap(Box::new(
                move |sql: String, params: wasm_bindgen::JsValue| -> wasm_bindgen::JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    wasm_bindgen::JsValue::from_str("not-a-wrapper")
                },
            )
                as Box<dyn FnMut(String, wasm_bindgen::JsValue) -> wasm_bindgen::JsValue>);
            let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let args = FetchOrdersArgs::default();
            let exec = JsCallbackExecutor::new(&callback);
            let res = super::fetch_orders(&exec, args).await;
            assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
        }
    }
}
