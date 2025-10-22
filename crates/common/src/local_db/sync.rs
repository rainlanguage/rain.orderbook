use super::{
    decode::{DecodedEvent, DecodedEventData},
    insert,
    query::{
        create_tables::REQUIRED_TABLES,
        fetch_erc20_tokens_by_addresses::{build_fetch_stmt as build_token_stmt, Erc20TokenRow},
        fetch_last_synced_block::{fetch_last_synced_block_stmt, SyncStatusResponse},
        fetch_store_addresses::{fetch_store_addresses_stmt, StoreAddressRow},
        fetch_tables::{fetch_tables_stmt, TableResponse},
        update_last_synced_block::build_update_last_synced_block_stmt,
        LocalDbQueryError, SqlStatement,
    },
    token_fetch::fetch_erc20_metadata_concurrent,
    tokens::{collect_store_addresses, collect_token_addresses},
    FetchConfig, LocalDb, LocalDbError,
};
use crate::rpc_client::BlockRange;
use alloy::primitives::Address;
use flate2::read::GzDecoder;
use rain_orderbook_app_settings::orderbook::OrderbookCfg;
use reqwest::Client;
use std::collections::BTreeSet;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    str::FromStr,
};

pub const DUMP_URL: &str = "https://raw.githubusercontent.com/rainlanguage/rain.strategies/3d6deafeaa52525d56d89641c0cb3c997923ad21/local_db.sql.gz";

use crate::local_db::query::LocalDbQueryExecutor;

pub trait StatusSink {
    fn send(&self, message: String) -> Result<(), LocalDbError>;
}

pub async fn sync_database_with_services<D: LocalDbQueryExecutor, S: StatusSink>(
    orderbook_cfg: &OrderbookCfg,
    db: &D,
    status: &S,
) -> Result<(), LocalDbError> {
    status.send("Starting database sync...".to_string())?;

    let has_tables = check_required_tables(db)
        .await
        .map_err(LocalDbError::TableCheckFailed)?;
    status.send(format!("has tables: {}", has_tables))?;

    if !has_tables {
        status.send("Initializing database tables and importing data...".to_string())?;
        let dump_sql = download_and_decompress_dump().await?;
        db.query_text(&SqlStatement::new(dump_sql))
            .await
            .map_err(LocalDbError::from)?;
    }

    let last_synced_block = get_last_synced_block(db)
        .await
        .map_err(LocalDbError::SyncStatusReadFailed)?;
    status.send(format!("Last synced block: {}", last_synced_block))?;

    let chain_id = orderbook_cfg.network.chain_id;
    let local_db = LocalDb::new_with_regular_rpcs(orderbook_cfg.network.rpcs.clone())?;

    let latest_block = local_db
        .rpc_client()
        .get_latest_block_number()
        .await
        .map_err(LocalDbError::from)?;

    let start_block = if last_synced_block == 0 {
        orderbook_cfg.deployment_block
    } else {
        last_synced_block.saturating_add(1)
    };

    let range = BlockRange::inclusive(start_block, latest_block)?;

    status.send("Fetching latest onchain events...".to_string())?;
    let events = local_db
        .fetch_orderbook_events(orderbook_cfg.address, range, &FetchConfig::default())
        .await
        .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

    status.send("Decoding fetched events...".to_string())?;
    let mut decoded_events = local_db
        .decode_events(&events)
        .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

    let existing_stores: Vec<StoreAddressRow> = db
        .query_json(&fetch_store_addresses_stmt())
        .await
        .map_err(LocalDbError::from)?;
    let store_addresses_vec = collect_all_store_addresses(&decoded_events, &existing_stores);
    let store_addresses: Vec<Address> = store_addresses_vec
        .iter()
        .map(|s| Address::from_str(s))
        .collect::<Result<_, _>>()?;

    let store_logs = local_db
        .fetch_store_events(&store_addresses, range, &FetchConfig::default())
        .await
        .map_err(|e| LocalDbError::FetchEventsFailed(Box::new(e)))?;

    let mut decoded_store_events = local_db
        .decode_events(&store_logs)
        .map_err(|e| LocalDbError::DecodeEventsFailed(Box::new(e)))?;

    merge_store_events(&mut decoded_events, &mut decoded_store_events)?;

    status.send("Populating token information...".to_string())?;
    let prep = prepare_erc20_tokens_prefix(
        db,
        &local_db,
        chain_id,
        &decoded_events,
        &FetchConfig::default(),
    )
    .await?;

    status.send("Populating database...".to_string())?;
    let prefix_sql = if prep.tokens_prefix_sql.is_empty() {
        None
    } else {
        Some(prep.tokens_prefix_sql.as_str())
    };

    let sql_batch = local_db
        .decoded_events_to_statement(
            &decoded_events,
            latest_block,
            &prep.decimals_by_addr,
            prefix_sql,
        )
        .map_err(|e| LocalDbError::SqlGenerationFailed(Box::new(e)))?
        .into_transaction()
        .map_err(|err| {
            LocalDbError::SqlGenerationFailed(Box::new(LocalDbError::InsertError {
                message: err.to_string(),
            }))
        })?;

    db.execute_batch(&sql_batch)
        .await
        .map_err(LocalDbError::from)?;

    let update_stmt = build_update_last_synced_block_stmt(latest_block);
    db.query_text(&update_stmt)
        .await
        .map_err(LocalDbError::from)?;

    status.send("Database sync complete.".to_string())?;
    Ok(())
}

