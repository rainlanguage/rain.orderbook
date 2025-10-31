use crate::local_db::query::fetch_vault::{
    build_fetch_vault_stmt, parse_io_indexed_pairs, LocalDbVault,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use alloy::primitives::Address;

pub async fn fetch_vault<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    orderbook_address: Address,
    vault_id: &str,
    token: &str,
) -> Result<Option<LocalDbVault>, LocalDbQueryError> {
    let stmt = build_fetch_vault_stmt(chain_id, orderbook_address, vault_id, token);
    let rows: Vec<LocalDbVault> = exec.query_json(&stmt).await?;
    Ok(rows.into_iter().next())
}

pub async fn fetch_vaults_for_io_string<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    chain_id: u32,
    orderbook_address: Address,
    io: &Option<String>,
) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
    let ios = parse_io_indexed_pairs(io);
    let mut vaults = Vec::with_capacity(ios.len());
    for (_, vault_id, token) in ios.iter() {
        if let Some(v) = fetch_vault(exec, chain_id, orderbook_address, vault_id, token).await? {
            vaults.push(v);
        }
    }
    Ok(vaults)
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::local_db::query::SqlValue;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::serde_wasm_bindgen::{from_value, to_value};
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_and_none_on_empty() {
        let chain_id = 100;
        let vault_id = "0x01";
        let token = "0xabc";
        let orderbook = Address::from([0x11; 20]);
        let expected_stmt = build_fetch_vault_stmt(chain_id, orderbook, vault_id, token);

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_vault(&exec, chain_id, orderbook, vault_id, token).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());

        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_returns_some_row_when_present() {
        let chain_id = 100;
        let vault_id = "0x01";
        let token = "0xabc";
        let orderbook = Address::from([0x22; 20]);
        let expected_stmt = build_fetch_vault_stmt(chain_id, orderbook, vault_id, token);

        // Single row JSON for LocalDbVault
        let row_json = r#"[{"vaultId":"1","token":"t","owner":"o","orderbookAddress":"ob","tokenName":"N","tokenSymbol":"S","tokenDecimals":18,"balance":"0x0","inputOrders":null,"outputOrders":null}]"#;

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback(row_json, store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = super::fetch_vault(&exec, chain_id, orderbook, vault_id, token).await;
        assert!(res.is_ok());
        let row = res.unwrap();
        assert!(row.is_some());
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn fetch_vaults_for_io_string_none_or_empty() {
        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        // None -> no calls, empty vec
        let none: Option<String> = None;
        let res = super::fetch_vaults_for_io_string(&exec, 1, Address::ZERO, &none).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());

        // Empty -> also no valid ios, empty vec
        let empty = Some(String::new());
        let res = super::fetch_vaults_for_io_string(&exec, 1, Address::ZERO, &empty).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[wasm_bindgen_test]
    async fn fetch_vaults_for_io_string_multiple_calls_and_results() {
        // Build IO string with two valid entries, out of order to test sorting
        let io = Some("2:v2:t2,1:v1:t1".to_string());
        let chain_id = 77u32;

        // JSON for a single LocalDbVault row
        let row_json = r#"[{"vaultId":"1","token":"t","owner":"o","orderbookAddress":"ob","tokenName":"N","tokenSymbol":"S","tokenDecimals":18,"balance":"0x0","inputOrders":null,"outputOrders":null}]"#;

        // Capture all SQL calls
        let calls: Rc<RefCell<Vec<(String, wasm_bindgen::JsValue)>>> =
            Rc::new(RefCell::new(vec![]));
        let calls_clone = calls.clone();
        let closure = Closure::wrap(Box::new(
            move |sql: String, params: wasm_bindgen::JsValue| -> wasm_bindgen::JsValue {
                calls_clone.borrow_mut().push((sql, params.clone()));
                let result = WasmEncodedResult::Success::<String> {
                    value: row_json.to_string(),
                    error: None,
                };
                to_value(&result).unwrap()
            },
        )
            as Box<dyn FnMut(String, wasm_bindgen::JsValue) -> wasm_bindgen::JsValue>);
        let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();

        // Act
        let exec = JsCallbackExecutor::new(&callback);
        let res = super::fetch_vaults_for_io_string(&exec, chain_id, Address::ZERO, &io).await;
        assert!(res.is_ok());
        let vaults = res.unwrap();
        assert_eq!(vaults.len(), 2);

        // Assert both SQLs fired in sorted order by io index
        let captured = calls.borrow().clone();
        let captured: Vec<(String, Vec<SqlValue>)> = captured
            .into_iter()
            .map(|(sql, params)| {
                let params_vec: Vec<SqlValue> =
                    from_value(params).expect("SQL params should deserialize");
                (sql, params_vec)
            })
            .collect();
        let expected1 = build_fetch_vault_stmt(chain_id, Address::ZERO, "v1", "t1");
        let expected2 = build_fetch_vault_stmt(chain_id, Address::ZERO, "v2", "t2");
        let expected = vec![
            (expected1.sql().to_string(), expected1.params().to_vec()),
            (expected2.sql().to_string(), expected2.params().to_vec()),
        ];
        assert_eq!(captured, expected);
    }
}
