use super::{
    decode::{DecodedEvent, DecodedEventData},
    insert,
    query::{
        create_tables::REQUIRED_TABLES, fetch_erc20_tokens_by_addresses::Erc20TokenRow,
        fetch_last_synced_block::SyncStatusResponse, fetch_store_addresses::StoreAddressRow,
        fetch_tables::TableResponse, LocalDbQuery, LocalDbQueryError,
    },
    token_fetch::fetch_erc20_metadata_concurrent,
    tokens::{collect_store_addresses, collect_token_addresses},
    FetchConfig, LocalDb, LocalDbError, RaindexClient,
};
use crate::{erc20::TokenInfo, rpc_client::LogEntryResponse};
use alloy::primitives::Address;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::collections::BTreeSet;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    io::Read,
    pin::Pin,
    str::FromStr,
};
use wasm_bindgen_utils::{prelude::*, wasm_export};

const DUMP_URL: &str = "https://raw.githubusercontent.com/rainlanguage/rain.strategies/07d48a0dd5136d42a29f2b0d8950cc9d77dfb1c9/local_db.sql.gz";

type DbFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, LocalDbQueryError>> + 'a>>;
type LocalDbFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, LocalDbError>> + 'a>>;

trait DatabaseBridge {
    fn fetch_all_tables(&self) -> DbFuture<'_, Vec<TableResponse>>;
    fn fetch_last_synced_block(&self) -> DbFuture<'_, Vec<SyncStatusResponse>>;
    fn fetch_store_addresses(&self) -> DbFuture<'_, Vec<StoreAddressRow>>;
    fn fetch_erc20_tokens_by_addresses(
        &self,
        chain_id: u32,
        addresses: Vec<String>,
    ) -> DbFuture<'_, Vec<Erc20TokenRow>>;
    fn execute_query_text(&self, sql: String) -> DbFuture<'_, String>;
}

trait StatusSink {
    fn send(&self, message: String) -> Result<(), LocalDbError>;
}

trait LocalDbApi {
    fn latest_block_number(&self) -> LocalDbFuture<'_, u64>;
    fn fetch_events(
        &self,
        contract_address: String,
        start_block: u64,
        end_block: u64,
    ) -> LocalDbFuture<'_, Vec<LogEntryResponse>>;
    fn decode_events(
        &self,
        events: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError>;
    fn fetch_store_set_events(
        &self,
        store_addresses: Vec<String>,
        start_block: u64,
        end_block: u64,
    ) -> LocalDbFuture<'_, Vec<LogEntryResponse>>;
    fn fetch_token_metadata(
        &self,
        missing_addrs: Vec<Address>,
    ) -> LocalDbFuture<'_, Vec<(Address, TokenInfo)>>;
    fn decoded_events_to_sql(
        &self,
        events: &[DecodedEventData<DecodedEvent>],
        end_block: u64,
        decimals_by_token: &HashMap<Address, u8>,
        prefix_sql: Option<&str>,
    ) -> Result<String, LocalDbError>;
}

struct JsDatabaseBridge<'a> {
    callback: &'a js_sys::Function,
}

impl<'a> JsDatabaseBridge<'a> {
    fn new(callback: &'a js_sys::Function) -> Self {
        Self { callback }
    }
}

impl<'a> DatabaseBridge for JsDatabaseBridge<'a> {
    fn fetch_all_tables(&self) -> DbFuture<'_, Vec<TableResponse>> {
        Box::pin(LocalDbQuery::fetch_all_tables(self.callback))
    }

    fn fetch_last_synced_block(&self) -> DbFuture<'_, Vec<SyncStatusResponse>> {
        Box::pin(LocalDbQuery::fetch_last_synced_block(self.callback))
    }

    fn fetch_store_addresses(&self) -> DbFuture<'_, Vec<StoreAddressRow>> {
        Box::pin(LocalDbQuery::fetch_store_addresses(self.callback))
    }

    fn fetch_erc20_tokens_by_addresses(
        &self,
        chain_id: u32,
        addresses: Vec<String>,
    ) -> DbFuture<'_, Vec<Erc20TokenRow>> {
        Box::pin(async move {
            LocalDbQuery::fetch_erc20_tokens_by_addresses(self.callback, chain_id, &addresses).await
        })
    }

    fn execute_query_text(&self, sql: String) -> DbFuture<'_, String> {
        Box::pin(async move { LocalDbQuery::execute_query_text(self.callback, &sql).await })
    }
}

struct JsStatusReporter<'a> {
    callback: &'a js_sys::Function,
}

impl<'a> JsStatusReporter<'a> {
    fn new(callback: &'a js_sys::Function) -> Self {
        Self { callback }
    }
}

impl<'a> StatusSink for JsStatusReporter<'a> {
    fn send(&self, message: String) -> Result<(), LocalDbError> {
        send_status_message(self.callback, message)
    }
}