async fn check_required_tables(db: &impl LocalDbQueryExecutor) -> Result<bool, LocalDbQueryError> {
    let tables: Vec<TableResponse> = db.query_json(&fetch_tables_stmt()).await?;
    let existing_table_names: HashSet<String> = tables
        .into_iter()
        .map(|t| t.name.to_ascii_lowercase())
        .collect();

    let has_all = REQUIRED_TABLES
        .iter()
        .all(|&table| existing_table_names.contains(&table.to_ascii_lowercase()));

    Ok(has_all)
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

async fn get_last_synced_block(db: &impl LocalDbQueryExecutor) -> Result<u64, LocalDbQueryError> {
    let results: Vec<SyncStatusResponse> = db.query_json(&fetch_last_synced_block_stmt()).await?;
    Ok(results.first().map(|r| r.last_synced_block).unwrap_or(0))
}

fn collect_all_store_addresses(
    decoded_events: &[DecodedEventData<DecodedEvent>],
    existing_stores: &[StoreAddressRow],
) -> Vec<String> {
    let mut store_addresses: BTreeSet<String> = collect_store_addresses(decoded_events)
        .into_iter()
        .map(|addr| addr.to_ascii_lowercase())
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
) -> Result<(), LocalDbError> {
    if store_events.is_empty() {
        return Ok(());
    }

    decoded_events.append(store_events);
    sort_events_by_block_and_log(decoded_events)?;
    Ok(())
}

struct TokenPrepResult {
    tokens_prefix_sql: String,
    decimals_by_addr: HashMap<Address, u8>,
}

async fn prepare_erc20_tokens_prefix(
    db: &impl LocalDbQueryExecutor,
    local_db: &LocalDb,
    chain_id: u32,
    decoded_events: &[DecodedEventData<DecodedEvent>],
    config: &FetchConfig,
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

        let existing_rows: Vec<Erc20TokenRow> = if let Some(stmt) =
            build_token_stmt(chain_id, &addr_strings)
                .map_err(|e| LocalDbError::CustomError(e.to_string()))?
        {
            db.query_json(&stmt).await.map_err(LocalDbError::from)?
        } else {
            Vec::new()
        };

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
            let successes = fetch_erc20_metadata_concurrent(rpcs, missing_addrs, config).await?;

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

fn sort_events_by_block_and_log(
    events: &mut [DecodedEventData<DecodedEvent>],
) -> Result<(), LocalDbError> {
    // Parse indices and associated numeric keys once, avoiding unwraps.
    let mut keyed_indices: Vec<(usize, u64, u64)> = Vec::with_capacity(events.len());
    for (idx, e) in events.iter().enumerate() {
        let block = parse_u64_hex_or_dec(&e.block_number).map_err(|err| {
            LocalDbError::CustomError(format!(
                "failed to parse block_number '{}': {}",
                e.block_number, err
            ))
        })?;
        let log = parse_u64_hex_or_dec(&e.log_index).map_err(|err| {
            LocalDbError::CustomError(format!(
                "failed to parse log_index '{}': {}",
                e.log_index, err
            ))
        })?;
        keyed_indices.push((idx, block, log));
    }

    keyed_indices.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.2.cmp(&b.2)));

    // Reorder events according to the sorted indices; Clone is available.
    let original = events.to_vec();
    for (pos, (idx, _, _)) in keyed_indices.into_iter().enumerate() {
        events[pos] = original[idx].clone();
    }

    Ok(())
}

