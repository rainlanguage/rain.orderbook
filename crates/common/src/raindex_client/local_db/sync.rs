use super::token_fetch::fetch_erc20_metadata_concurrent;
use super::{
    decode::{DecodedEvent, DecodedEventData},
    insert,
    query::{
        create_tables::REQUIRED_TABLES, fetch_erc20_tokens_by_addresses::Erc20TokenRow,
        fetch_store_addresses::StoreAddressRow, LocalDbQuery, LocalDbQueryError,
    },
    tokens::{collect_store_addresses, collect_token_addresses},
    FetchConfig, LocalDb, LocalDbError, RaindexClient,
};
use alloy::primitives::Address;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::collections::BTreeSet;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    str::FromStr,
};
use wasm_bindgen_utils::{prelude::*, wasm_export};

const DUMP_URL: &str = "https://raw.githubusercontent.com/rainlanguage/rain.strategies/07d48a0dd5136d42a29f2b0d8950cc9d77dfb1c9/local_db.sql.gz";

async fn check_required_tables(db_callback: &js_sys::Function) -> Result<bool, LocalDbQueryError> {
    let tables = LocalDbQuery::fetch_all_tables(db_callback).await?;
    let existing_table_names: std::collections::HashSet<String> =
        tables.into_iter().map(|t| t.name).collect();

    let has_all_tables = REQUIRED_TABLES
        .iter()
        .all(|&table| existing_table_names.contains(table));

    Ok(has_all_tables)
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

pub async fn get_last_synced_block(
    db_callback: &js_sys::Function,
) -> Result<u64, LocalDbQueryError> {
    let results = LocalDbQuery::fetch_last_synced_block(db_callback).await?;
    if let Some(sync_status) = results.first() {
        Ok(sync_status.last_synced_block)
    } else {
        Ok(0)
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

        let has_tables = check_required_tables(&db_callback)
            .await
            .map_err(LocalDbError::TableCheckFailed)?;

        send_status_message(&status_callback, format!("has tables: {}", has_tables))?;

        if !has_tables {
            send_status_message(
                &status_callback,
                "Initializing database tables and importing data...".to_string(),
            )?;
            let dump_sql = download_and_decompress_dump().await?;

            LocalDbQuery::execute_query_text(&db_callback, &dump_sql).await?;
        }

        let last_synced_block = get_last_synced_block(&db_callback)
            .await
            .map_err(LocalDbError::SyncStatusReadFailed)?;
        send_status_message(
            &status_callback,
            format!("Last synced block: {}", last_synced_block),
        )?;

        let orderbooks = self
            .get_orderbooks_by_chain_id(chain_id)
            .map_err(|e| LocalDbError::OrderbookConfigNotFound(Box::new(e)))?;

        let Some(orderbook_cfg) = orderbooks.first() else {
            return Err(LocalDbError::CustomError(format!(
                "No orderbook configuration found for chain ID {}",
                chain_id
            )));
        };

        let local_db = LocalDb::new_with_regular_rpcs(orderbook_cfg.network.rpcs.clone())?;

        let latest_block = local_db.rpc_client().get_latest_block_number().await?;

        let start_block = if last_synced_block == 0 {
            orderbook_cfg.deployment_block
        } else {
            last_synced_block + 1
        };

        send_status_message(
            &status_callback,
            "Fetching latest onchain events...".to_string(),
        )?;
        let events = local_db
            .fetch_events(
                &orderbook_cfg.address.to_string(),
                start_block,
                latest_block,
            )
            .await
            .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

        send_status_message(&status_callback, "Decoding fetched events...".to_string())?;
        let mut decoded_events = local_db
            .decode_events(&events)
            .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

        let existing_stores: Vec<StoreAddressRow> =
            LocalDbQuery::fetch_store_addresses(&db_callback).await?;
        let store_addresses_vec = collect_all_store_addresses(&decoded_events, &existing_stores);

        let store_events = local_db
            .fetch_store_set_events(
                &store_addresses_vec,
                start_block,
                latest_block,
                &FetchConfig::default(),
            )
            .await
            .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

        let mut decoded_store_events = local_db
            .decode_events(&store_events)
            .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

        merge_store_events(&mut decoded_events, &mut decoded_store_events);

        send_status_message(
            &status_callback,
            "Populating token information...".to_string(),
        )?;
        let prep =
            prepare_erc20_tokens_prefix(&db_callback, &local_db, chain_id, &decoded_events).await?;

        send_status_message(&status_callback, "Populating database...".to_string())?;

        let prefix_sql = if prep.tokens_prefix_sql.is_empty() {
            None
        } else {
            Some(prep.tokens_prefix_sql.as_str())
        };

        let sql_commands = local_db
            .decoded_events_to_sql(
                &decoded_events,
                latest_block,
                &prep.decimals_by_addr,
                prefix_sql,
            )
            .map_err(|e| LocalDbError::SqlGenerationFailed(Box::new(e)))?;

        LocalDbQuery::execute_query_text(&db_callback, &sql_commands).await?;

        send_status_message(&status_callback, "Database sync complete.".to_string())?;
        Ok(())
    }
}

fn sort_events_by_block_and_log(events: &mut [DecodedEventData<DecodedEvent>]) {
    events.sort_by(|a, b| {
        let block_a = parse_block_number(&a.block_number);
        let block_b = parse_block_number(&b.block_number);
        block_a
            .cmp(&block_b)
            .then_with(|| parse_block_number(&a.log_index).cmp(&parse_block_number(&b.log_index)))
    });
}

fn parse_block_number(value: &str) -> u64 {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        trimmed.parse::<u64>().unwrap_or(0)
    }
}

