use super::fetch_vault::LocalDbVault;
use super::*;
use crate::raindex_client::vaults::GetVaultsFilters;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub owners: Vec<String>,
    pub tokens: Vec<String>,
    pub hide_zero_balance: bool,
}

impl From<GetVaultsFilters> for FetchVaultsArgs {
    fn from(filters: GetVaultsFilters) -> Self {
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

impl LocalDbQuery {
    pub async fn fetch_vaults(
        db_callback: &js_sys::Function,
        chain_id: u32,
        args: FetchVaultsArgs,
    ) -> Result<Vec<LocalDbVault>, LocalDbQueryError> {
        let FetchVaultsArgs {
            owners,
            tokens,
            hide_zero_balance,
        } = args;

        let sanitize_literal = |value: &str| value.replace('\'', "''");

        let owner_values: Vec<String> = owners
            .into_iter()
            .filter_map(|owner| {
                let trimmed = owner.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(format!("'{}'", sanitize_literal(trimmed)))
                }
            })
            .collect();
        let filter_owners = if owner_values.is_empty() {
            String::new()
        } else {
            format!("\nAND lower(o.owner) IN ({})\n", owner_values.join(", "))
        };

        let token_values: Vec<String> = tokens
            .into_iter()
            .filter_map(|token| {
                let trimmed = token.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(format!("'{}'", sanitize_literal(trimmed)))
                }
            })
            .collect();
        let filter_tokens = if token_values.is_empty() {
            String::new()
        } else {
            format!("\nAND lower(o.token) IN ({})\n", token_values.join(", "))
        };

        const BALANCE_EXPR: &str = "COALESCE((\n    SELECT FLOAT_SUM(vd.delta)\n    FROM vault_deltas vd\n    WHERE vd.owner    = o.owner\n      AND vd.token    = o.token\n      AND vd.vault_id = o.vault_id\n  ), FLOAT_ZERO_HEX())";

        let filter_hide_zero_balance = if hide_zero_balance {
            format!("\nAND NOT FLOAT_IS_ZERO({expr})\n", expr = BALANCE_EXPR)
        } else {
            String::new()
        };

        let sql = QUERY
            .replace("?chain_id", &chain_id.to_string())
            .replace("?filter_owners", &filter_owners)
            .replace("?filter_tokens", &filter_tokens)
            .replace("?filter_hide_zero_balance", &filter_hide_zero_balance);

        LocalDbQuery::execute_query_json::<Vec<LocalDbVault>>(db_callback, &sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::{
            create_sql_capturing_callback, create_success_callback,
        };
        use alloy::primitives::Address;
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::str::FromStr;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_parses_data() {
            let vaults = vec![
                LocalDbVault {
                    vault_id: "0x01".into(),
                    token: "0xaaa".into(),
                    owner: "0x1111111111111111111111111111111111111111".into(),
                    orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                    token_name: "Token A".into(),
                    token_symbol: "TA".into(),
                    token_decimals: 18,
                    balance: "0x10".into(),
                    input_orders: Some(
                        "0x01:0xabc0000000000000000000000000000000000000000000000000000000000001:1"
                            .into(),
                    ),
                    output_orders: Some(
                        "0x01:0xdef0000000000000000000000000000000000000000000000000000000000002:0"
                            .into(),
                    ),
                },
                LocalDbVault {
                    vault_id: "0x02".into(),
                    token: "0xbbb".into(),
                    owner: "0x2222222222222222222222222222222222222222".into(),
                    orderbook_address: "0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB".into(),
                    token_name: "Token B".into(),
                    token_symbol: "TB".into(),
                    token_decimals: 6,
                    balance: "0x0".into(),
                    input_orders: None,
                    output_orders: None,
                },
            ];
            let json_data = serde_json::to_string(&vaults).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_vaults(&callback, 1, FetchVaultsArgs::default()).await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].vault_id, vaults[0].vault_id);
            assert_eq!(data[0].token, vaults[0].token);
            assert_eq!(data[0].owner, vaults[0].owner);
            assert_eq!(data[0].orderbook_address, vaults[0].orderbook_address);
            assert_eq!(data[0].token_name, vaults[0].token_name);
            assert_eq!(data[0].token_symbol, vaults[0].token_symbol);
            assert_eq!(data[0].token_decimals, vaults[0].token_decimals);
            assert_eq!(data[0].balance, vaults[0].balance);
            assert_eq!(data[0].input_orders, vaults[0].input_orders);
            assert_eq!(data[0].output_orders, vaults[0].output_orders);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_empty() {
            let callback = create_success_callback("[]");
            let result = LocalDbQuery::fetch_vaults(&callback, 1, FetchVaultsArgs::default()).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_vaults_applies_filters_in_sql() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());

            let filters = GetVaultsFilters {
                owners: vec![
                    Address::from_str("0x1111111111111111111111111111111111111111").unwrap(),
                ],
                hide_zero_balance: true,
                tokens: Some(vec![Address::from_str(
                    "0x2222222222222222222222222222222222222222",
                )
                .unwrap()]),
            };

            let args = FetchVaultsArgs::from(filters);
            let _ = LocalDbQuery::fetch_vaults(&callback, 1, args).await;

            let sql = captured_sql.borrow();
            assert!(
                sql.contains("lower(o.owner) IN ('0x1111111111111111111111111111111111111111')")
            );
            assert!(
                sql.contains("lower(o.token) IN ('0x2222222222222222222222222222222222222222')")
            );
            assert!(sql.contains("AND NOT FLOAT_IS_ZERO("));
        }
    }
}
