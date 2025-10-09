use super::*;

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: String,
}

impl LocalDbQuery {
    pub async fn fetch_store_addresses(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<StoreAddressRow>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json::<Vec<StoreAddressRow>>(
            db_callback,
            FETCH_STORE_ADDRESSES_SQL,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::tests::create_success_callback;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_fetch_store_addresses() {
            let rows = vec![
                StoreAddressRow {
                    store_address: "0x1111111111111111111111111111111111111111".to_string(),
                },
                StoreAddressRow {
                    store_address: "0x2222222222222222222222222222222222222222".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&rows).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_store_addresses(&callback).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), rows);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_store_addresses_empty() {
            let callback = create_success_callback("[]");
            let result = LocalDbQuery::fetch_store_addresses(&callback).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }
    }
}
