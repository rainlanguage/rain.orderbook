use super::*;
use crate::local_db::query::fetch_vault::{
    build_fetch_vault_query, parse_io_indexed_pairs, LocalDbVault,
};

impl LocalDbQuery {
    pub async fn fetch_vault(
        db_callback: &js_sys::Function,
        chain_id: u32,
        vault_id: &str,
        token: &str,
    ) -> Result<Option<LocalDbVault>, LocalDbQueryError> {
        let sql = build_fetch_vault_query(chain_id, vault_id, token);
        let mut rows: Vec<LocalDbVault> =
            LocalDbQuery::execute_query_json(db_callback, &sql).await?;
        Ok(rows.pop())
    }

    pub fn parse_io_indexed_pairs(io: &Option<String>) -> Vec<(usize, String, String)> {
        parse_io_indexed_pairs(io)
    }

    pub async fn fetch_vaults_for_io_string(
        db_callback: &js_sys::Function,
        chain_id: u32,
        io: &Option<String>,
    ) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
        let ios = parse_io_indexed_pairs(io);
        let mut vaults = Vec::with_capacity(ios.len());
        for (_, vault_id, token) in ios.iter() {
            if let Some(v) =
                LocalDbQuery::fetch_vault(db_callback, chain_id, vault_id, token).await?
            {
                vaults.push(v);
            }
        }
        Ok(vaults)
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::query::tests::create_sql_capturing_callback;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_and_none_on_empty() {
        let chain_id = 100;
        let vault_id = "0x01";
        let token = "0xabc";
        let expected_sql = build_fetch_vault_query(chain_id, vault_id, token);

        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());

        let res = LocalDbQuery::fetch_vault(&callback, chain_id, vault_id, token).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_returns_some_row_when_present() {
        let chain_id = 100;
        let vault_id = "0x01";
        let token = "0xabc";
        let expected_sql = build_fetch_vault_query(chain_id, vault_id, token);

        // Single row JSON for LocalDbVault
        let row_json = r#"[{"vaultId":"v","token":"t","owner":"o","orderbookAddress":"ob","tokenName":"N","tokenSymbol":"S","tokenDecimals":18,"balance":"0x0","inputOrders":null,"outputOrders":null}]"#;

        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback(row_json, store.clone());

        let res = LocalDbQuery::fetch_vault(&callback, chain_id, vault_id, token).await;
        assert!(res.is_ok());
        let row = res.unwrap();
        assert!(row.is_some());
        assert_eq!(store.borrow().clone(), expected_sql);
    }

    #[wasm_bindgen_test]
    async fn fetch_vaults_for_io_string_none_or_empty() {
        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());

        // None -> no calls, empty vec
        let none: Option<String> = None;
        let res = LocalDbQuery::fetch_vaults_for_io_string(&callback, 1, &none).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());

        // Empty -> also no valid ios, empty vec
        let empty = Some(String::new());
        let res = LocalDbQuery::fetch_vaults_for_io_string(&callback, 1, &empty).await;
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
        let calls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
        let calls_clone = calls.clone();
        let closure = Closure::wrap(Box::new(move |sql: String| -> wasm_bindgen::JsValue {
            calls_clone.borrow_mut().push(sql);
            let result = wasm_bindgen_utils::result::WasmEncodedResult::Success::<String> {
                value: row_json.to_string(),
                error: None,
            };
            wasm_bindgen_utils::prelude::serde_wasm_bindgen::to_value(&result).unwrap()
        })
            as Box<dyn FnMut(String) -> wasm_bindgen::JsValue>);
        let callback: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();

        // Act
        let res = LocalDbQuery::fetch_vaults_for_io_string(&callback, chain_id, &io).await;
        assert!(res.is_ok());
        let vaults = res.unwrap();
        assert_eq!(vaults.len(), 2);

        // Assert both SQLs fired in sorted order by io index
        let captured = calls.borrow().clone();
        let expected1 = build_fetch_vault_query(chain_id, "v1", "t1");
        let expected2 = build_fetch_vault_query(chain_id, "v2", "t2");
        assert_eq!(captured, vec![expected1, expected2]);
    }
}
