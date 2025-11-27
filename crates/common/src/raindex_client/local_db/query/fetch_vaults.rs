use crate::local_db::query::fetch_vault::LocalDbVault;
use crate::local_db::query::fetch_vaults::{build_fetch_vaults_stmt, FetchVaultsArgs};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::OrderbookIdentifier;
use crate::raindex_client::vaults::GetVaultsFilters;

impl FetchVaultsArgs {
    pub fn from_filters(filters: GetVaultsFilters) -> Self {
        FetchVaultsArgs {
            owners: filters.owners,
            tokens: filters.tokens.unwrap_or_default(),
            hide_zero_balance: filters.hide_zero_balance,
        }
    }
}

impl From<GetVaultsFilters> for FetchVaultsArgs {
    fn from(filters: GetVaultsFilters) -> Self {
        FetchVaultsArgs::from_filters(filters)
    }
}

pub async fn fetch_vaults<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    ob_id: &OrderbookIdentifier,
    args: FetchVaultsArgs,
) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
    let stmt = build_fetch_vaults_stmt(ob_id, &args)?;
    exec.query_json(&stmt).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn from_filters_builds_args() {
        let owner = address!("0x0123456789ABCDEF0123456789ABCDEF01234567");
        let token = address!("0x89ABCDEF0123456789ABCDEF0123456789ABCDEF");
        let filters = GetVaultsFilters {
            owners: vec![owner],
            hide_zero_balance: true,
            tokens: Some(vec![token]),
        };
        let args = FetchVaultsArgs::from_filters(filters);
        // Owners lowered
        assert_eq!(
            args.owners,
            vec![address!("0x0123456789abcdef0123456789abcdef01234567")]
        );
        // Tokens lowered
        assert_eq!(
            args.tokens,
            vec![address!("0x89abcdef0123456789abcdef0123456789abcdef")]
        );
        // Hide zero balance
        assert!(args.hide_zero_balance);
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
        use crate::raindex_client::local_db::executor::JsCallbackExecutor;
        use alloy::primitives::{address, Address};
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::wasm_bindgen;

        #[wasm_bindgen_test]
        async fn wrapper_uses_builder_sql_exactly() {
            let mut args = FetchVaultsArgs::default();
            args.owners = vec![
                address!("0x0000000000000000000000000000000000000abc"),
                address!("0x00000000000000000000000000000000000000ef"),
            ];
            args.tokens = vec![address!("0x00000000000000000000000000000000000000aa")];
            args.hide_zero_balance = true;

            let orderbook = Address::from([0x44; 20]);
            let expected_stmt =
                build_fetch_vaults_stmt(&OrderbookIdentifier::new(137, orderbook), &args).unwrap();

            let store = Rc::new(RefCell::new((
                String::new(),
                wasm_bindgen::JsValue::UNDEFINED,
            )));
            let callback = create_sql_capturing_callback("[]", store.clone());
            let exec = JsCallbackExecutor::from_ref(&callback);

            let res =
                super::fetch_vaults(&exec, &OrderbookIdentifier::new(137, orderbook), args).await;
            assert!(res.is_ok());

            let captured = store.borrow().clone();
            assert_eq!(captured.0, expected_stmt.sql);
        }
    }
}
