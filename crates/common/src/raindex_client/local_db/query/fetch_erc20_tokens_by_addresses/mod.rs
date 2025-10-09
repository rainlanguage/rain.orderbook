use super::*;

pub const FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erc20TokenRow {
    pub chain_id: u32,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl LocalDbQuery {
    pub async fn fetch_erc20_tokens_by_addresses(
        db_callback: &js_sys::Function,
        chain_id: u32,
        addresses: &[String],
    ) -> Result<Vec<Erc20TokenRow>, LocalDbQueryError> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        let in_clause = addresses
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = FETCH_ERC20_TOKENS_BY_ADDRESSES_SQL
            .replace("?chain_id", &chain_id.to_string())
            .replace("?addresses_in", &in_clause);

        LocalDbQuery::execute_query_json::<Vec<Erc20TokenRow>>(db_callback, &sql).await
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
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_empty_addresses_returns_empty() {
            let callback = create_success_callback("[]");
            let result = LocalDbQuery::fetch_erc20_tokens_by_addresses(&callback, 1, &[]).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_builds_sql_with_chain_and_in_clause() {
            let captured_sql = Rc::new(RefCell::new(String::new()));
            let callback = create_sql_capturing_callback("[]", captured_sql.clone());
            let addrs = vec![
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            ];

            let _ = LocalDbQuery::fetch_erc20_tokens_by_addresses(&callback, 137, &addrs).await;
            let sql = captured_sql.borrow();
            assert!(sql.contains("FROM erc20_tokens"));
            assert!(sql.contains("WHERE chain_id = 137"));
            assert!(sql.contains("address IN ('0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb')"));
        }

        #[wasm_bindgen_test]
        async fn test_parses_success_data() {
            let rows = vec![
                Erc20TokenRow {
                    chain_id: 1,
                    address: "0x01".into(),
                    name: "A".into(),
                    symbol: "AA".into(),
                    decimals: 18,
                },
                Erc20TokenRow {
                    chain_id: 1,
                    address: "0x02".into(),
                    name: "Bee".into(),
                    symbol: "B".into(),
                    decimals: 6,
                },
            ];
            let json_data = serde_json::to_string(&rows).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_erc20_tokens_by_addresses(
                &callback,
                1,
                &["0x01".into(), "0x02".into()],
            )
            .await;
            assert!(result.is_ok());
            let out = result.unwrap();
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].name, "A");
            assert_eq!(out[1].symbol, "B");
        }
    }
}
