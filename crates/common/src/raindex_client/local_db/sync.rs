use super::helpers::patch_deposit_amounts_with_decimals;
use super::token_fetch::fetch_erc20_metadata_concurrent;
use super::{
    insert::generate_erc20_tokens_sql,
    query::{
        create_tables::REQUIRED_TABLES, fetch_erc20_tokens_by_addresses::Erc20TokenRow,
        fetch_store_addresses::StoreAddressRow, LocalDbQuery,
    },
    tokens::{collect_store_addresses, collect_token_addresses},
    *,
};
use alloy::primitives::Address;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    str::FromStr,
};

const DUMP_URL: &str = "https://raw.githubusercontent.com/rainlanguage/rain.strategies/07d48a0dd5136d42a29f2b0d8950cc9d77dfb1c9/local_db.sql.gz";

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
        let mut raw_events: Vec<serde_json::Value> = Vec::new();

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

        if let Some(orderbook_logs) = events.as_array() {
            raw_events.extend(orderbook_logs.iter().cloned());
        }

        send_status_message(&status_callback, "Decoding fetched events...".to_string())?;
        let mut decoded_events = match local_db.decode_events(events) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "There was a problem trying to decode events: {}",
                    e
                )));
            }
        };

        let mut store_addresses: HashSet<String> = collect_store_addresses(&decoded_events)
            .into_iter()
            .collect();

        let existing_stores: Vec<StoreAddressRow> =
            LocalDbQuery::fetch_store_addresses(&db_callback).await?;
        for row in existing_stores {
            if !row.store_address.is_empty() {
                store_addresses.insert(row.store_address.to_ascii_lowercase());
            }
        }

        let store_addresses_vec: Vec<String> = store_addresses.into_iter().collect();

        let store_events = local_db
            .fetch_store_set_events(
                &store_addresses_vec,
                start_block,
                latest_block,
                &FetchConfig::default(),
            )
            .await?;

        if let Some(store_logs) = store_events.as_array() {
            raw_events.extend(store_logs.iter().cloned());
        }

        let decoded_store_events = match local_db.decode_events(store_events) {
            Ok(result) => result,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "There was a problem trying to decode interpreter store events: {}",
                    e
                )));
            }
        };

        if let (Some(mut base_events), Some(store_array)) = (
            decoded_events.as_array().cloned(),
            decoded_store_events.as_array(),
        ) {
            base_events.extend(store_array.iter().cloned());
            decoded_events = serde_json::Value::Array(base_events);
        }

        let raw_events_sql = match local_db.raw_events_to_sql(&raw_events) {
            Ok(sql) => sql,
            Err(e) => {
                return Err(LocalDbError::CustomError(format!(
                    "Failed to build raw events SQL: {}",
                    e
                )));
            }
        };

        send_status_message(
            &status_callback,
            "Populating token information...".to_string(),
        )?;
        let prep =
            prepare_erc20_tokens_prefix(&db_callback, &local_db, chain_id, &decoded_events).await?;
        let TokenPrepResult {
            tokens_prefix_sql,
            decimals_by_addr,
        } = prep;

        let decoded_events =
            patch_deposit_amounts_with_decimals(decoded_events, &decimals_by_addr)?;

        let mut combined_prefix_sql = raw_events_sql;
        if !tokens_prefix_sql.is_empty() {
            combined_prefix_sql.push_str(&tokens_prefix_sql);
        }

        send_status_message(&status_callback, "Populating database...".to_string())?;
        let sql_commands = match local_db.decoded_events_to_sql_with_prefix(
            decoded_events,
            latest_block,
            &combined_prefix_sql,
        ) {
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

struct TokenPrepResult {
    tokens_prefix_sql: String,
    decimals_by_addr: HashMap<String, u8>,
}

async fn prepare_erc20_tokens_prefix(
    db_callback: &js_sys::Function,
    local_db: &LocalDb,
    chain_id: u32,
    decoded_events: &serde_json::Value,
) -> Result<TokenPrepResult, LocalDbError> {
    let address_set = collect_token_addresses(decoded_events);
    let mut all_token_addrs: Vec<Address> = address_set.into_iter().collect();
    all_token_addrs.sort();

    let mut tokens_prefix_sql = String::new();
    let mut decimals_by_addr: HashMap<String, u8> = HashMap::new();

    if !all_token_addrs.is_empty() {
        let addr_strings: Vec<String> = all_token_addrs
            .iter()
            .map(|a| format!("0x{:x}", a))
            .collect();

        let existing: Vec<Erc20TokenRow> =
            LocalDbQuery::fetch_erc20_tokens_by_addresses(db_callback, chain_id, &addr_strings)
                .await?;
        // Populate decimals map from existing DB rows
        for row in existing.iter() {
            decimals_by_addr.insert(row.address.to_ascii_lowercase(), row.decimals);
        }

        let existing_set: HashSet<String> = existing
            .into_iter()
            .map(|r| r.address.to_ascii_lowercase())
            .collect();
        let missing_strings: Vec<String> = addr_strings
            .into_iter()
            .filter(|s| !existing_set.contains(s))
            .collect();

        if !missing_strings.is_empty() {
            let rpcs = local_db.rpc_urls().to_vec();
            let missing_addrs: Vec<Address> = missing_strings
                .iter()
                .filter_map(|s| Address::from_str(s).ok())
                .collect();

            let successes = fetch_erc20_metadata_concurrent(rpcs, missing_addrs).await?;

            tokens_prefix_sql = generate_erc20_tokens_sql(chain_id, &successes);

            // Populate decimals map from fetched metadata
            for (addr, info) in successes.into_iter() {
                let key = format!("0x{:x}", addr);
                decimals_by_addr.insert(key, info.decimals);
            }
        }
    }

    Ok(TokenPrepResult {
        tokens_prefix_sql,
        decimals_by_addr,
    })
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
        use alloy::primitives::U256;
        use rain_math_float::Float;
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

        #[wasm_bindgen_test]
        async fn test_prepare_erc20_tokens_prefix_sql_no_tokens() {
            let db_cb = create_success_callback("[]");

            let local_db = LocalDb::default();
            let decoded = serde_json::json!([]);

            let res = prepare_erc20_tokens_prefix(&db_cb, &local_db, 1, &decoded)
                .await
                .unwrap();
            assert!(res.tokens_prefix_sql.is_empty());
            assert!(res.decimals_by_addr.is_empty());
        }

        #[wasm_bindgen_test]
        async fn test_prepare_erc20_tokens_prefix_sql_all_known() {
            // decoded events with two tokens
            let decoded = serde_json::json!([
                {"event_type":"DepositV2","decoded_data": {"token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
                {"event_type":"WithdrawV2","decoded_data": {"token": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}
            ]);

            // Callback returns both rows from erc20_tokens query
            let rows = vec![
                Erc20TokenRow {
                    chain_id: 1,
                    address: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    name: "A".into(),
                    symbol: "AA".into(),
                    decimals: 18,
                },
                Erc20TokenRow {
                    chain_id: 1,
                    address: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                    name: "B".into(),
                    symbol: "BB".into(),
                    decimals: 6,
                },
            ];
            let rows_json = serde_json::to_string(&rows).unwrap();

            let cb = js_sys::Function::new_with_args(
                "sql",
                &format!(
                    "if (sql.includes('FROM erc20_tokens')) return {};
                     return {};",
                    js_sys::JSON::stringify(
                        &serde_wasm_bindgen::to_value(&WasmEncodedResult::Success::<String> {
                            value: rows_json,
                            error: None
                        })
                        .unwrap()
                    )
                    .unwrap()
                    .as_string()
                    .unwrap(),
                    js_sys::JSON::stringify(
                        &serde_wasm_bindgen::to_value(&WasmEncodedResult::Success::<String> {
                            value: "[]".into(),
                            error: None
                        })
                        .unwrap()
                    )
                    .unwrap()
                    .as_string()
                    .unwrap(),
                ),
            );

            let local_db = LocalDb::default();

            let res = prepare_erc20_tokens_prefix(&cb, &local_db, 1, &decoded)
                .await
                .unwrap();
            assert!(res.tokens_prefix_sql.is_empty());
            assert_eq!(res.decimals_by_addr.len(), 2);
            assert_eq!(
                res.decimals_by_addr
                    .get("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                Some(&18)
            );
            assert_eq!(
                res.decimals_by_addr
                    .get("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                Some(&6)
            );
        }

        #[wasm_bindgen_test]
        fn test_patch_deposit_amounts_with_decimals_success() {
            let token = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
            let decoded = serde_json::json!([
                {
                    "event_type": "DepositV2",
                    "decoded_data": {
                        "sender": "0x0000000000000000000000000000000000000001",
                        "token": token,
                        "vault_id": "0x01",
                        "deposit_amount_uint256": "0xfa0"
                    }
                }
            ]);

            let mut map = HashMap::new();
            map.insert(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                6u8,
            );

            let patched = patch_deposit_amounts_with_decimals(decoded, &map).unwrap();
            let arr = patched.as_array().unwrap();
            let dd = &arr[0]["decoded_data"];

            let expected = Float::from_fixed_decimal(U256::from(4000u64), 6)
                .unwrap()
                .as_hex();
            assert_eq!(dd["deposit_amount"], expected);
        }

        #[wasm_bindgen_test]
        fn test_patch_deposit_amounts_with_decimals_missing_decimals_errors() {
            let decoded = serde_json::json!([
                {
                    "event_type": "DepositV2",
                    "decoded_data": {
                        "sender": "0x0000000000000000000000000000000000000001",
                        "token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "vault_id": "0x01",
                        "deposit_amount_uint256": "0xfa0"
                    }
                }
            ]);

            let map: HashMap<String, u8> = HashMap::new();

            let res = patch_deposit_amounts_with_decimals(decoded, &map);
            assert!(res.is_err());
            match res.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Missing decimals for token"));
                }
                other => panic!("Expected CustomError for missing decimals, got {other:?}"),
            }
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
        use rain_orderbook_test_fixtures::LocalEvm;
        use std::io::Write;
        use url::Url;

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

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_generate_tokens_sql_after_fetch_success() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let rpcs = vec![Url::parse(&local_evm.url()).unwrap()];
            let addr = *local_evm.tokens[0].address();

            let fetched = fetch_erc20_metadata_concurrent(rpcs, vec![addr])
                .await
                .unwrap();
            let sql = generate_erc20_tokens_sql(1, &fetched);
            assert!(sql.contains("INSERT INTO erc20_tokens"));
            assert!(sql.contains("0x"));
            assert!(sql.contains("Token1"));
            assert!(sql.contains("TOKEN1"));
            assert!(sql.contains("decimals"));
        }
    }
}