impl LocalDbApi for LocalDb {
    fn latest_block_number(&self) -> LocalDbFuture<'_, u64> {
        let client = self.rpc_client().clone();
        Box::pin(async move {
            client
                .get_latest_block_number()
                .await
                .map_err(LocalDbError::from)
        })
    }

    fn fetch_events(
        &self,
        contract_address: String,
        start_block: u64,
        end_block: u64,
    ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
        Box::pin(async move {
            LocalDb::fetch_events(self, &contract_address, start_block, end_block).await
        })
    }

    fn decode_events(
        &self,
        events: &[LogEntryResponse],
    ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
        LocalDb::decode_events(self, events)
    }

    fn fetch_store_set_events(
        &self,
        store_addresses: Vec<String>,
        start_block: u64,
        end_block: u64,
    ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
        Box::pin(async move {
            LocalDb::fetch_store_set_events(
                self,
                &store_addresses,
                start_block,
                end_block,
                &FetchConfig::default(),
            )
            .await
        })
    }

    fn fetch_token_metadata(
        &self,
        missing_addrs: Vec<Address>,
    ) -> LocalDbFuture<'_, Vec<(Address, TokenInfo)>> {
        let rpcs = self.rpc_client().rpc_urls().to_vec();
        Box::pin(async move { fetch_erc20_metadata_concurrent(rpcs, missing_addrs).await })
    }

    fn decoded_events_to_sql(
        &self,
        events: &[DecodedEventData<DecodedEvent>],
        end_block: u64,
        decimals_by_token: &HashMap<Address, u8>,
        prefix_sql: Option<&str>,
    ) -> Result<String, LocalDbError> {
        LocalDb::decoded_events_to_sql(self, events, end_block, decimals_by_token, prefix_sql)
    }
}

