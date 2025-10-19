use super::*;
use crate::local_db::query::fetch_vault::LocalDbVault;
use crate::local_db::query::fetch_vaults::{build_fetch_vaults_query, FetchVaultsArgs};
use crate::local_db::query::LocalDbQueryExecutor;
use crate::raindex_client::vaults::GetVaultsFilters;

impl FetchVaultsArgs {
    pub fn from_filters(filters: GetVaultsFilters) -> Self {
        let owners = filters
            .owners
            .into_iter()
            .map(|owner| owner.to_string().to_lowercase())
            .collect();
        let tokens = filters
            .tokens
            .unwrap_or_default()
            .into_iter()
            .map(|token| token.to_string().to_lowercase())
            .collect();

        FetchVaultsArgs {
            owners,
            tokens,
            hide_zero_balance: filters.hide_zero_balance,
        }
    }
}

impl From<GetVaultsFilters> for FetchVaultsArgs {
    fn from(filters: GetVaultsFilters) -> Self {
        FetchVaultsArgs::from_filters(filters)
    }
}

impl LocalDbQuery {
    pub async fn fetch_vaults<E: LocalDbQueryExecutor + ?Sized>(
        exec: &E,
        chain_id: u32,
        args: FetchVaultsArgs,
    ) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
        let sql = build_fetch_vaults_query(chain_id, &args);
        exec.query_json(&sql).await
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let mut args = FetchVaultsArgs::default();
        args.owners = vec![" 0xAbC ".into(), "O'Owner".into()];
        args.tokens = vec![" Tok'A ".into()];
        args.hide_zero_balance = true;

        let expected_sql = build_fetch_vaults_query(137, &args);

        let store = Rc::new(RefCell::new(String::new()));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::new(&callback);

        let res = LocalDbQuery::fetch_vaults(&exec, 137, args).await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured, expected_sql);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn from_filters_builds_args() {
        let owner = Address::from_str("0x0123456789ABCDEF0123456789ABCDEF01234567").unwrap();
        let token = Address::from_str("0x89ABCDEF0123456789ABCDEF0123456789ABCDEF").unwrap();
        let filters = GetVaultsFilters {
            owners: vec![owner],
            hide_zero_balance: true,
            tokens: Some(vec![token]),
        };
        let args = FetchVaultsArgs::from_filters(filters);
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
        // Hide zero balance
        assert!(args.hide_zero_balance);
    }
}
