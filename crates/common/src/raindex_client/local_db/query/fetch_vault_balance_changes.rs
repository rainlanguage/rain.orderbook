use crate::local_db::query::fetch_vault_balance_changes::{
    build_fetch_balance_changes_stmt, LocalDbVaultBalanceChange,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::OrderbookIdentifier;
use alloy::primitives::{Address, U256};

pub async fn fetch_vault_balance_changes<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    ob_id: &OrderbookIdentifier,
    vault_id: U256,
    token: Address,
    owner: Address,
) -> Result<Vec<LocalDbVaultBalanceChange>, LocalDbQueryError> {
    let stmt = build_fetch_balance_changes_stmt(ob_id, vault_id, token, owner);
    exec.query_json(&stmt).await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use alloy::primitives::{address, Address, U256};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let vault_id = U256::from(1);
        let token = address!("0x00000000000000000000000000000000000000aa");
        let orderbook = Address::from([0x51; 20]);
        let owner = address!("0x00000000000000000000000000000000000000f1");
        let expected_stmt = build_fetch_balance_changes_stmt(
            &OrderbookIdentifier::new(1, orderbook),
            vault_id,
            token,
            owner,
        );

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_vault_balance_changes(
            &exec,
            &OrderbookIdentifier::new(1, orderbook),
            vault_id,
            token,
            owner,
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn wrapper_returns_rows_when_present() {
        let vault_id = U256::from(1);
        let token = address!("0x00000000000000000000000000000000000000bb");
        let orderbook = Address::from([0x61; 20]);
        let owner = address!("0x0000000000000000000000000000000000000011");
        let expected_stmt = build_fetch_balance_changes_stmt(
            &OrderbookIdentifier::new(1, orderbook),
            vault_id,
            token,
            owner,
        );

        let row_json = r#"[{
            "transactionHash":"0x0000000000000000000000000000000000000abc",
            "logIndex":1,
            "blockNumber":100,
            "blockTimestamp":999,
            "owner":"0x0000000000000000000000000000000000000011",
            "changeType":"deposit",
            "token":"0x00000000000000000000000000000000000000bb",
            "vaultId":"0x01",
            "delta":"0x01",
            "runningBalance":"0x01"
        }]"#;

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback(row_json, store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_vault_balance_changes(
            &exec,
            &OrderbookIdentifier::new(1, orderbook),
            vault_id,
            token,
            owner,
        )
        .await;
        assert!(res.is_ok());
        let rows = res.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(store.borrow().clone().0, expected_stmt.sql);
    }
}
