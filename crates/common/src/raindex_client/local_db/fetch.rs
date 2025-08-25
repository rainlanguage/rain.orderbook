use crate::hyper_rpc::{fetch_block_timestamps, HyperRpcClient};
use alloy::{primitives::U256, sol_types::SolEvent};
use futures::StreamExt;
use rain_orderbook_bindings::{
    IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    },
    OrderBook::MetaV1_2,
};
use std::collections::HashSet;

pub async fn fetch_url(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP request failed with status: {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

async fn backfill_missing_timestamps(
    events: &mut serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let events_array = match events.as_array_mut() {
        Some(array) => array,
        None => return Err("Events is not an array".into()),
    };

    // Find events missing blockTimestamp and collect unique block numbers
    let mut missing_blocks = HashSet::new();

    for event in events_array.iter() {
        if event.get("blockTimestamp").is_none() {
            if let Some(block_number_hex) = event.get("blockNumber").and_then(|v| v.as_str()) {
                if let Ok(block_u256) = block_number_hex.parse::<U256>() {
                    missing_blocks.insert(block_u256.to::<u64>());
                }
            }
        }
    }

    if missing_blocks.is_empty() {
        return Ok(());
    }

    // Fetch timestamps for missing blocks
    let block_numbers: Vec<u64> = missing_blocks.into_iter().collect();
    let timestamps = fetch_block_timestamps(block_numbers).await?;

    // Inject timestamps into events
    for event in events_array.iter_mut() {
        if event.get("blockTimestamp").is_none() {
            if let Some(block_number_hex) = event.get("blockNumber").and_then(|v| v.as_str()) {
                if let Ok(block_u256) = block_number_hex.parse::<U256>() {
                    let block_number = block_u256.to::<u64>();
                    if let Some(timestamp) = timestamps.get(&block_number) {
                        if let Some(event_obj) = event.as_object_mut() {
                            event_obj.insert(
                                "blockTimestamp".to_string(),
                                serde_json::Value::String(timestamp.clone()),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn fetch_events(
    contract_address: &str,
    start_block: u64,
    end_block: u64,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let chunk_size = 5000u64; // Balance between performance and reliability

    // Use multiple event signatures for filtering
    let topics = Some(vec![Some(vec![
        AddOrderV3::SIGNATURE_HASH.to_string(),
        TakeOrderV3::SIGNATURE_HASH.to_string(),
        WithdrawV2::SIGNATURE_HASH.to_string(),
        DepositV2::SIGNATURE_HASH.to_string(),
        RemoveOrderV3::SIGNATURE_HASH.to_string(),
        ClearV3::SIGNATURE_HASH.to_string(),
        AfterClearV2::SIGNATURE_HASH.to_string(),
        MetaV1_2::SIGNATURE_HASH.to_string(),
    ])]);

    // Prepare all chunk ranges
    let mut chunks = Vec::new();
    let mut current_block = start_block;
    while current_block <= end_block {
        let to_block = std::cmp::min(current_block + chunk_size - 1, end_block);
        chunks.push((current_block, to_block));
        current_block = to_block + 1;
    }

    // Process chunks with concurrency limit to avoid timeouts
    let results: Vec<Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>>> =
        futures::stream::iter(chunks)
            .map(|(from_block, to_block)| {
                let client = HyperRpcClient {};
                let topics = topics.clone();
                let contract_address = contract_address.to_string();

                async move {
                    let from_block_hex = format!("0x{:x}", from_block);
                    let to_block_hex = format!("0x{:x}", to_block);

                    // Retry logic for failed requests
                    let mut result: Result<String, Box<dyn std::error::Error + Send + Sync>> =
                        Err("Not attempted".into());
                    for _attempt in 1..=3 {
                        result = client
                            .get_logs(
                                &from_block_hex,
                                &to_block_hex,
                                &contract_address,
                                topics.clone(),
                            )
                            .await;

                        if result.is_ok() {
                            break;
                        }

                        // Note: removed sleep for WASM compatibility
                        // In WASM, immediate retry is often sufficient
                    }

                    match result {
                        Ok(response) => {
                            // Parse JSON response and extract events
                            let (_, events_data) = if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&response)
                            {
                                if let Some(result) = json.get("result") {
                                    if let Some(logs) = result.as_array() {
                                        (logs.len(), logs.clone())
                                    } else {
                                        (0, vec![])
                                    }
                                } else {
                                    (0, vec![])
                                }
                            } else {
                                (0, vec![])
                            };

                            Ok(events_data)
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .buffer_unordered(10) // Max 14 concurrent requests
            .collect()
            .await;

    // Collect all events
    let mut all_events = Vec::new();

    for events_data in results.into_iter().flatten() {
        all_events.extend(events_data);
    }

    // Sort events by block number
    all_events.sort_by(|a, b| {
        let block_a = a
            .get("blockNumber")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<U256>().ok())
            .map(|u| u.to::<u64>())
            .unwrap_or(0);
        let block_b = b
            .get("blockNumber")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<U256>().ok())
            .map(|u| u.to::<u64>())
            .unwrap_or(0);
        block_a.cmp(&block_b)
    });

    // Backfill missing timestamps
    let mut events_array = serde_json::Value::Array(all_events);
    let _ = backfill_missing_timestamps(&mut events_array).await;

    Ok(events_array)
}
