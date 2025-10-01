use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeSet;

use rain_orderbook_common::raindex_client::local_db::tokens::collect_store_addresses;

use super::{data_source::SyncDataSource, storage::fetch_existing_store_addresses};

pub(crate) fn collect_all_store_addresses(
    db_path: &str,
    decoded_events: &Value,
) -> Result<Vec<String>> {
    let mut addresses: BTreeSet<String> = collect_store_addresses(decoded_events);
    let existing = fetch_existing_store_addresses(db_path)?;
    addresses.extend(existing);

    Ok(addresses.into_iter().collect())
}

pub(crate) struct StoreFetchStats {
    pub(crate) fetched_raw_count: usize,
    pub(crate) decoded_count: usize,
    pub(crate) total_decoded_count: usize,
}

pub(crate) struct StoreFetchOutcome {
    pub(crate) events: Value,
    pub(crate) stats: StoreFetchStats,
}

pub(crate) async fn fetch_and_merge_store_events<D>(
    data_source: &D,
    decoded_events: Value,
    store_addresses: &[String],
    start_block: u64,
    target_block: u64,
) -> Result<StoreFetchOutcome>
where
    D: SyncDataSource + Send + Sync,
{
    if store_addresses.is_empty() {
        let total_decoded_count = decoded_events.as_array().map(|a| a.len()).unwrap_or(0);
        return Ok(StoreFetchOutcome {
            events: decoded_events,
            stats: StoreFetchStats {
                fetched_raw_count: 0,
                decoded_count: 0,
                total_decoded_count,
            },
        });
    }

    let store_events = data_source
        .fetch_store_set_events(store_addresses, start_block, target_block)
        .await?;
    let raw_count = store_events.as_array().map(|a| a.len()).unwrap_or(0);

    let decoded_store = data_source.decode_events(store_events)?;
    let decoded_count = decoded_store.as_array().map(|a| a.len()).unwrap_or(0);

    let merged_events = if decoded_count == 0 {
        decoded_events
    } else if let (Some(mut base_events), Some(store_array)) =
        (decoded_events.as_array().cloned(), decoded_store.as_array())
    {
        base_events.extend(store_array.iter().cloned());
        Value::Array(base_events)
    } else {
        decoded_events
    };

    let total_decoded_count = merged_events.as_array().map(|a| a.len()).unwrap_or(0);

    Ok(StoreFetchOutcome {
        events: merged_events,
        stats: StoreFetchStats {
            fetched_raw_count: raw_count,
            decoded_count,
            total_decoded_count,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use url::Url;

    use crate::commands::local_db::sqlite::sqlite_execute;

    use super::super::storage::DEFAULT_SCHEMA_SQL;

    struct MockDataSource {
        store_events: Value,
        decoded_store: Value,
        rpc_urls: Vec<Url>,
        calls: Mutex<Vec<(Vec<String>, u64, u64)>>,
    }

    #[async_trait]
    impl SyncDataSource for MockDataSource {
        async fn latest_block(&self) -> anyhow::Result<u64> {
            Ok(0)
        }

        async fn fetch_events(&self, _: &str, _: u64, _: u64) -> anyhow::Result<Value> {
            Ok(Value::Array(vec![]))
        }

        async fn fetch_store_set_events(
            &self,
            store_addresses: &[String],
            start_block: u64,
            end_block: u64,
        ) -> anyhow::Result<Value> {
            self.calls
                .lock()
                .unwrap()
                .push((store_addresses.to_vec(), start_block, end_block));
            Ok(self.store_events.clone())
        }

        fn decode_events(&self, events: Value) -> anyhow::Result<Value> {
            if events == self.store_events {
                Ok(self.decoded_store.clone())
            } else {
                Ok(events)
            }
        }

        fn events_to_sql(&self, _: Value, _: u64, _: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }

        fn rpc_urls(&self) -> &[Url] {
            &self.rpc_urls
        }
    }

    #[tokio::test]
    async fn merge_includes_store_events() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("store.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();
        sqlite_execute(
            &db_path_str,
            "INSERT INTO interpreter_store_sets (store_address, transaction_hash, log_index, block_number, block_timestamp, namespace, key, value) VALUES ('0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', '0x1', 0, 1, 0, '0x0', '0x0', '0x0');",
        )
        .unwrap();

        let decoded = json!([
            {
                "event_type": "AddOrderV3",
                "decoded_data": {
                    "order": {
                        "evaluable": {"store": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}
                    },
                    "valid_inputs": [],
                    "valid_outputs": []
                }
            }
        ]);

        let store_events = json!([
            {
                "blockNumber": "0x1",
                "data": "0x",
                "topics": []
            }
        ]);
        let decoded_store = json!([
            {
                "event_type": "Set",
                "decoded_data": {
                    "namespace": "0x0",
                    "key": "0x0",
                    "value": "0x0"
                }
            }
        ]);

        let data_source = MockDataSource {
            store_events: store_events.clone(),
            decoded_store: decoded_store.clone(),
            rpc_urls: Vec::new(),
            calls: Mutex::new(Vec::new()),
        };

        let addresses = collect_all_store_addresses(&db_path_str, &decoded).unwrap();
        assert_eq!(addresses.len(), 2);
        assert!(addresses
            .iter()
            .any(|addr| addr == "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(addresses
            .iter()
            .any(|addr| addr == "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));

        let outcome =
            fetch_and_merge_store_events(&data_source, decoded.clone(), &addresses, 10, 20)
                .await
                .unwrap();

        let calls = data_source.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, addresses);
        assert_eq!(calls[0].1, 10);
        assert_eq!(calls[0].2, 20);

        assert_eq!(outcome.stats.fetched_raw_count, 1);
        assert_eq!(outcome.stats.decoded_count, 1);
        assert_eq!(outcome.stats.total_decoded_count, 2);
        assert_eq!(
            outcome.events.as_array().unwrap().len(),
            outcome.stats.total_decoded_count
        );
    }

    #[tokio::test]
    async fn merge_bails_on_empty_store_events() {
        let decoded = json!([]);
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("empty.db");
        let db_path_str = db_path.to_string_lossy();

        sqlite_execute(&db_path_str, DEFAULT_SCHEMA_SQL).unwrap();

        let data_source = MockDataSource {
            store_events: json!([]),
            decoded_store: json!([]),
            rpc_urls: Vec::new(),
            calls: Mutex::new(Vec::new()),
        };

        let addresses = collect_all_store_addresses(&db_path_str, &decoded).unwrap();
        assert!(addresses.is_empty());

        let outcome = fetch_and_merge_store_events(&data_source, decoded.clone(), &[], 0, 0)
            .await
            .unwrap();

        assert_eq!(outcome.stats.fetched_raw_count, 0);
        assert_eq!(outcome.stats.decoded_count, 0);
        assert_eq!(outcome.stats.total_decoded_count, 0);
        assert_eq!(outcome.events, decoded);

        assert!(data_source.calls.lock().unwrap().is_empty());
    }
}
