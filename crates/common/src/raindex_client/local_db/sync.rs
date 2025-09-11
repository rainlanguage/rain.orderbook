use super::{
    query::{create_tables::REQUIRED_TABLES, LocalDbQuery},
    *,
};
use flate2::read::GzDecoder;
use reqwest::Client;
use std::io::Read;

const DUMP_URL: &str = "https://raw.githubusercontent.com/rainlanguage/rain.strategies/46a8065a2ccbf49e9fc509cbc052cd497feb6bd5/local-db-dump.sql.gz";

async fn check_required_tables(db_callback: &js_sys::Function) -> Result<bool, LocalDbError> {
    match LocalDbQuery::fetch_all_tables(db_callback).await {
        Ok(tables) => {
            let existing_table_names: std::collections::HashSet<String> =
                tables.into_iter().map(|t| t.name).collect();

            let has_all_tables = REQUIRED_TABLES
                .iter()
                .all(|&table| existing_table_names.contains(table));

            Ok(has_all_tables)
        }
        Err(_) => Ok(false),
    }
}

async fn download_and_decompress_dump() -> Result<String, LocalDbError> {
    let client = Client::new();
    let response = client.get(DUMP_URL).send().await?;

    if !response.status().is_success() {
        return Err(LocalDbError::CustomError(format!(
            "Failed to download dump, status: {}",
            response.status()
        )));
    }
    let response = response.bytes().await?.to_vec();

    let mut decoder = GzDecoder::new(response.as_slice());
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;

    Ok(decompressed)
}

pub async fn get_last_synced_block(db_callback: &js_sys::Function) -> Result<u64, LocalDbError> {
    match LocalDbQuery::fetch_last_synced_block(db_callback).await {
        Ok(results) => {
            if let Some(sync_status) = results.first() {
                Ok(sync_status.last_synced_block)
            } else {
                Ok(0)
            }
        }
        Err(e) => Err(e.into()),
    }
}

