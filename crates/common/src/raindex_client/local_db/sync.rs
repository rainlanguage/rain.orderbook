use super::{
    decode::decode_events,
    insert::decoded_events_to_sql,
    query::{create_tables::REQUIRED_TABLES, LocalDbQuery},
    *,
};
use flate2::read::GzDecoder;
use reqwest::Client;
use std::io::Read;

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
    let response = client.get("").send().await?;

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

async fn import_database_dump(db_callback: &js_sys::Function) -> Result<(), LocalDbError> {
    let dump_sql = download_and_decompress_dump().await?;

    LocalDbQuery::execute_query_with_callback::<()>(db_callback, &dump_sql).await?;
    // TODO: Replace with actual block number from dump
    LocalDbQuery::update_last_synced_block(db_callback, 1).await?;

    Ok(())
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
        Err(_) => Ok(0),
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
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
        #[wasm_export(param_description = "JavaScript function called with status updates")]
        status_callback: js_sys::Function,
        #[wasm_export(param_description = "The contract address to sync events for")]
        contract_address: String,
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

        if !has_tables {
            send_status_message(
                &status_callback,
                "Initializing database tables and importing data...".to_string(),
            )?;
            if let Err(e) = import_database_dump(&db_callback).await {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to import database dump: {}",
                    e
                )));
            }
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

        // TODO: Replace with actual client initialization
        let local_db = LocalDb::new(1, "".to_string())?;

        let latest_block = match local_db.client.get_latest_block_number().await {
            Ok(block) => block,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to get latest block: {}",
                    e
                )));
            }
        };

        let start_block = if last_synced_block == 0 {
            // TODO: Fetch the contract deployment block
            1
        } else {
            last_synced_block + 1
        };

        send_status_message(
            &status_callback,
            "Fetching latest onchain events...".to_string(),
        )?;
        let events = match local_db
            .fetch_events(&contract_address, start_block, latest_block)
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
        // TODO: This needs to be implemented in LocalDb
        let decoded_events = match decode_events(events) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "There was a problem trying to decode events: {}",
                    e
                )));
            }
        };

        send_status_message(&status_callback, "Populating database...".to_string())?;
        // TODO: This needs to be implemented in LocalDb
        let sql_commands = match decoded_events_to_sql(decoded_events, latest_block) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(e.to_string()));
            }
        };

        LocalDbQuery::execute_query_with_callback::<()>(&db_callback, &sql_commands).await?;

        send_status_message(&status_callback, "Database sync complete.".to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::raindex_client::local_db::query::{
            create_tables::REQUIRED_TABLES, 
            tests::create_success_callback,
            fetch_tables::TableResponse,
            fetch_last_synced_block::SyncStatusResponse
        };
        use super::*;
        use wasm_bindgen_test::*;

        wasm_bindgen_test_configure!(run_in_browser);

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

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
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
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use httpmock::prelude::*;
        use flate2::write::GzEncoder;
        use flate2::Compression;
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
                let response = client.get(&server.url("/")).send().await?;

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
                let response = client.get(&server.url("/")).send().await?;

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
                let response = client.get(&server.url("/")).send().await?;

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
                let response = client.get(&server.url("/")).send().await?;

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