fn parse_u64_hex_or_dec(value: &str) -> Result<u64, std::num::ParseIntError> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::decode::{DecodedEvent, DecodedEventData, EventType, UnknownEventDecoded};
    use crate::local_db::query::FromDbJson;
    use crate::local_db::query::{
        create_tables::REQUIRED_TABLES,
        fetch_last_synced_block::{fetch_last_synced_block_stmt, SyncStatusResponse},
        fetch_tables::{fetch_tables_stmt, TableResponse},
        LocalDbQueryError, SqlStatementBatch,
    };
    use async_trait::async_trait;

    struct MockDb {
        json_map: std::collections::HashMap<String, String>,
        text_map: std::collections::HashMap<String, String>,
        err_on: std::collections::HashSet<String>,
        // no pattern map; tests use exact SQL
    }

    impl MockDb {
        fn new() -> Self {
            Self {
                json_map: Default::default(),
                text_map: Default::default(),
                err_on: Default::default(),
            }
        }

        fn with_json(mut self, sql: &str, value: &str) -> Self {
            self.json_map.insert(sql.to_string(), value.to_string());
            self
        }

        fn with_error(mut self, sql: &str) -> Self {
            self.err_on.insert(sql.to_string());
            self
        }

        // no with_json_contains; tests generate exact SQL
    }

    #[async_trait(?Send)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            for stmt in batch {
                let _ = self.query_text(stmt).await?;
            }
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = &stmt.sql;
            if self.err_on.contains(sql) {
                return Err(LocalDbQueryError::database("forced error"));
            }
            let Some(body) = self.json_map.get(sql) else {
                return Err(LocalDbQueryError::database("no json for sql"));
            };
            serde_json::from_str::<T>(body)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = &stmt.sql;
            if self.err_on.contains(sql) {
                return Err(LocalDbQueryError::database("forced error"));
            }
            let Some(body) = self.text_map.get(sql) else {
                return Err(LocalDbQueryError::database("no text for sql"));
            };
            Ok(body.clone())
        }
    }

    #[tokio::test]
    async fn test_check_required_tables_all_exist() {
        let table_data: Vec<TableResponse> = REQUIRED_TABLES
            .iter()
            .map(|&name| TableResponse {
                name: name.to_string(),
            })
            .collect();
        let json_data = serde_json::to_string(&table_data).unwrap();
        let db = MockDb::new().with_json(&fetch_tables_stmt().sql, &json_data);

        let has_tables = check_required_tables(&db).await.unwrap();
        assert!(has_tables);
    }

    #[tokio::test]
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
        let db = MockDb::new().with_json(&fetch_tables_stmt().sql, &json_data);

        let has_tables = check_required_tables(&db).await.unwrap();
        assert!(!has_tables);
    }

    #[tokio::test]
    async fn test_check_required_tables_empty_db() {
        let db = MockDb::new().with_json(&fetch_tables_stmt().sql, "[]");
        let has_tables = check_required_tables(&db).await.unwrap();
        assert!(!has_tables);
    }

    #[tokio::test]
    async fn test_check_required_tables_query_fails() {
        let db = MockDb::new().with_error(&fetch_tables_stmt().sql);
        let err = check_required_tables(&db).await.err().unwrap();
        match err {
            LocalDbQueryError::Database { .. } => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[tokio::test]
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
        let db = MockDb::new().with_json(&fetch_tables_stmt().sql, &json_data);
        let has_tables = check_required_tables(&db).await.unwrap();
        assert!(has_tables);
    }

    #[tokio::test]
    async fn test_get_last_synced_block_exists() {
        let sync_data = vec![SyncStatusResponse {
            id: 1,
            last_synced_block: 12345,
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
        }];
        let db = MockDb::new().with_json(
            &fetch_last_synced_block_stmt().sql,
            &serde_json::to_string(&sync_data).unwrap(),
        );
        let val = get_last_synced_block(&db).await.unwrap();
        assert_eq!(val, 12345);
    }

    #[tokio::test]
    async fn test_get_last_synced_block_empty() {
        let db = MockDb::new().with_json(&fetch_last_synced_block_stmt().sql, "[]");
        let val = get_last_synced_block(&db).await.unwrap();
        assert_eq!(val, 0);
    }

    #[tokio::test]
    async fn test_get_last_synced_block_query_fails() {
        let db = MockDb::new().with_error(&fetch_last_synced_block_stmt().sql);
        let err = get_last_synced_block(&db).await.err().unwrap();
        match err {
            LocalDbQueryError::Database { .. } => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn test_parse_u64_hex_or_dec_variants() {
        assert_eq!(parse_u64_hex_or_dec("0x0").unwrap(), 0);
        assert_eq!(parse_u64_hex_or_dec("0x1a").unwrap(), 26);
        assert_eq!(parse_u64_hex_or_dec("26").unwrap(), 26);
        assert!(parse_u64_hex_or_dec("garbage").is_err());
        assert_eq!(parse_u64_hex_or_dec("  0x2A  ").unwrap(), 42);
    }

    #[test]
    fn test_sort_events_by_block_and_log() {
        let mut events = vec![
            DecodedEventData {
                event_type: EventType::Unknown,
                block_number: "0x2".to_string(),
                block_timestamp: "0x0".to_string(),
                transaction_hash: "0x1".to_string(),
                log_index: "0x1".to_string(),
                decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".to_string(),
                    note: "".to_string(),
                }),
            },
            DecodedEventData {
                event_type: EventType::Unknown,
                block_number: "0x1".to_string(),
                block_timestamp: "0x0".to_string(),
                transaction_hash: "0x2".to_string(),
                log_index: "0x2".to_string(),
                decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".to_string(),
                    note: "".to_string(),
                }),
            },
            DecodedEventData {
                event_type: EventType::Unknown,
                block_number: "0x1".to_string(),
                block_timestamp: "0x0".to_string(),
                transaction_hash: "0x3".to_string(),
                log_index: "0x1".to_string(),
                decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".to_string(),
                    note: "".to_string(),
                }),
            },
        ];
        sort_events_by_block_and_log(&mut events).unwrap();
        assert_eq!(events[0].transaction_hash, "0x3");
        assert_eq!(events[1].transaction_hash, "0x2");
        assert_eq!(events[2].transaction_hash, "0x1");
    }

    #[test]
    fn test_merge_store_events_merges_and_sorts() {
        let mut decoded = vec![
            DecodedEventData {
                event_type: EventType::Unknown,
                block_number: "0x2".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0xA".into(),
                log_index: "0x1".into(),
                decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".into(),
                    note: "".into(),
                }),
            },
            DecodedEventData {
                event_type: EventType::Unknown,
                block_number: "0x1".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0xB".into(),
                log_index: "0x2".into(),
                decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: "0x".into(),
                    note: "".into(),
                }),
            },
        ];
        let mut store = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "0x1".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0xC".into(),
            log_index: "0x1".into(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".into(),
                note: "".into(),
            }),
        }];

        merge_store_events(&mut decoded, &mut store).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].transaction_hash, "0xC");
        assert_eq!(decoded[1].transaction_hash, "0xB");
        assert_eq!(decoded[2].transaction_hash, "0xA");
    }

    #[test]
    fn test_sort_events_by_block_and_log_returns_error_on_bad_block() {
        let mut events = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "0xZZ".to_string(),
            block_timestamp: "0x0".to_string(),
            transaction_hash: "0x1".to_string(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".to_string(),
                note: "".to_string(),
            }),
        }];
        let err = sort_events_by_block_and_log(&mut events).unwrap_err();
        match err {
            LocalDbError::CustomError(msg) => assert!(msg.contains("failed to parse block_number")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_sort_events_by_block_and_log_returns_error_on_bad_log_index() {
        let mut events = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "0x1".to_string(),
            block_timestamp: "0x0".to_string(),
            transaction_hash: "0x1".to_string(),
            log_index: "not-a-number".to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".to_string(),
                note: "".to_string(),
            }),
        }];
        let err = sort_events_by_block_and_log(&mut events).unwrap_err();
        match err {
            LocalDbError::CustomError(msg) => assert!(msg.contains("failed to parse log_index")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_sort_events_by_block_and_log_returns_error_on_bad_decimal_block() {
        let mut events = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "123x".to_string(),
            block_timestamp: "0x0".to_string(),
            transaction_hash: "0x1".to_string(),
            log_index: "0x0".to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".to_string(),
                note: "".to_string(),
            }),
        }];
        let err = sort_events_by_block_and_log(&mut events).unwrap_err();
        match err {
            LocalDbError::CustomError(msg) => assert!(msg.contains("failed to parse block_number")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_merge_store_events_propagates_parse_error() {
        let mut decoded = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "0x1".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0xA".into(),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".into(),
                note: "".into(),
            }),
        }];
        let mut store = vec![DecodedEventData {
            event_type: EventType::Unknown,
            block_number: "garbage".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0xB".into(),
            log_index: "0x1".into(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".into(),
                note: "".into(),
            }),
        }];

        let err = merge_store_events(&mut decoded, &mut store).unwrap_err();
        match err {
            LocalDbError::CustomError(msg) => assert!(msg.contains("failed to parse block_number")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_collect_all_store_addresses_dedupe_merge() {
        use crate::local_db::query::fetch_store_addresses::StoreAddressRow;
        use alloy::primitives::Address;

        let store_addr_event = Address::from([0x11u8; 20]);
        let decoded_events = vec![DecodedEventData {
            event_type: EventType::InterpreterStoreSet,
            block_number: "0x1".into(),
            block_timestamp: "0x0".into(),
            transaction_hash: "0x0".into(),
            log_index: "0x0".into(),
            decoded_data: DecodedEvent::InterpreterStoreSet(Box::new(
                crate::local_db::decode::InterpreterStoreSetEvent {
                    store_address: store_addr_event,
                    namespace: alloy::primitives::FixedBytes::from([0u8; 32]),
                    key: alloy::primitives::FixedBytes::from([0u8; 32]),
                    value: alloy::primitives::FixedBytes::from([0u8; 32]),
                },
            )),
        }];

        let existing = vec![
            StoreAddressRow {
                store_address: format!("0x{:X}", Address::from([0x22u8; 20])),
            },
            StoreAddressRow {
                // duplicate of event address but different case to test normalization
                store_address: format!("0x{:X}", store_addr_event),
            },
        ];

        let out = collect_all_store_addresses(&decoded_events, &existing);
        let set: std::collections::HashSet<_> = out.iter().cloned().collect();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&format!("0x{:x}", store_addr_event)));
        assert!(set.contains(&format!("0x{:x}", Address::from([0x22u8; 20]))));
    }

    #[cfg(not(target_family = "wasm"))]
    mod prepare_tokens_tests {
        use super::*;
        use crate::local_db::query::fetch_erc20_tokens_by_addresses::{
            build_fetch_stmt, Erc20TokenRow,
        };
        use alloy::primitives::Address;
        use rain_orderbook_bindings::IOrderBookV5::DepositV2;
        use rain_orderbook_test_fixtures::LocalEvm;
        use url::Url;

        fn build_deposit_event(token: Address) -> DecodedEventData<DecodedEvent> {
            use alloy::primitives::U256;
            DecodedEventData {
                event_type: EventType::DepositV2,
                block_number: "0x1".into(),
                block_timestamp: "0x0".into(),
                transaction_hash: "0x0".into(),
                log_index: "0x0".into(),
                decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                    sender: Address::from([0u8; 20]),
                    token,
                    vaultId: U256::from(1).into(),
                    depositAmountUint256: U256::from(1),
                })),
            }
        }

        #[tokio::test]
        async fn test_prepare_tokens_existing_only() {
            let chain_id = 9999u32;
            let token = Address::from([0xAAu8; 20]);
            let events = vec![build_deposit_event(token)];

            // DB returns existing row for the token query
            let row = Erc20TokenRow {
                chain_id,
                address: format!("0x{:x}", token),
                name: "Foo".into(),
                symbol: "FOO".into(),
                decimals: 6,
            };
            let stmt = build_fetch_stmt(chain_id, &[format!("0x{:x}", token)])
                .expect("stmt")
                .expect("some");
            let db = MockDb::new().with_json(
                &stmt.sql,
                &serde_json::to_string(&vec![row.clone()]).unwrap(),
            );

            // LocalDb not used on existing-only path, but needs to exist
            let local_db =
                LocalDb::new_with_regular_rpcs(vec![]).unwrap_or_else(|_| LocalDb::default());

            let out = prepare_erc20_tokens_prefix(
                &db,
                &local_db,
                chain_id,
                &events,
                &FetchConfig::default(),
            )
            .await
            .unwrap();
            assert!(out.tokens_prefix_sql.is_empty());
            assert_eq!(out.decimals_by_addr.get(&token), Some(&6));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_prepare_tokens_fetch_missing_and_build_sql() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let rpc_url = Url::parse(&local_evm.url()).unwrap();

            // Use LocalDb that points to the local EVM so metadata fetch succeeds
            let local_db = LocalDb::new_with_url(rpc_url);

            // Use the fixture token address from the local EVM for the event
            let token = *local_evm.tokens[0].address();
            let events = vec![build_deposit_event(token)];

            // DB query returns empty so the token is considered missing
            let stmt = build_fetch_stmt(1, &[format!("0x{:x}", token)])
                .expect("stmt")
                .expect("some");
            let db = MockDb::new().with_json(&stmt.sql, "[]");

            let out =
                prepare_erc20_tokens_prefix(&db, &local_db, 1, &events, &FetchConfig::default())
                    .await
                    .unwrap();
            assert!(!out.tokens_prefix_sql.is_empty());
            assert!(out.tokens_prefix_sql.contains("erc20_tokens"));
            assert!(out.tokens_prefix_sql.contains(&format!("0x{:x}", token)));
            assert_eq!(out.decimals_by_addr.get(&token), Some(&18));
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod http_dump_tests {
        use super::*;
        use httpmock::prelude::*;

        #[tokio::test]
        async fn test_download_and_decompress_http_404() {
            let server = MockServer::start();
            let _m = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(404);
            });

            // replicate body with custom URL
            let result = async {
                let client = reqwest::Client::new();
                let response = client.get(server.url("/")).send().await?;
                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();
                let mut decoder = flate2::read::GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                use std::io::Read;
                decoder.read_to_string(&mut decompressed)?;
                Ok::<String, LocalDbError>(decompressed)
            }
            .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to download dump, status: 404"));
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_download_and_decompress_http_500() {
            let server = MockServer::start();
            let _m = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(500);
            });

            let result = async {
                let client = reqwest::Client::new();
                let response = client.get(server.url("/")).send().await?;
                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();
                let mut decoder = flate2::read::GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                use std::io::Read;
                decoder.read_to_string(&mut decompressed)?;
                Ok::<String, LocalDbError>(decompressed)
            }
            .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::CustomError(msg) => {
                    assert!(msg.contains("Failed to download dump, status: 500"));
                }
                other => panic!("unexpected error variant: {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_download_and_decompress_invalid_gzip() {
            let server = MockServer::start();
            let _m = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(200)
                    .header("content-type", "application/gzip")
                    .body("not-gzip");
            });

            let result = async {
                let client = reqwest::Client::new();
                let response = client.get(server.url("/")).send().await?;
                if !response.status().is_success() {
                    return Err(LocalDbError::CustomError(format!(
                        "Failed to download dump, status: {}",
                        response.status()
                    )));
                }
                let response = response.bytes().await?.to_vec();
                let mut decoder = flate2::read::GzDecoder::new(response.as_slice());
                let mut decompressed = String::new();
                use std::io::Read;
                decoder.read_to_string(&mut decompressed)?;
                Ok::<String, LocalDbError>(decompressed)
            }
            .await;

            assert!(result.is_err());
            match result.unwrap_err() {
                LocalDbError::IoError(_) => {}
                other => panic!("unexpected error variant: {other:?}"),
            }
        }
    }
}
