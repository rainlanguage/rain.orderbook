use alloy::{primitives::U256, sol_types::SolEvent};
use futures::StreamExt;
use rain_orderbook_bindings::{
    IOrderBookV4::{
        AddOrderV2, AfterClear, ClearV2, Deposit, RemoveOrderV2, TakeOrderV2, Withdraw,
    },
    OrderBook::MetaV1_2,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const API_TOKEN: &str = "41e50e69-6da4-4462-b70e-c7b5e7b70f05";
const RPC_URL: &str = "https://base.rpc.hypersync.xyz/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    #[serde(rename = "fromBlock")]
    pub from_block: String,
    #[serde(rename = "toBlock")]
    pub to_block: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<Option<Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperRpcClient {}

impl HyperRpcClient {
    fn get_url() -> String {
        format!("{}/{}", RPC_URL, API_TOKEN)
    }

    pub async fn fetch_url(
        &self,
        url: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", response.status()).into());
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn get_latest_block_number(
        &self,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(Self::get_url())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_blockNumber",
                "params": []
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        // Check for RPC errors
        if let Some(error) = json.get("error") {
            return Err(format!("RPC error getting latest block: {}", error).into());
        }

        if let Some(result) = json.get("result") {
            if let Some(block_hex) = result.as_str() {
                let block_hex = block_hex.strip_prefix("0x").unwrap_or(block_hex);
                let block_number = u64::from_str_radix(block_hex, 16)?;
                return Ok(block_number);
            }
        }

        Err("No result field in response".into())
    }

    pub async fn get_logs(
        &self,
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(Self::get_url())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_getLogs",
                "params": [LogFilter {
                    from_block: from_block.to_string(),
                    to_block: to_block.to_string(),
                    address: address.to_string(),
                    topics: topics.clone(),
                }]
            }))
            .send()
            .await?;

        let text = response.text().await?;

        // Check for RPC errors in the response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error) = json.get("error") {
                return Err(format!("RPC error: {}", error).into());
            }
        }

        Ok(text)
    }

    pub async fn get_block_by_number(
        &self,
        block_number: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let block_hex = format!("0x{:x}", block_number);

        let response = client
            .post(Self::get_url())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_getBlockByNumber",
                "params": [block_hex, false]
            }))
            .send()
            .await?;

        let text = response.text().await?;

        // Check for RPC errors in the response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(error) = json.get("error") {
                return Err(format!("RPC error: {}", error).into());
            }
        }

        Ok(text)
    }
}

pub async fn fetch_block_timestamps(
    block_numbers: Vec<u64>,
) -> Result<HashMap<u64, String>, Box<dyn std::error::Error + Send + Sync>> {
    if block_numbers.is_empty() {
        return Ok(HashMap::new());
    }

    let client = HyperRpcClient {};
    let results: Vec<Result<(u64, String), Box<dyn std::error::Error + Send + Sync>>> =
        futures::stream::iter(block_numbers)
            .map(|block_number| {
                let client = client.clone();
                async move {
                    // Retry logic for failed requests
                    let mut result = Err("Not attempted".into());
                    for _attempt in 1..=3 {
                        result = client.get_block_by_number(block_number).await;
                        if result.is_ok() {
                            break;
                        }
                    }

                    match result {
                        Ok(response) => {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                                if let Some(result) = json.get("result") {
                                    if let Some(block) = result.as_object() {
                                        if let Some(timestamp) =
                                            block.get("timestamp").and_then(|t| t.as_str())
                                        {
                                            return Ok((block_number, timestamp.to_string()));
                                        }
                                    }
                                }
                            }
                            Err(
                                format!("Failed to parse timestamp for block {}", block_number)
                                    .into(),
                            )
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .buffer_unordered(14) // Same concurrency as event fetching
            .collect()
            .await;

    let mut timestamps = HashMap::new();

    for (block_number, timestamp) in results.into_iter().flatten() {
        timestamps.insert(block_number, timestamp);
    }

    Ok(timestamps)
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
        AddOrderV2::SIGNATURE_HASH.to_string(),    // add order
        TakeOrderV2::SIGNATURE_HASH.to_string(),   // take order
        Withdraw::SIGNATURE_HASH.to_string(),      // withdraw
        Deposit::SIGNATURE_HASH.to_string(),       // deposit
        RemoveOrderV2::SIGNATURE_HASH.to_string(), // remove order
        ClearV2::SIGNATURE_HASH.to_string(),       // clear
        AfterClear::SIGNATURE_HASH.to_string(),    // after clear
        MetaV1_2::SIGNATURE_HASH.to_string(),      // meta
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