fn send_status_message(
    status_callback: &js_sys::Function,
    message: String,
) -> Result<(), LocalDbError> {
    status_callback
        .call1(&JsValue::NULL, &JsValue::from_str(&message))
        .map_err(|e| LocalDbError::CustomError(format!("JavaScript callback error: {:?}", e)))?;
    Ok(())
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "syncLocalDatabase", unchecked_return_type = "void")]
    pub async fn sync_database(
        &self,
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
        #[wasm_export(param_description = "JavaScript function called with status updates")]
        status_callback: js_sys::Function,
        #[wasm_export(param_description = "The blockchain network ID to sync against")]
        chain_id: u32,
    ) -> Result<(), LocalDbError> {
        send_status_message(&status_callback, "Starting database sync...".to_string())?;

        let has_tables = match check_required_tables(&db_callback).await {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to check tables: {}",
                    e
                )));
            }
        };

        send_status_message(&status_callback, format!("has tables: {}", has_tables))?;

        if !has_tables {
            send_status_message(
                &status_callback,
                "Initializing database tables and importing data...".to_string(),
            )?;
            let dump_sql = download_and_decompress_dump().await?;

            LocalDbQuery::execute_query_text(&db_callback, &dump_sql).await?;
        }

        let last_synced_block = match get_last_synced_block(&db_callback).await {
            Ok(block) => block,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to read sync status: {}",
                    e
                )));
            }
        };
        send_status_message(
            &status_callback,
            format!("Last synced block: {}", last_synced_block),
        )?;

        let orderbooks = match self.get_orderbooks_by_chain_id(chain_id) {
            Ok(o) => o,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to get orderbook configurations: {}",
                    e
                )));
            }
        };

        // TODO: For simplicity, we only handle one orderbook per chain ID here.
        // This will be changed in the future to support multiple orderbooks.
        let orderbook_cfg = match orderbooks.first() {
            Some(cfg) => cfg,
            None => {
                return Err(LocalDbError::CustomError(format!(
                    "No orderbook configuration found for chain ID {}",
                    chain_id
                )));
            }
        };

        let local_db = LocalDb::new_with_regular_rpcs(orderbook_cfg.network.rpcs.clone());

        let latest_block = match local_db
            .rpc_client()
            .get_latest_block_number(local_db.rpc_urls())
            .await
        {
            Ok(block) => block,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to get latest block: {}",
                    e
                )));
            }
        };

        let start_block = if last_synced_block == 0 {
            orderbook_cfg.deployment_block
        } else {
            last_synced_block + 1
        };

        send_status_message(
            &status_callback,
            "Fetching latest onchain events...".to_string(),
        )?;
        let events = match local_db
            .fetch_events(
                &orderbook_cfg.address.to_string(),
                start_block,
                latest_block,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "There was a problem trying to fetch events: {}",
                    e
                )));
            }
        };

        send_status_message(&status_callback, "Decoding fetched events...".to_string())?;
        let decoded_events = match local_db.decode_events(events) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "There was a problem trying to decode events: {}",
                    e
                )));
            }
        };

        send_status_message(&status_callback, "Populating database...".to_string())?;
        let sql_commands = match local_db.decoded_events_to_sql(decoded_events, latest_block) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(e.to_string()));
            }
        };

        LocalDbQuery::execute_query_text(&db_callback, &sql_commands).await?;

        send_status_message(&status_callback, "Database sync complete.".to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::query::{
            create_tables::REQUIRED_TABLES, fetch_last_synced_block::SyncStatusResponse,
            fetch_tables::TableResponse, tests::create_success_callback, LocalDbQueryError,
        };
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen_test::*;

        #[wasm_bindgen_test]
        async fn test_check_required_tables_all_exist() {
            let table_data: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();

            let json_data = serde_json::to_string(&table_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = check_required_tables(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_missing_some() {
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

            let result = check_required_tables(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_empty_db() {
            let callback = create_success_callback("[]");

            let result = check_required_tables(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_query_fails() {
            let callback = create_success_callback("invalid_json");

            let result = check_required_tables(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_extra_tables() {
            let mut table_data: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();

            table_data.push(TableResponse {
                name: "extra_table_1".to_string(),
            });
            table_data.push(TableResponse {
                name: "extra_table_2".to_string(),
            });

            let json_data = serde_json::to_string(&table_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = check_required_tables(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);
        }

        #[wasm_bindgen_test]
        async fn test_get_last_synced_block_exists() {
            let sync_data = vec![SyncStatusResponse {
                id: 1,
                last_synced_block: 12345,
                updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            }];
            let json_data = serde_json::to_string(&sync_data).unwrap();
            let callback = create_success_callback(&json_data);

            let result = get_last_synced_block(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 12345);
        }

        #[wasm_bindgen_test]
        async fn test_get_last_synced_block_empty() {
            let callback = create_success_callback("[]");

            let result = get_last_synced_block(&callback).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_get_last_synced_block_query_fails() {
            let callback = create_success_callback("invalid_json");

            let result = get_last_synced_block(&callback).await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::LocalDbQueryError(LocalDbQueryError::JsonError(_)) => {}
                other => panic!("Expected LocalDbQueryError::JsonError, got {other:?}"),
            }
        }

        #[wasm_bindgen_test]
        fn test_send_status_message_success() {
            let callback = js_sys::Function::new_no_args("return true;");
            let message = "Test status message".to_string();

            let result = send_status_message(&callback, message);

            assert!(result.is_ok());
        }

        #[wasm_bindgen_test]
        fn test_send_status_message_callback_error() {
            let callback = js_sys::Function::new_no_args("throw new Error('Callback failed');");
            let message = "Test status message".to_string();

            let result = send_status_message(&callback, message);

            assert!(result.is_err());
            match result {
                Err(LocalDbError::CustomError(msg)) => {
                    assert!(msg.contains("JavaScript callback error"));
                }
                _ => panic!("Expected CustomError from JavaScript callback failure"),
            }
        }

        fn create_status_collector() -> (js_sys::Function, Rc<RefCell<Vec<String>>>) {
            let captured = Rc::new(RefCell::new(Vec::<String>::new()));
            let captured_clone = captured.clone();

            let callback = Closure::wrap(Box::new(move |msg: String| -> JsValue {
                captured_clone.borrow_mut().push(msg);
                JsValue::TRUE
            }) as Box<dyn Fn(String) -> JsValue>);

            (callback.into_js_value().dyn_into().unwrap(), captured)
        }

        fn create_dispatching_db_callback(
            tables_json: &str,
            last_synced_json: &str,
        ) -> js_sys::Function {
            // Build two success payloads and choose based on SQL string
            let success_tables = WasmEncodedResult::Success::<String> {
                value: tables_json.to_string(),
                error: None,
            };
            let success_last = WasmEncodedResult::Success::<String> {
                value: last_synced_json.to_string(),
                error: None,
            };

            let tables_json_val = serde_wasm_bindgen::to_value(&success_tables).unwrap();
            let last_json_val = serde_wasm_bindgen::to_value(&success_last).unwrap();

            let tables_literal = js_sys::JSON::stringify(&tables_json_val)
                .unwrap()
                .as_string()
                .unwrap();
            let last_literal = js_sys::JSON::stringify(&last_json_val)
                .unwrap()
                .as_string()
                .unwrap();

            js_sys::Function::new_with_args(
                "sql",
                &format!(
                    "if (sql.includes('sqlite_master')) return {};
                     if (sql.includes('sync_status')) return {};
                     return {};",
                    tables_literal, last_literal, tables_literal
                ),
            )
        }

        fn make_tables_json() -> String {
            let table_data: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            serde_json::to_string(&table_data).unwrap()
        }

        #[wasm_bindgen_test]
        async fn test_sync_invalid_chain_id() {
            // Any client config is fine; we bail after initial steps
            let client = RaindexClient::new(
                vec![crate::raindex_client::tests::get_test_yaml(
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            // Callbacks
            let db_callback = create_dispatching_db_callback(&make_tables_json(), "[]");
            let (status_callback, captured) = create_status_collector();

            let result = client
                .sync_database(db_callback, status_callback, 999u32)
                .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to get orderbook configurations"));
                }
                other => panic!("Expected CustomError from missing chain ID, got {other:?}"),
            }

            // Status messages should be emitted before the error occurs
            let msgs = captured.borrow();
            assert!(msgs.len() >= 3);
            assert_eq!(msgs[0], "Starting database sync...");
            assert!(msgs[1].starts_with("has tables:"));
            assert!(msgs[2].starts_with("Last synced block:"));
        }

        #[wasm_bindgen_test]
        async fn test_sync_tables_exist_last_synced_zero() {
            // Use test YAML; orderbook address must match the YAML
            let client = RaindexClient::new(
                vec![crate::raindex_client::tests::get_test_yaml(
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let tables_json = make_tables_json();
            let last_synced_json = "[]"; // yields last_synced_block = 0
            let db_callback = create_dispatching_db_callback(&tables_json, last_synced_json);
            let (status_callback, captured) = create_status_collector();

            let result = client
                .sync_database(db_callback, status_callback, 1u32)
                .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to get latest block"));
                }
                other => panic!("Expected CustomError from latest block fetch, got {other:?}"),
            }

            let msgs = captured.borrow();
            // Check key status messages in order of occurrence
            assert!(msgs.len() >= 3);
            assert_eq!(msgs[0], "Starting database sync...");
            assert_eq!(msgs[1], "has tables: true");
            assert_eq!(msgs[2], "Last synced block: 0");
        }

        #[wasm_bindgen_test]
        async fn test_sync_missing_orderbook_error() {
            let client = RaindexClient::new(
                vec![crate::raindex_client::tests::get_test_yaml(
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let tables_json = make_tables_json();
            // Return a non-zero last synced for variety
            let last_synced = vec![SyncStatusResponse {
                id: 1,
                last_synced_block: 123,
                updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            }];
            let last_synced_json = serde_json::to_string(&last_synced).unwrap();
            let db_callback = create_dispatching_db_callback(&tables_json, &last_synced_json);
            let (status_callback, captured) = create_status_collector();

            let result = client
                .sync_database(db_callback, status_callback, 999u32)
                .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to get orderbook configurations"));
                }
                other => panic!("Expected CustomError from missing orderbook, got {other:?}"),
            }

            let msgs = captured.borrow();
            assert!(msgs.len() >= 3);
            assert_eq!(msgs[0], "Starting database sync...");
            assert_eq!(msgs[1], "has tables: true");
            assert_eq!(msgs[2], "Last synced block: 123");
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use httpmock::prelude::*;
        use std::io::Write;

        fn create_gzipped_sql() -> Vec<u8> {
            let sql_content = "CREATE TABLE test (id INTEGER);";
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(sql_content.as_bytes()).unwrap();
            encoder.finish().unwrap()
        }

        fn create_invalid_gzip() -> Vec<u8> {
            b"invalid gzip content".to_vec()
        }

        #[tokio::test]
        async fn test_download_and_decompress_success() {
            let server = MockServer::start();
            let gzipped_data = create_gzipped_sql();

            let mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(200)
                    .header("content-type", "application/gzip")
                    .body(gzipped_data);
            });

            let modified_fn = async {
                let client = Client::new();
                let response = client.get(server.url("/")).send().await?;

                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();

                let mut decoder = GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;

                Ok(decompressed)
            };

            let result = modified_fn.await;

            mock.assert();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "CREATE TABLE test (id INTEGER);");
        }

        #[tokio::test]
        async fn test_download_and_decompress_http_404() {
            let server = MockServer::start();

            let mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(404);
            });

            let modified_fn = async {
                let client = Client::new();
                let response = client.get(server.url("/")).send().await?;

                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();

                let mut decoder = GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;

                Ok(decompressed)
            };

            let result = modified_fn.await;

            mock.assert();
            assert!(result.is_err());
            match result {
                Err(LocalDbError::CustomError(msg)) => {
                    assert!(msg.contains("Failed to download dump, status: 404"));
                }
                _ => panic!("Expected CustomError with 404 status"),
            }
        }

        #[tokio::test]
        async fn test_download_and_decompress_http_500() {
            let server = MockServer::start();

            let mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(500);
            });

            let modified_fn = async {
                let client = Client::new();
                let response = client.get(server.url("/")).send().await?;

                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();

                let mut decoder = GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;

                Ok(decompressed)
            };

            let result = modified_fn.await;

            mock.assert();
            assert!(result.is_err());
            match result {
                Err(LocalDbError::CustomError(msg)) => {
                    assert!(msg.contains("Failed to download dump, status: 500"));
                }
                _ => panic!("Expected CustomError with 500 status"),
            }
        }

        #[tokio::test]
        async fn test_download_and_decompress_invalid_gzip() {
            let server = MockServer::start();
            let invalid_data = create_invalid_gzip();

            let mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(200)
                    .header("content-type", "application/gzip")
                    .body(invalid_data);
            });

            let modified_fn = async {
                let client = Client::new();
                let response = client.get(server.url("/")).send().await?;

                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();

                let mut decoder = GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;

                Ok(decompressed)
            };

            let result = modified_fn.await;

            mock.assert();
            assert!(result.is_err());
            match result {
                Err(LocalDbError::IoError(_)) => (),
                _ => panic!("Expected IoError from invalid gzip decompression"),
            }
        }

        #[tokio::test]
        async fn test_download_and_decompress_network_timeout() {
            let modified_fn = async {
                let client = Client::builder()
                    .timeout(std::time::Duration::from_millis(1))
                    .build()?;
                let response = client.get("https://httpbin.org/delay/10").send().await?;

                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();

                let mut decoder = GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                decoder.read_to_string(&mut decompressed)?;

                Ok(decompressed)
            };

            let result = modified_fn.await;

            assert!(result.is_err());
            match result {
                Err(LocalDbError::Http(_)) => (),
                _ => panic!("Expected Http error from network timeout"),
            }
        }
    }
}