async fn check_required_tables(db: &impl DatabaseBridge) -> Result<bool, LocalDbQueryError> {
    let tables = db.fetch_all_tables().await?;
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

async fn get_last_synced_block(db: &impl DatabaseBridge) -> Result<u64, LocalDbQueryError> {
    let results = db.fetch_last_synced_block().await?;
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

async fn sync_database_with_services(
    client: &RaindexClient,
    db: &impl DatabaseBridge,
    status: &impl StatusSink,
    chain_id: u32,
    local_db_override: Option<&dyn LocalDbApi>,
) -> Result<(), LocalDbError> {
    status.send("Starting database sync...".to_string())?;

    let has_tables = check_required_tables(db)
        .await
        .map_err(LocalDbError::TableCheckFailed)?;

    status.send(format!("has tables: {}", has_tables))?;

    if !has_tables {
        status.send("Initializing database tables and importing data...".to_string())?;
        let dump_sql = download_and_decompress_dump().await?;
        db.execute_query_text(dump_sql)
            .await
            .map_err(LocalDbError::from)?;
    }

    let last_synced_block = get_last_synced_block(db)
        .await
        .map_err(LocalDbError::SyncStatusReadFailed)?;
    status.send(format!("Last synced block: {}", last_synced_block))?;

    let orderbooks = client
        .get_orderbooks_by_chain_id(chain_id)
        .map_err(|e| LocalDbError::OrderbookConfigNotFound(Box::new(e)))?;

    let Some(orderbook_cfg) = orderbooks.first() else {
        return Err(LocalDbError::CustomError(format!(
            "No orderbook configuration found for chain ID {}",
            chain_id
        )));
    };

    enum LocalDbHolder<'a> {
        Borrowed(&'a dyn LocalDbApi),
        Owned(LocalDb),
    }

    let holder = if let Some(override_db) = local_db_override {
        LocalDbHolder::Borrowed(override_db)
    } else {
        LocalDbHolder::Owned(LocalDb::new_with_regular_rpcs(
            orderbook_cfg.network.rpcs.clone(),
        )?)
    };

    let local_db: &dyn LocalDbApi = match &holder {
        LocalDbHolder::Borrowed(db) => *db,
        LocalDbHolder::Owned(db) => db,
    };

    let latest_block = local_db.latest_block_number().await?;

    let start_block = if last_synced_block == 0 {
        orderbook_cfg.deployment_block
    } else {
        last_synced_block + 1
    };

    status.send("Fetching latest onchain events...".to_string())?;
    let events = local_db
        .fetch_events(orderbook_cfg.address.to_string(), start_block, latest_block)
        .await
        .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

    status.send("Decoding fetched events...".to_string())?;
    let mut decoded_events = local_db
        .decode_events(&events)
        .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

    let existing_stores: Vec<StoreAddressRow> = db.fetch_store_addresses().await?;
    let store_addresses_vec = collect_all_store_addresses(&decoded_events, &existing_stores);

    let store_logs = local_db
        .fetch_store_set_events(store_addresses_vec, start_block, latest_block)
        .await
        .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

    let mut decoded_store_events = local_db
        .decode_events(&store_logs)
        .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

    merge_store_events(&mut decoded_events, &mut decoded_store_events);

    status.send("Populating token information...".to_string())?;
    let prep = prepare_erc20_tokens_prefix(db, local_db, chain_id, &decoded_events).await?;

    status.send("Populating database...".to_string())?;
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

    db.execute_query_text(sql_commands).await?;

    status.send("Database sync complete.".to_string())?;
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
        let db_bridge = JsDatabaseBridge::new(&db_callback);
        let status_bridge = JsStatusReporter::new(&status_callback);
        sync_database_with_services(self, &db_bridge, &status_bridge, chain_id, None).await
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
    db: &impl DatabaseBridge,
    local_db: &dyn LocalDbApi,
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

        let existing_rows: Vec<Erc20TokenRow> = db
            .fetch_erc20_tokens_by_addresses(chain_id, addr_strings.clone())
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
            let successes = local_db.fetch_token_metadata(missing_addrs).await?;

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
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = check_required_tables(&db_bridge).await;

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
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = check_required_tables(&db_bridge).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_empty_db() {
            let callback = create_success_callback("[]");
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = check_required_tables(&db_bridge).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[wasm_bindgen_test]
        async fn test_check_required_tables_query_fails() {
            let callback = create_success_callback("invalid_json");
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = check_required_tables(&db_bridge).await;

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
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = check_required_tables(&db_bridge).await;

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
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = get_last_synced_block(&db_bridge).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 12345);
        }

        #[wasm_bindgen_test]
        async fn test_get_last_synced_block_empty() {
            let callback = create_success_callback("[]");
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = get_last_synced_block(&db_bridge).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_get_last_synced_block_query_fails() {
            let callback = create_success_callback("invalid_json");
            let db_bridge = JsDatabaseBridge::new(&callback);

            let result = get_last_synced_block(&db_bridge).await;

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
            let db = JsDatabaseBridge::new(&db_cb);

            let local_db = LocalDb::default();
            let decoded: Vec<DecodedEventData<DecodedEvent>> = Vec::new();

            let res = prepare_erc20_tokens_prefix(&db, &local_db, 1, &decoded)
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
            let db = JsDatabaseBridge::new(&cb);

            let res = prepare_erc20_tokens_prefix(&db, &local_db, 1, &events)
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
        use std::{
            cell::RefCell,
            collections::HashMap,
            rc::Rc,
            sync::{Arc, Mutex},
        };
        use url::Url;

        use crate::{
            erc20::TokenInfo,
            raindex_client::{
                local_db::{
                    decode::{EventType, InterpreterStoreSetEvent},
                    query::{
                        create_tables::REQUIRED_TABLES,
                        fetch_erc20_tokens_by_addresses::Erc20TokenRow,
                        fetch_last_synced_block::SyncStatusResponse,
                        fetch_store_addresses::StoreAddressRow, fetch_tables::TableResponse,
                        LocalDbQueryError,
                    },
                },
                tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS},
            },
            rpc_client::LogEntryResponse,
        };
        use alloy::{
            hex,
            primitives::{Bytes, FixedBytes, U256},
            sol_types::SolEvent,
        };
        use rain_orderbook_bindings::IInterpreterStoreV3::Set;
        use rain_orderbook_bindings::IOrderBookV5::{
            AddOrderV3, DepositV2, EvaluableV4, OrderV4, IOV2,
        };

        const ORDERBOOK_ADDRESS: &str = CHAIN_ID_1_ORDERBOOK_ADDRESS;

        fn to_hex(value: u64) -> String {
            format!("0x{:x}", value)
        }

        fn make_log_entry(
            address: Address,
            topic: String,
            data: String,
            block_number: u64,
            block_timestamp: u64,
            transaction_suffix: u64,
            log_index: u64,
        ) -> LogEntryResponse {
            LogEntryResponse {
                address: format!("0x{:x}", address),
                topics: vec![topic],
                data,
                block_number: to_hex(block_number),
                block_timestamp: Some(to_hex(block_timestamp)),
                transaction_hash: format!("0x{:064x}", block_number * 100 + transaction_suffix),
                transaction_index: "0x0".to_string(),
                block_hash: "0x0".to_string(),
                log_index: to_hex(log_index),
                removed: false,
            }
        }

        fn build_deposit_log(
            orderbook: Address,
            sender: Address,
            token: Address,
            block_number: u64,
            log_index: u64,
        ) -> LogEntryResponse {
            let event = DepositV2 {
                sender,
                token,
                vaultId: U256::from(1u64).into(),
                depositAmountUint256: U256::from(1u64),
            };
            let encoded = format!("0x{}", hex::encode(event.encode_data()));
            make_log_entry(
                orderbook,
                format!("0x{}", hex::encode(DepositV2::SIGNATURE_HASH)),
                encoded,
                block_number,
                block_number + 1,
                log_index,
                log_index,
            )
        }

        fn build_add_order_log(
            orderbook: Address,
            store_address: Address,
            tokens: &[Address],
            block_number: u64,
            log_index: u64,
        ) -> LogEntryResponse {
            let order = OrderV4 {
                owner: Address::from([0x45; 20]),
                nonce: U256::from(2u64).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([0x55; 20]),
                    store: store_address,
                    bytecode: Bytes::from_static(b"\x01\x02"),
                },
                validInputs: tokens
                    .iter()
                    .enumerate()
                    .map(|(idx, token)| IOV2 {
                        token: *token,
                        vaultId: U256::from(idx as u64).into(),
                    })
                    .collect(),
                validOutputs: vec![],
            };
            let event = AddOrderV3 {
                sender: Address::from([0x66; 20]),
                orderHash: FixedBytes::<32>::from([0x77; 32]),
                order,
            };
            let encoded = format!("0x{}", hex::encode(event.encode_data()));
            make_log_entry(
                orderbook,
                format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH)),
                encoded,
                block_number,
                block_number + 2,
                log_index,
                log_index,
            )
        }

        fn build_store_set_log(
            store: Address,
            namespace: [u8; 32],
            key: [u8; 32],
            value: [u8; 32],
            block_number: u64,
            log_index: u64,
        ) -> LogEntryResponse {
            let mut data = Vec::with_capacity(96);
            data.extend_from_slice(&namespace);
            data.extend_from_slice(&key);
            data.extend_from_slice(&value);
            make_log_entry(
                store,
                format!("0x{}", hex::encode(Set::SIGNATURE_HASH)),
                format!("0x{}", hex::encode(data)),
                block_number,
                block_number + 3,
                log_index,
                log_index,
            )
        }

        #[derive(Default)]
        struct TestStatusSink {
            messages: RefCell<Vec<String>>,
        }

        impl TestStatusSink {
            fn new() -> Self {
                Self {
                    messages: RefCell::new(Vec::new()),
                }
            }

            fn messages(&self) -> Vec<String> {
                self.messages.borrow().clone()
            }
        }

        impl StatusSink for TestStatusSink {
            fn send(&self, message: String) -> Result<(), LocalDbError> {
                self.messages.borrow_mut().push(message);
                Ok(())
            }
        }

        struct MockDatabaseBridge {
            tables: Vec<TableResponse>,
            sync_status: Vec<SyncStatusResponse>,
            store_rows: Vec<StoreAddressRow>,
            token_rows: Vec<Erc20TokenRow>,
            executed_sql: Arc<Mutex<Vec<String>>>,
            execute_result: Arc<Mutex<Option<Result<String, LocalDbQueryError>>>>,
        }

        impl MockDatabaseBridge {
            fn new(
                tables: Vec<TableResponse>,
                sync_status: Vec<SyncStatusResponse>,
                store_rows: Vec<StoreAddressRow>,
                token_rows: Vec<Erc20TokenRow>,
            ) -> Self {
                Self {
                    tables,
                    sync_status,
                    store_rows,
                    token_rows,
                    executed_sql: Arc::new(Mutex::new(Vec::new())),
                    execute_result: Arc::new(Mutex::new(None)),
                }
            }

            fn recorded_sql(&self) -> Vec<String> {
                self.executed_sql.lock().unwrap().clone()
            }

            fn set_execute_result(&self, result: Result<String, LocalDbQueryError>) {
                *self.execute_result.lock().unwrap() = Some(result);
            }
        }

        impl DatabaseBridge for MockDatabaseBridge {
            fn fetch_all_tables(&self) -> DbFuture<'_, Vec<TableResponse>> {
                let tables = self.tables.clone();
                Box::pin(async move { Ok(tables) })
            }

            fn fetch_last_synced_block(&self) -> DbFuture<'_, Vec<SyncStatusResponse>> {
                let rows = self.sync_status.clone();
                Box::pin(async move { Ok(rows) })
            }

            fn fetch_store_addresses(&self) -> DbFuture<'_, Vec<StoreAddressRow>> {
                let rows = self.store_rows.clone();
                Box::pin(async move { Ok(rows) })
            }

            fn fetch_erc20_tokens_by_addresses(
                &self,
                chain_id: u32,
                addresses: Vec<String>,
            ) -> DbFuture<'_, Vec<Erc20TokenRow>> {
                let rows = self.token_rows.clone();
                Box::pin(async move {
                    let address_set: std::collections::HashSet<String> = addresses
                        .into_iter()
                        .map(|a| a.to_ascii_lowercase())
                        .collect();
                    let out: Vec<Erc20TokenRow> = rows
                        .into_iter()
                        .filter(|row| row.chain_id == chain_id)
                        .filter(|row| address_set.contains(&row.address.to_ascii_lowercase()))
                        .collect();
                    Ok(out)
                })
            }

            fn execute_query_text(&self, sql: String) -> DbFuture<'_, String> {
                let log = Arc::clone(&self.executed_sql);
                let exec_result = Arc::clone(&self.execute_result);
                Box::pin(async move {
                    log.lock().unwrap().push(sql.clone());
                    match exec_result.lock().unwrap().take() {
                        Some(result) => result,
                        None => Ok(sql),
                    }
                })
            }
        }

        struct MockLocalDb {
            latest_block: u64,
            event_logs: Vec<LogEntryResponse>,
            store_logs: Vec<LogEntryResponse>,
            token_metadata: HashMap<Address, TokenInfo>,
            inner: LocalDb,
            event_types: Rc<RefCell<Vec<EventType>>>,
        }

        impl MockLocalDb {
            fn new(
                latest_block: u64,
                event_logs: Vec<LogEntryResponse>,
                store_logs: Vec<LogEntryResponse>,
                token_metadata: HashMap<Address, TokenInfo>,
            ) -> Self {
                Self {
                    latest_block,
                    event_logs,
                    store_logs,
                    token_metadata,
                    inner: LocalDb::default(),
                    event_types: Rc::new(RefCell::new(Vec::new())),
                }
            }

            fn event_types(&self) -> Vec<EventType> {
                self.event_types.borrow().clone()
            }
        }

        impl LocalDbApi for MockLocalDb {
            fn latest_block_number(&self) -> LocalDbFuture<'_, u64> {
                let block = self.latest_block;
                Box::pin(async move { Ok(block) })
            }

            fn fetch_events(
                &self,
                _contract_address: String,
                _start_block: u64,
                _end_block: u64,
            ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
                let logs = self.event_logs.clone();
                Box::pin(async move { Ok(logs) })
            }

            fn decode_events(
                &self,
                events: &[LogEntryResponse],
            ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
                self.inner.decode_events(events)
            }

            fn fetch_store_set_events(
                &self,
                _store_addresses: Vec<String>,
                _start_block: u64,
                _end_block: u64,
            ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
                let logs = self.store_logs.clone();
                Box::pin(async move { Ok(logs) })
            }

            fn fetch_token_metadata(
                &self,
                missing_addrs: Vec<Address>,
            ) -> LocalDbFuture<'_, Vec<(Address, TokenInfo)>> {
                let map = self.token_metadata.clone();
                Box::pin(async move {
                    let mut out = Vec::new();
                    for addr in missing_addrs {
                        match map.get(&addr) {
                            Some(info) => out.push((addr, info.clone())),
                            None => {
                                return Err(LocalDbError::CustomError(format!(
                                    "Missing metadata for token 0x{:x}",
                                    addr
                                )))
                            }
                        }
                    }
                    Ok(out)
                })
            }

            fn decoded_events_to_sql(
                &self,
                events: &[DecodedEventData<DecodedEvent>],
                end_block: u64,
                decimals_by_token: &HashMap<Address, u8>,
                prefix_sql: Option<&str>,
            ) -> Result<String, LocalDbError> {
                *self.event_types.borrow_mut() = events.iter().map(|e| e.event_type).collect();
                self.inner
                    .decoded_events_to_sql(events, end_block, decimals_by_token, prefix_sql)
            }
        }

        type ResultCell<T> = RefCell<Option<Result<T, LocalDbError>>>;
        type StoreRequest = (Vec<String>, u64, u64);
        type StoreRequestLog = Rc<RefCell<Vec<StoreRequest>>>;
        type TokenMetadata = Vec<(Address, TokenInfo)>;

        struct StubLocalDb {
            latest_block_result: ResultCell<u64>,
            fetch_events_result: ResultCell<Vec<LogEntryResponse>>,
            decode_events_result: ResultCell<Vec<DecodedEventData<DecodedEvent>>>,
            fetch_store_events_result: ResultCell<Vec<LogEntryResponse>>,
            fetch_token_metadata_result: ResultCell<TokenMetadata>,
            decoded_events_to_sql_result: ResultCell<String>,
            recorded_fetch_events: Rc<RefCell<Vec<(String, u64, u64)>>>,
            recorded_store_requests: StoreRequestLog,
        }

        impl StubLocalDb {
            fn new() -> Self {
                Self {
                    latest_block_result: RefCell::new(Some(Ok(0))),
                    fetch_events_result: RefCell::new(Some(Ok(Vec::new()))),
                    decode_events_result: RefCell::new(Some(Ok(Vec::new()))),
                    fetch_store_events_result: RefCell::new(Some(Ok(Vec::new()))),
                    fetch_token_metadata_result: RefCell::new(Some(Ok(Vec::new()))),
                    decoded_events_to_sql_result: RefCell::new(Some(Ok(String::new()))),
                    recorded_fetch_events: Rc::new(RefCell::new(Vec::new())),
                    recorded_store_requests: Rc::new(RefCell::new(Vec::new())),
                }
            }

            fn set_latest_block_result(&self, result: Result<u64, LocalDbError>) {
                *self.latest_block_result.borrow_mut() = Some(result);
            }

            fn set_fetch_events_result(&self, result: Result<Vec<LogEntryResponse>, LocalDbError>) {
                *self.fetch_events_result.borrow_mut() = Some(result);
            }

            fn set_decode_events_result(
                &self,
                result: Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError>,
            ) {
                *self.decode_events_result.borrow_mut() = Some(result);
            }

            fn set_fetch_store_events_result(
                &self,
                result: Result<Vec<LogEntryResponse>, LocalDbError>,
            ) {
                *self.fetch_store_events_result.borrow_mut() = Some(result);
            }

            fn set_fetch_token_metadata_result(&self, result: Result<TokenMetadata, LocalDbError>) {
                *self.fetch_token_metadata_result.borrow_mut() = Some(result);
            }

            fn set_decoded_events_to_sql_result(&self, result: Result<String, LocalDbError>) {
                *self.decoded_events_to_sql_result.borrow_mut() = Some(result);
            }

            fn recorded_fetch_events(&self) -> Vec<(String, u64, u64)> {
                self.recorded_fetch_events.borrow().clone()
            }

            fn recorded_store_requests(&self) -> Vec<StoreRequest> {
                self.recorded_store_requests.borrow().clone()
            }
        }

        impl LocalDbApi for StubLocalDb {
            fn latest_block_number(&self) -> LocalDbFuture<'_, u64> {
                let result = self
                    .latest_block_result
                    .borrow_mut()
                    .take()
                    .unwrap_or(Ok(0));
                Box::pin(async move { result })
            }

            fn fetch_events(
                &self,
                contract_address: String,
                start_block: u64,
                end_block: u64,
            ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
                self.recorded_fetch_events.borrow_mut().push((
                    contract_address.clone(),
                    start_block,
                    end_block,
                ));
                let result = self
                    .fetch_events_result
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| Ok(Vec::new()));
                Box::pin(async move { result })
            }

            fn decode_events(
                &self,
                _events: &[LogEntryResponse],
            ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
                self.decode_events_result
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| Ok(Vec::new()))
            }

            fn fetch_store_set_events(
                &self,
                store_addresses: Vec<String>,
                start_block: u64,
                end_block: u64,
            ) -> LocalDbFuture<'_, Vec<LogEntryResponse>> {
                self.recorded_store_requests.borrow_mut().push((
                    store_addresses.clone(),
                    start_block,
                    end_block,
                ));
                let result = self
                    .fetch_store_events_result
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| Ok(Vec::new()));
                Box::pin(async move { result })
            }

            fn fetch_token_metadata(
                &self,
                _missing_addrs: Vec<Address>,
            ) -> LocalDbFuture<'_, Vec<(Address, TokenInfo)>> {
                let result = self
                    .fetch_token_metadata_result
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| Ok(Vec::new()));
                Box::pin(async move { result })
            }

            fn decoded_events_to_sql(
                &self,
                _events: &[DecodedEventData<DecodedEvent>],
                _end_block: u64,
                _decimals_by_token: &HashMap<Address, u8>,
                _prefix_sql: Option<&str>,
            ) -> Result<String, LocalDbError> {
                self.decoded_events_to_sql_result
                    .borrow_mut()
                    .take()
                    .unwrap_or_else(|| Ok(String::new()))
            }
        }

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

        fn make_deposit_event(token: Address) -> DecodedEventData<DecodedEvent> {
            let deposit = DepositV2 {
                sender: Address::from([0x01; 20]),
                token,
                vaultId: U256::from(1).into(),
                depositAmountUint256: U256::from(10),
            };

            DecodedEventData {
                event_type: EventType::DepositV2,
                block_number: "0x10".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0xabc".into(),
                log_index: "0x0".into(),
                decoded_data: DecodedEvent::DepositV2(Box::new(deposit)),
            }
        }

        fn make_add_order_event(
            store: Address,
            tokens: Vec<Address>,
        ) -> DecodedEventData<DecodedEvent> {
            let order = OrderV4 {
                owner: Address::from([0x02; 20]),
                nonce: U256::from(1).into(),
                evaluable: EvaluableV4 {
                    interpreter: Address::from([0x03; 20]),
                    store,
                    bytecode: Bytes::from_static(b"\x00"),
                },
                validInputs: tokens
                    .iter()
                    .enumerate()
                    .map(|(idx, token)| IOV2 {
                        token: *token,
                        vaultId: U256::from(idx as u64).into(),
                    })
                    .collect(),
                validOutputs: Vec::new(),
            };
            let add = AddOrderV3 {
                sender: Address::from([0x04; 20]),
                orderHash: FixedBytes::<32>::from([0x05; 32]),
                order,
            };

            DecodedEventData {
                event_type: EventType::AddOrderV3,
                block_number: "0x20".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0xdef".into(),
                log_index: "0x1".into(),
                decoded_data: DecodedEvent::AddOrderV3(Box::new(add)),
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

        #[tokio::test]
        async fn test_sync_database_store_set_flow_generates_sql() {
            let orderbook_addr = Address::from_str(ORDERBOOK_ADDRESS).unwrap();
            let store_addr = Address::from([0x99; 20]);
            let token_a = Address::from([0xaa; 20]);
            let token_b = Address::from([0xbb; 20]);

            let event_logs = vec![
                build_deposit_log(orderbook_addr, Address::from([0x10; 20]), token_a, 12345, 0),
                build_add_order_log(orderbook_addr, store_addr, &[token_b], 12345, 1),
            ];

            let store_logs = vec![build_store_set_log(
                store_addr, [0x11; 32], [0x22; 32], [0x33; 32], 12346, 0,
            )];

            let mut token_metadata = HashMap::new();
            token_metadata.insert(
                token_a,
                TokenInfo {
                    decimals: 18,
                    name: "TokenA".into(),
                    symbol: "TKNA".into(),
                },
            );
            token_metadata.insert(
                token_b,
                TokenInfo {
                    decimals: 6,
                    name: "TokenB".into(),
                    symbol: "TKNB".into(),
                },
            );

            let mock_local_db = MockLocalDb::new(12346, event_logs, store_logs, token_metadata);

            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();

            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let result = sync_database_with_services(
                &client,
                &mock_db,
                &status_sink,
                1,
                Some(&mock_local_db),
            )
            .await;

            assert!(result.is_ok());

            let sql_statements = mock_db.recorded_sql();
            assert_eq!(
                sql_statements.len(),
                1,
                "expected a single SQL execution, got {sql_statements:?}"
            );
            let sql = &sql_statements[0];
            assert!(
                sql.contains("INSERT INTO erc20_tokens"),
                "expected token prefix SQL, got {sql}"
            );
            let sql_lower = sql.to_lowercase();
            assert!(
                sql_lower.contains(&format!("0x{:x}", token_a)),
                "missing tokenA address in SQL"
            );
            assert!(
                sql_lower.contains(&format!("0x{:x}", token_b)),
                "missing tokenB address in SQL"
            );
            assert!(
                sql.contains("interpreter_store_sets"),
                "missing interpreter store set insert"
            );
            assert!(
                sql_lower.contains(&format!("0x{:x}", store_addr)),
                "missing store address in SQL"
            );

            let events = mock_local_db.event_types();
            assert_eq!(
                events,
                vec![
                    EventType::DepositV2,
                    EventType::AddOrderV3,
                    EventType::InterpreterStoreSet
                ]
            );

            let messages = status_sink.messages();
            assert_eq!(
                messages,
                vec![
                    "Starting database sync...",
                    "has tables: true",
                    "Last synced block: 0",
                    "Fetching latest onchain events...",
                    "Decoding fetched events...",
                    "Populating token information...",
                    "Populating database...",
                    "Database sync complete."
                ]
            );
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

        #[tokio::test]
        async fn test_sync_database_initial_sync_uses_deployment_block() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(13000));
            local_db.set_decoded_events_to_sql_result(Ok("/*SQL*/".to_string()));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:4000/sg1",
                    "http://localhost:4000/sg2",
                    "http://localhost:4000/rpc1",
                    "http://localhost:4000/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            assert!(result.is_ok());

            let recorded = local_db.recorded_fetch_events();
            assert_eq!(recorded.len(), 1);
            assert_eq!(
                recorded[0].1, 12345,
                "start block should use deployment block"
            );
            assert_eq!(
                recorded[0].2, 13000,
                "end block should use latest block number"
            );

            let sql = mock_db.recorded_sql();
            assert_eq!(sql, vec!["/*SQL*/".to_string()]);

            let msgs = status_sink.messages();
            assert!(msgs.contains(&"Last synced block: 0".to_string()));
        }

        #[tokio::test]
        async fn test_sync_database_resumes_from_last_synced_plus_one() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let sync_status = vec![SyncStatusResponse {
                id: 1,
                last_synced_block: 15000,
                updated_at: None,
            }];
            let mock_db = MockDatabaseBridge::new(tables, sync_status, Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(15500));
            local_db.set_decoded_events_to_sql_result(Ok("-- sql".to_string()));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:5000/sg1",
                    "http://localhost:5000/sg2",
                    "http://localhost:5000/rpc1",
                    "http://localhost:5000/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            assert!(result.is_ok());

            let recorded = local_db.recorded_fetch_events();
            assert_eq!(recorded.len(), 1);
            assert_eq!(recorded[0].1, 15001);
            assert_eq!(recorded[0].2, 15500);

            let sql = mock_db.recorded_sql();
            assert_eq!(sql, vec!["-- sql".to_string()]);

            let msgs = status_sink.messages();
            assert!(msgs.contains(&"Last synced block: 15000".to_string()));
        }

        #[tokio::test]
        async fn test_sync_database_propagates_fetch_events_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(42));
            local_db.set_fetch_events_result(Err(LocalDbError::CustomError(
                "fetch failed".to_string(),
            )));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6000/sg1",
                    "http://localhost:6000/sg2",
                    "http://localhost:6000/rpc1",
                    "http://localhost:6000/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::FetchEventsFailed(err)) => match *err {
                    LocalDbError::CustomError(message) => {
                        assert!(
                            message.contains("fetch failed"),
                            "unexpected message: {message}"
                        );
                    }
                    other => panic!("expected wrapped CustomError, got {other:?}"),
                },
                other => panic!("expected FetchEventsFailed, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_sync_database_propagates_decode_events_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(100));
            let orderbook_addr = Address::from_str(ORDERBOOK_ADDRESS).unwrap();
            let sender = Address::from([0x10; 20]);
            let token = Address::from([0x20; 20]);
            let log = build_deposit_log(orderbook_addr, sender, token, 100, 0);
            local_db.set_fetch_events_result(Ok(vec![log]));
            local_db.set_decode_events_result(Err(LocalDbError::CustomError(
                "decode failure".to_string(),
            )));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6100/sg1",
                    "http://localhost:6100/sg2",
                    "http://localhost:6100/rpc1",
                    "http://localhost:6100/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::DecodeEventsFailed(err)) => match *err {
                    LocalDbError::CustomError(message) => {
                        assert!(
                            message.contains("decode failure"),
                            "unexpected message: {message}"
                        );
                    }
                    other => panic!("expected wrapped CustomError, got {other:?}"),
                },
                other => panic!("expected DecodeEventsFailed, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_sync_database_propagates_store_event_fetch_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let store_addr = Address::from([0x33; 20]);
            let token = Address::from([0x44; 20]);
            let decoded = make_add_order_event(store_addr, vec![token]);

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(200));
            local_db.set_decode_events_result(Ok(vec![decoded]));
            local_db.set_fetch_store_events_result(Err(LocalDbError::CustomError(
                "store fetch failed".to_string(),
            )));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6200/sg1",
                    "http://localhost:6200/sg2",
                    "http://localhost:6200/rpc1",
                    "http://localhost:6200/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::FetchEventsFailed(err)) => match *err {
                    LocalDbError::CustomError(message) => {
                        assert!(
                            message.contains("store fetch failed"),
                            "unexpected message: {message}"
                        );
                    }
                    other => panic!("expected wrapped CustomError, got {other:?}"),
                },
                other => panic!("expected FetchEventsFailed, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_sync_database_propagates_token_metadata_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let token = Address::from([0x55; 20]);
            let decoded = make_deposit_event(token);

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(300));
            local_db.set_decode_events_result(Ok(vec![decoded]));
            local_db.set_fetch_token_metadata_result(Err(LocalDbError::CustomError(
                "metadata failed".to_string(),
            )));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6300/sg1",
                    "http://localhost:6300/sg2",
                    "http://localhost:6300/rpc1",
                    "http://localhost:6300/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::CustomError(message)) => {
                    assert!(
                        message.contains("metadata failed"),
                        "unexpected message: {message}"
                    );
                }
                other => panic!("expected CustomError, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_sync_database_propagates_sql_generation_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            let status_sink = TestStatusSink::new();

            let token = Address::from([0x66; 20]);
            let decoded = make_deposit_event(token);

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(400));
            local_db.set_decode_events_result(Ok(vec![decoded]));
            local_db.set_fetch_token_metadata_result(Ok(vec![(
                token,
                TokenInfo {
                    decimals: 18,
                    name: "TokenX".into(),
                    symbol: "TKX".into(),
                },
            )]));
            local_db.set_decoded_events_to_sql_result(Err(LocalDbError::CustomError(
                "sql generation failed".to_string(),
            )));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6400/sg1",
                    "http://localhost:6400/sg2",
                    "http://localhost:6400/rpc1",
                    "http://localhost:6400/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::SqlGenerationFailed(err)) => match *err {
                    LocalDbError::CustomError(message) => {
                        assert!(
                            message.contains("sql generation failed"),
                            "unexpected message: {message}"
                        );
                    }
                    other => panic!("expected wrapped CustomError, got {other:?}"),
                },
                other => panic!("expected SqlGenerationFailed, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_sync_database_propagates_database_execution_error() {
            let tables: Vec<TableResponse> = REQUIRED_TABLES
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect();
            let mock_db = MockDatabaseBridge::new(tables, Vec::new(), Vec::new(), Vec::new());
            mock_db.set_execute_result(Err(LocalDbQueryError::DatabaseError {
                message: "sqlite busy".to_string(),
            }));
            let status_sink = TestStatusSink::new();

            let token = Address::from([0x77; 20]);
            let decoded = make_deposit_event(token);

            let local_db = StubLocalDb::new();
            local_db.set_latest_block_result(Ok(500));
            local_db.set_decode_events_result(Ok(vec![decoded]));
            local_db.set_fetch_token_metadata_result(Ok(vec![(
                token,
                TokenInfo {
                    decimals: 18,
                    name: "TokenY".into(),
                    symbol: "TKY".into(),
                },
            )]));
            local_db.set_decoded_events_to_sql_result(Ok("INSERT".to_string()));

            let client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://localhost:6500/sg1",
                    "http://localhost:6500/sg2",
                    "http://localhost:6500/rpc1",
                    "http://localhost:6500/rpc2",
                )],
                None,
            )
            .unwrap();

            let result =
                sync_database_with_services(&client, &mock_db, &status_sink, 1, Some(&local_db))
                    .await;

            match result {
                Err(LocalDbError::LocalDbQueryError(LocalDbQueryError::DatabaseError {
                    message,
                })) => {
                    assert!(
                        message.contains("sqlite busy"),
                        "unexpected message: {message}"
                    );
                }
                other => panic!("expected LocalDbQueryError::DatabaseError, got {other:?}"),
            }
        }
    }
}