fn collect_all_store_addresses(
    decoded_events: &[DecodedEventData<DecodedEvent>],
    existing_stores: &[StoreAddressRow],
) -> Vec<String> {
    let mut store_addresses: BTreeSet<String> = collect_store_addresses(decoded_events)
        .into_iter()
        .collect();

    for row in existing_stores {
        if !row.store_address.is_empty() {
            store_addresses.insert(row.store_address.to_ascii_lowercase());
        }
    }

    store_addresses.into_iter().collect()
}

fn merge_store_events(
    decoded_events: &mut Vec<DecodedEventData<DecodedEvent>>,
    store_events: &mut Vec<DecodedEventData<DecodedEvent>>,
) {
    if store_events.is_empty() {
        return;
    }

    decoded_events.append(store_events);
    sort_events_by_block_and_log(decoded_events);
}

struct TokenPrepResult {
    tokens_prefix_sql: String,
    decimals_by_addr: HashMap<Address, u8>,
}

async fn prepare_erc20_tokens_prefix(
    db_callback: &js_sys::Function,
    local_db: &LocalDb,
    chain_id: u32,
    decoded_events: &[DecodedEventData<DecodedEvent>],
) -> Result<TokenPrepResult, LocalDbError> {
    let address_set = collect_token_addresses(decoded_events);
    let mut all_token_addrs: Vec<Address> = address_set.into_iter().collect();
    all_token_addrs.sort();

    let mut tokens_prefix_sql = String::new();
    let mut decimals_by_addr: HashMap<Address, u8> = HashMap::new();

    if !all_token_addrs.is_empty() {
        let addr_strings: Vec<String> = all_token_addrs
            .iter()
            .map(|a| format!("0x{:x}", a))
            .collect();

        let existing_rows: Vec<Erc20TokenRow> =
            LocalDbQuery::fetch_erc20_tokens_by_addresses(db_callback, chain_id, &addr_strings)
                .await?;

        let mut existing_set: HashSet<Address> = HashSet::new();
        for row in existing_rows.iter() {
            if let Ok(addr) = Address::from_str(&row.address) {
                decimals_by_addr.insert(addr, row.decimals);
                existing_set.insert(addr);
            }
        }

        let missing_addrs: Vec<Address> = all_token_addrs
            .into_iter()
            .filter(|addr| !existing_set.contains(addr))
            .collect();

        if !missing_addrs.is_empty() {
            let rpcs = local_db.rpc_client().rpc_urls().to_vec();
            let successes = fetch_erc20_metadata_concurrent(rpcs, missing_addrs).await?;

            tokens_prefix_sql = insert::generate_erc20_tokens_sql(chain_id, &successes);

            for (addr, info) in successes.iter() {
                decimals_by_addr.insert(*addr, info.decimals);
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
        use crate::raindex_client::local_db::decode::EventType;
        use crate::raindex_client::local_db::query::{
            create_tables::REQUIRED_TABLES, fetch_last_synced_block::SyncStatusResponse,
            fetch_tables::TableResponse, tests::create_success_callback, LocalDbQueryError,
        };
        use crate::raindex_client::RaindexError;
        use alloy::primitives::{Address, U256};
        use rain_orderbook_app_settings::yaml::YamlError;
        use rain_orderbook_bindings::IOrderBookV5::{DepositV2, WithdrawV2};
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::str::FromStr;
        use wasm_bindgen::JsCast;
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

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbQueryError::JsonError(_) => {}
                other => panic!("Expected LocalDbQueryError::JsonError, got {other:?}"),
            }
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
                LocalDbQueryError::JsonError(_) => {}
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
            let decoded: Vec<DecodedEventData<DecodedEvent>> = Vec::new();

            let res = prepare_erc20_tokens_prefix(&db_cb, &local_db, 1, &decoded)
                .await
                .unwrap();
            assert!(res.tokens_prefix_sql.is_empty());
            assert!(res.decimals_by_addr.is_empty());
        }

        #[wasm_bindgen_test]
        async fn test_prepare_erc20_tokens_prefix_sql_all_known() {
            // decoded events with two tokens
            let mut events: Vec<DecodedEventData<DecodedEvent>> = Vec::new();
            let deposit = DepositV2 {
                sender: Address::from([0x11; 20]),
                token: Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(),
                vaultId: U256::from(1).into(),
                depositAmountUint256: U256::from(0),
            };
            events.push(DecodedEventData {
                event_type: EventType::DepositV2,
                block_number: "0x0".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0x0".into(),
                log_index: "0x0".into(),
                decoded_data: DecodedEvent::DepositV2(Box::new(deposit)),
            });

            let withdraw = WithdrawV2 {
                sender: Address::from([0x22; 20]),
                token: Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
                vaultId: U256::from(2).into(),
                targetAmount: U256::from(0).into(),
                withdrawAmount: U256::from(0).into(),
                withdrawAmountUint256: U256::from(0),
            };
            events.push(DecodedEventData {
                event_type: EventType::WithdrawV2,
                block_number: "0x0".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0x1".into(),
                log_index: "0x1".into(),
                decoded_data: DecodedEvent::WithdrawV2(Box::new(withdraw)),
            });

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

            let res = prepare_erc20_tokens_prefix(&cb, &local_db, 1, &events)
                .await
                .unwrap();
            assert!(res.tokens_prefix_sql.is_empty());
            assert_eq!(res.decimals_by_addr.len(), 2);
            let addr_a = Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
            let addr_b = Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
            assert_eq!(res.decimals_by_addr.get(&addr_a), Some(&18));
            assert_eq!(res.decimals_by_addr.get(&addr_b), Some(&6));
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
        async fn test_sync_unknown_chain_id() {
            // Any client config is fine; we bail before using it once the chain lookup fails
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

            // Provide existing tables and an empty sync status so we skip dump downloads
            let tables_json = make_tables_json();
            let db_callback = create_dispatching_db_callback(&tables_json, "[]");
            let (status_callback, captured) = create_status_collector();

            let missing_chain_id = 999_999u32;
            let result = client
                .sync_database(db_callback, status_callback, missing_chain_id)
                .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::OrderbookConfigNotFound(err) => match *err {
                    RaindexError::YamlError(YamlError::NotFound(message)) => {
                        assert!(message.contains(&missing_chain_id.to_string()));
                    }
                    other => panic!("Expected YamlError::NotFound, got {other:?}"),
                },
                other => panic!("Expected OrderbookConfigNotFound, got {other:?}"),
            }
            // We emit status messages before failing on the chain lookup
            let msgs = captured.borrow();
            assert!(msgs.len() >= 3);
            assert_eq!(msgs[0], "Starting database sync...");
            assert_eq!(msgs[1], "has tables: true");
            assert_eq!(msgs[2], "Last synced block: 0");
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

            // Chain ID for mainnet in test YAML
            let result = client.sync_database(db_callback, status_callback, 1).await;

            // Should eventually fail when attempting to reach the mocked RPC endpoint
            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::Rpc(_) => {}
                other => panic!("Expected Rpc error, got {other:?}"),
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

            // Chain ID not present in the test YAML
            let missing_chain_id = 999_999u32;
            let result = client
                .sync_database(db_callback, status_callback, missing_chain_id)
                .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::OrderbookConfigNotFound(err) => match *err {
                    RaindexError::YamlError(YamlError::NotFound(message)) => {
                        assert!(message.contains(&missing_chain_id.to_string()));
                    }
                    other => panic!("Expected YamlError::NotFound, got {other:?}"),
                },
                other => panic!("Expected OrderbookConfigNotFound, got {other:?}"),
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

        use crate::raindex_client::local_db::decode::{EventType, InterpreterStoreSetEvent};
        use alloy::primitives::FixedBytes;

        fn make_store_set_event(
            block_number: u64,
            log_index: u64,
            store_byte: u8,
        ) -> DecodedEventData<DecodedEvent> {
            let store_event = InterpreterStoreSetEvent {
                store_address: Address::from([store_byte; 20]),
                namespace: FixedBytes::<32>::from([0x11; 32]),
                key: FixedBytes::<32>::from([0x22; 32]),
                value: FixedBytes::<32>::from([0x33; 32]),
            };

            DecodedEventData {
                event_type: EventType::InterpreterStoreSet,
                block_number: format!("0x{block_number:x}"),
                block_timestamp: "0x0".into(),
                transaction_hash: format!("0x{block_number:x}{log_index:x}"),
                log_index: format!("0x{log_index:x}"),
                decoded_data: DecodedEvent::InterpreterStoreSet(Box::new(store_event)),
            }
        }

        #[test]
        fn test_collect_all_store_addresses_merges_sources() {
            let decoded_events = vec![
                make_store_set_event(10, 0, 0x11),
                make_store_set_event(11, 1, 0x22),
            ];

            let existing = vec![
                StoreAddressRow {
                    store_address: "0x2222222222222222222222222222222222222222".to_string(),
                },
                StoreAddressRow {
                    store_address: "0X3333333333333333333333333333333333333333".to_string(),
                },
                StoreAddressRow {
                    store_address: "".to_string(),
                },
            ];

            let result = collect_all_store_addresses(&decoded_events, &existing);

            assert_eq!(result.len(), 3);
            assert!(result.contains(&"0x1111111111111111111111111111111111111111".to_string()));
            assert!(result.contains(&"0x2222222222222222222222222222222222222222".to_string()));
            assert!(result.contains(&"0x3333333333333333333333333333333333333333".to_string()));

            // Ensure deterministic ordering (BTreeSet) and lowercasing behaviour
            assert_eq!(
                result,
                vec![
                    "0x1111111111111111111111111111111111111111".to_string(),
                    "0x2222222222222222222222222222222222222222".to_string(),
                    "0x3333333333333333333333333333333333333333".to_string(),
                ]
            );
        }

        #[test]
        fn test_merge_store_events_sorts_and_appends() {
            let mut base_events = vec![
                make_store_set_event(12, 1, 0xaa),
                make_store_set_event(15, 0, 0xbb),
            ];
            let mut store_events = vec![make_store_set_event(8, 2, 0xcc)];

            merge_store_events(&mut base_events, &mut store_events);

            assert!(store_events.is_empty(), "store events drained after merge");
            assert_eq!(base_events.len(), 3);
            assert_eq!(base_events[0].block_number, "0x8");
            assert_eq!(base_events[0].log_index, "0x2");
            match &base_events[0].decoded_data {
                DecodedEvent::InterpreterStoreSet(store_event) => {
                    assert_eq!(store_event.store_address, Address::from([0xcc; 20]));
                }
                other => panic!("expected InterpreterStoreSet event, got {other:?}"),
            }
        }

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
            let sql =
                crate::raindex_client::local_db::insert::generate_erc20_tokens_sql(1, &fetched);
            assert!(sql.contains("INSERT INTO erc20_tokens"));
            assert!(sql.contains("0x"));
            assert!(sql.contains("Token1"));
            assert!(sql.contains("TOKEN1"));
            assert!(sql.contains("decimals"));
        }
    }
}
