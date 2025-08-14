use alloy::sol_types::SolEvent;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_bindings::{
    IOrderBookV4::{AddOrderV2, Deposit, RemoveOrderV2, TakeOrderV2, Withdraw},
    OrderBook::MetaV1_2,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

const API_TOKEN: &str = env!("HYPER_SYNC_API_TOKEN");
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

    pub async fn get_latest_block_number(&self) -> anyhow::Result<u64> {
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
            return Err(anyhow::anyhow!("RPC error getting latest block: {}", error));
        }

        if let Some(result) = json.get("result") {
            if let Some(block_hex) = result.as_str() {
                let block_hex = block_hex.strip_prefix("0x").unwrap_or(block_hex);
                let block_number = u64::from_str_radix(block_hex, 16)?;
                return Ok(block_number);
            }
        }

        Err(anyhow::anyhow!("No result field in response"))
    }

    pub async fn get_logs(
        &self,
        from_block: &str,
        to_block: &str,
        address: &str,
        topics: Option<Vec<Option<Vec<String>>>>,
    ) -> anyhow::Result<String> {
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
                return Err(anyhow::anyhow!("RPC error: {}", error));
            }
        }

        Ok(text)
    }
}

#[derive(Debug, Clone, Parser)]
pub struct FetchEvents {
    #[clap(short, long, default_value = "fetch_events_results.json")]
    pub output_file: String,
}

impl FetchEvents {
    pub async fn execute(self) -> Result<()> {
        let client = HyperRpcClient {};

        let latest_block = client.get_latest_block_number().await?;
        println!("Latest block: {}", latest_block);

        let start_block = 19033330u64;
        let chunk_size = 50000u64; // Optimal block chunk size
        let contract_address = "0xd2938e7c9fe3597f78832ce780feb61945c377d7";

        // Use multiple event signatures for filtering
        let topics = Some(vec![Some(vec![
            AddOrderV2::SIGNATURE_HASH.to_string(),    // add order
            TakeOrderV2::SIGNATURE_HASH.to_string(),   // take order
            Withdraw::SIGNATURE_HASH.to_string(),      // withdraw
            Deposit::SIGNATURE_HASH.to_string(),       // deposit
            RemoveOrderV2::SIGNATURE_HASH.to_string(), // remove order
            MetaV1_2::SIGNATURE_HASH.to_string(),      // meta
        ])]);

        println!("Block diff: {}", latest_block - start_block);

        // Prepare all chunk ranges
        let mut chunks = Vec::new();
        let mut current_block = start_block;
        while current_block <= latest_block {
            let to_block = std::cmp::min(current_block + chunk_size - 1, latest_block);
            chunks.push((current_block, to_block));
            current_block = to_block + 1;
        }

        println!("Total chunks to process: {}", chunks.len());
        let total_chunks = chunks.len();

        let total_start = std::time::Instant::now();

        // Process chunks with concurrency limit to avoid timeouts
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10)); // Max 10 concurrent requests
        let tasks: Vec<_> = chunks
            .into_iter()
            .enumerate()
            .map(|(i, (from_block, to_block))| {
                let client = HyperRpcClient {};
                let topics = topics.clone();
                let contract_address = contract_address.to_string();
                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let from_block_hex = format!("0x{:x}", from_block);
                    let to_block_hex = format!("0x{:x}", to_block);
                    let chunk_start = std::time::Instant::now();

                    println!(
                        "Starting chunk {}: {} to {}",
                        i + 1,
                        from_block_hex,
                        to_block_hex
                    );

                    // Retry logic for failed requests
                    let mut result = Err(anyhow::anyhow!("Not attempted"));
                    for attempt in 1..=3 {
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

                        if attempt < 3 {
                            println!("Chunk {} attempt {} failed, retrying...", i + 1, attempt);
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000 * attempt))
                                .await;
                        }
                    }

                    match result {
                        Ok(response) => {
                            let chunk_time = chunk_start.elapsed();
                            let response_size = response.len();

                            // Parse JSON response and extract events
                            let (events_count, events_data) = if let Ok(json) =
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

                            println!(
                                "✓ Chunk {} ({} to {}): {} events, {} bytes in {:.2}s",
                                i + 1,
                                from_block_hex,
                                to_block_hex,
                                events_count,
                                response_size,
                                chunk_time.as_secs_f64()
                            );

                            Ok((events_count, response_size, chunk_time, events_data))
                        }
                        Err(e) => {
                            let chunk_time = chunk_start.elapsed();
                            println!(
                                "✗ Chunk {} failed in {:.2}s: {}",
                                i + 1,
                                chunk_time.as_secs_f64(),
                                e
                            );
                            Err(e)
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        let total_time = total_start.elapsed();

        // Aggregate results and collect all events
        let mut total_events = 0;
        let mut total_size_bytes = 0;
        let mut successful_chunks = 0;
        let mut failed_chunks = 0;
        let mut all_events = Vec::new();

        for result in results {
            match result {
                Ok(Ok((events, size, _duration, events_data))) => {
                    total_events += events;
                    total_size_bytes += size;
                    successful_chunks += 1;
                    all_events.extend(events_data);
                }
                Ok(Err(_)) | Err(_) => {
                    failed_chunks += 1;
                }
            }
        }

        // Sort events by block number
        all_events.sort_by(|a, b| {
            let block_a = a
                .get("blockNumber")
                .and_then(|v| v.as_str())
                .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                .unwrap_or(0);
            let block_b = b
                .get("blockNumber")
                .and_then(|v| v.as_str())
                .and_then(|s| u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok())
                .unwrap_or(0);
            block_a.cmp(&block_b)
        });

        let output_data = serde_json::json!({
            "metadata": {
                "test_config": {
                    "start_block": start_block,
                    "latest_block": latest_block,
                    "chunk_size": chunk_size,
                    "contract_address": contract_address,
                    "total_chunks": total_chunks
                },
                "results": {
                    "total_blocks_processed": latest_block - start_block + 1,
                    "successful_chunks": successful_chunks,
                    "failed_chunks": failed_chunks,
                    "total_events_found": total_events,
                    "total_time_seconds": total_time.as_secs_f64(),
                    "total_size_bytes": total_size_bytes,
                    "total_size_mb": total_size_bytes as f64 / 1_048_576.0,
                    "average_time_per_chunk": if successful_chunks > 0 {
                        total_time.as_secs_f64() / successful_chunks as f64
                    } else {
                        0.0
                    }
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "events": all_events
        });

        println!("\n=== SUMMARY ===");
        println!("Total blocks processed: {}", latest_block - start_block + 1);
        println!("Successful chunks: {}", successful_chunks);
        println!("Failed chunks: {}", failed_chunks);
        println!("Total events found: {}", total_events);
        println!("Total time: {:.2}s", total_time.as_secs_f64());
        println!(
            "Total size: {} bytes ({:.2} MB)",
            total_size_bytes,
            total_size_bytes as f64 / 1_048_576.0
        );
        if successful_chunks > 0 {
            println!(
                "Average time per successful chunk: {:.2}s (actual parallel execution)",
                total_time.as_secs_f64()
            );
        }

        // Save results to file
        let mut file = File::create(&self.output_file)?;
        file.write_all(serde_json::to_string_pretty(&output_data)?.as_bytes())?;
        println!("Events and results saved to: {}", self.output_file);

        Ok(())
    }
}
