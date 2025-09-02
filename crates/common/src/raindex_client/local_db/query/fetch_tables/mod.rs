use super::*;

const QUERY: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableResponse {
    pub name: String,
}

impl LocalDbQuery {
    pub async fn fetch_all_tables(
        db_callback: &js_sys::Function,
    ) -> Result<Vec<TableResponse>, LocalDbQueryError> {
        LocalDbQuery::execute_query_json::<Vec<TableResponse>>(db_callback, QUERY).await
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

        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        async fn test_fetch_all_tables() {
            let table_data = vec![
                TableResponse {
                    name: "sync_status".to_string(),
                },
                TableResponse {
                    name: "deposits".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&table_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_all_tables(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "sync_status");
            assert_eq!(data[1].name, "deposits");
        }

        #[wasm_bindgen_test]
        async fn test_fetch_all_tables_empty() {
            let callback = create_success_callback("[]");

            let result = LocalDbQuery::fetch_all_tables(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_fetch_all_tables_single() {
            let table_data = vec![TableResponse {
                name: "sqlite_master".to_string(),
            }];
            let json_data = serde_json::to_string(&table_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_all_tables(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].name, "sqlite_master");
        }

        #[wasm_bindgen_test]
        async fn test_fetch_all_tables_special_names() {
            let table_data = vec![
                TableResponse {
                    name: "table_with_numbers_123".to_string(),
                },
                TableResponse {
                    name: "table_with_underscores".to_string(),
                },
                TableResponse {
                    name: "UPPERCASE_TABLE".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&table_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = LocalDbQuery::fetch_all_tables(&callback).await;

            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 3);
            assert!(data.iter().any(|t| t.name == "table_with_numbers_123"));
            assert!(data.iter().any(|t| t.name == "table_with_underscores"));
            assert!(data.iter().any(|t| t.name == "UPPERCASE_TABLE"));
        }
    }
}
