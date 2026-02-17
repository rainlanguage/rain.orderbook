use crate::local_db::query::fetch_order_trades::LocalDbOrderTrade;
use crate::local_db::query::fetch_trades_by_tx::build_fetch_trades_by_tx_stmt;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::OrderbookIdentifier;
use alloy::primitives::B256;

pub async fn fetch_trades_by_tx<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    ob_id: &OrderbookIdentifier,
    tx_hash: B256,
) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
    let stmt = build_fetch_trades_by_tx_stmt(ob_id, tx_hash)?;
    exec.query_json(&stmt).await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use alloy::primitives::{b256, Address};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let chain_id = 111;
        let orderbook = Address::from([0x77; 20]);
        let tx_hash =
            b256!("0x000000000000000000000000000000000000000000000000000000000000abcd");

        let expected_stmt = build_fetch_trades_by_tx_stmt(
            &OrderbookIdentifier::new(chain_id, orderbook),
            tx_hash.clone(),
        )
        .unwrap();

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_trades_by_tx(
            &exec,
            &OrderbookIdentifier::new(chain_id, orderbook),
            tx_hash,
        )
        .await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }
}
