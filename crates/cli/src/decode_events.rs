use alloy::sol_types::SolEvent;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_bindings::IOrderBookV4::{
    AddOrderV2, Deposit, RemoveOrderV2, TakeOrderV2, Withdraw,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_type: String,
    pub block_number: String,
    pub block_timestamp: String,
    pub transaction_hash: String,
    pub log_index: String,
    pub decoded_data: serde_json::Value,
}

#[derive(Debug, Clone, Parser)]
pub struct DecodeEvents {
    #[clap(short, long, default_value = "stress_test_results.json")]
    pub input_file: String,

    #[clap(short, long, default_value = "decoded_events.json")]
    pub output_file: String,
}

impl DecodeEvents {
    pub async fn execute(self) -> Result<()> {
        // Read the stress test results file
        let file_content = std::fs::read_to_string(&self.input_file)?;
        let json_data: serde_json::Value = serde_json::from_str(&file_content)?;

        // Extract events array
        let events = json_data
            .get("events")
            .and_then(|e| e.as_array())
            .ok_or_else(|| anyhow::anyhow!("No events found in input file"))?;

        // Create topic to event type mapping
        let mut topic_map = HashMap::new();
        topic_map.insert(AddOrderV2::SIGNATURE_HASH.to_string(), "AddOrderV2");
        topic_map.insert(TakeOrderV2::SIGNATURE_HASH.to_string(), "TakeOrderV2");
        topic_map.insert(Withdraw::SIGNATURE_HASH.to_string(), "Withdraw");
        topic_map.insert(Deposit::SIGNATURE_HASH.to_string(), "Deposit");
        topic_map.insert(RemoveOrderV2::SIGNATURE_HASH.to_string(), "RemoveOrderV2");

        println!("Processing {} events...", events.len());

        let mut decoded_events = Vec::new();
        let mut decode_stats = HashMap::new();

        for event in events {
            if let Some(topics) = event.get("topics").and_then(|t| t.as_array()) {
                if let Some(topic0) = topics.first().and_then(|t| t.as_str()) {
                    let topic0_clean = if let Some(stripped) = topic0.strip_prefix("0x") {
                        stripped
                    } else {
                        topic0
                    };

                    // Find matching event type
                    let event_type = topic_map
                        .iter()
                        .find(|(hash, _)| {
                            let hash_clean = if let Some(stripped) = hash.strip_prefix("0x") {
                                stripped
                            } else {
                                hash.as_str()
                            };
                            hash_clean.eq_ignore_ascii_case(topic0_clean)
                        })
                        .map(|(_, name)| *name)
                        .unwrap_or("Unknown");

                    *decode_stats.entry(event_type.to_string()).or_insert(0) += 1;

                    if let Some(data_str) = event.get("data").and_then(|d| d.as_str()) {
                        let decoded_data = match event_type {
                            "AddOrderV2" => decode_add_order_v2(data_str)?,
                            "TakeOrderV2" => decode_take_order_v2(data_str)?,
                            "Withdraw" => decode_withdraw(data_str)?,
                            "Deposit" => decode_deposit(data_str)?,
                            "RemoveOrderV2" => decode_remove_order_v2(data_str)?,
                            _ => {
                                serde_json::json!({
                                    "raw_data": data_str,
                                    "note": "Unknown event type - could not decode"
                                })
                            }
                        };

                        let event_data = EventData {
                            event_type: event_type.to_string(),
                            block_number: event
                                .get("blockNumber")
                                .and_then(|b| b.as_str())
                                .unwrap_or("0x0")
                                .to_string(),
                            block_timestamp: event
                                .get("blockTimestamp")
                                .and_then(|b| b.as_str())
                                .unwrap_or("0x0")
                                .to_string(),
                            transaction_hash: event
                                .get("transactionHash")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .to_string(),
                            log_index: event
                                .get("logIndex")
                                .and_then(|l| l.as_str())
                                .unwrap_or("0x0")
                                .to_string(),
                            decoded_data,
                        };

                        decoded_events.push(event_data);
                    }
                }
            }
        }

        // Create output data
        let output_data = serde_json::json!({
            "metadata": {
                "source_file": self.input_file,
                "total_events_processed": events.len(),
                "total_events_decoded": decoded_events.len(),
                "decode_statistics": decode_stats,
                "timestamp": chrono::Utc::now().to_rfc3339()
            },
            "decoded_events": decoded_events
        });

        // Write to output file
        let mut file = File::create(&self.output_file)?;
        file.write_all(serde_json::to_string_pretty(&output_data)?.as_bytes())?;

        println!("\n=== DECODE SUMMARY ===");
        println!("Total events processed: {}", events.len());
        println!("Total events decoded: {}", decoded_events.len());
        println!("Decode statistics:");
        for (event_type, count) in &decode_stats {
            println!("  {}: {}", event_type, count);
        }
        println!("Decoded events saved to: {}", self.output_file);

        Ok(())
    }
}

fn decode_add_order_v2(data_str: &str) -> Result<serde_json::Value> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match AddOrderV2::abi_decode_data(&data_bytes, true) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "order_hash": format!("0x{}", hex::encode(decoded.1)),
            "order": {
                "owner": format!("0x{:x}", decoded.2.owner),
                "nonce": format!("0x{:x}", decoded.2.nonce),
                "evaluable": {
                    "interpreter": format!("0x{:x}", decoded.2.evaluable.interpreter),
                    "store": format!("0x{:x}", decoded.2.evaluable.store),
                    "bytecode": format!("0x{}", hex::encode(&decoded.2.evaluable.bytecode))
                },
                "valid_inputs": decoded.2.validInputs.iter().map(|input| {
                    serde_json::json!({
                        "token": format!("0x{:x}", input.token),
                        "decimals": input.decimals,
                        "vault_id": format!("0x{:x}", input.vaultId)
                    })
                }).collect::<Vec<_>>(),
                "valid_outputs": decoded.2.validOutputs.iter().map(|output| {
                    serde_json::json!({
                        "token": format!("0x{:x}", output.token),
                        "decimals": output.decimals,
                        "vault_id": format!("0x{:x}", output.vaultId)
                    })
                }).collect::<Vec<_>>()
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "raw_data": data_str,
            "decode_error": e.to_string()
        })),
    }
}

fn decode_take_order_v2(data_str: &str) -> Result<serde_json::Value> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match TakeOrderV2::abi_decode_data(&data_bytes, true) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "config": {
                "order": {
                    "owner": format!("0x{:x}", decoded.1.order.owner),
                    "nonce": format!("0x{:x}", decoded.1.order.nonce),
                    "evaluable": {
                        "interpreter": format!("0x{:x}", decoded.1.order.evaluable.interpreter),
                        "store": format!("0x{:x}", decoded.1.order.evaluable.store),
                        "bytecode": format!("0x{}", hex::encode(&decoded.1.order.evaluable.bytecode))
                    },
                    "valid_inputs": decoded.1.order.validInputs.iter().map(|input| {
                        serde_json::json!({
                            "token": format!("0x{:x}", input.token),
                            "decimals": input.decimals,
                            "vault_id": format!("0x{:x}", input.vaultId)
                        })
                    }).collect::<Vec<_>>(),
                    "valid_outputs": decoded.1.order.validOutputs.iter().map(|output| {
                        serde_json::json!({
                            "token": format!("0x{:x}", output.token),
                            "decimals": output.decimals,
                            "vault_id": format!("0x{:x}", output.vaultId)
                        })
                    }).collect::<Vec<_>>()
                },
                "input_io_index": decoded.1.inputIOIndex,
                "output_io_index": decoded.1.outputIOIndex,
                "signed_context": decoded.1.signedContext.iter().map(|ctx| {
                    serde_json::json!({
                        "signer": format!("0x{:x}", ctx.signer),
                        "context": ctx.context.iter().map(|c| format!("0x{:x}", c)).collect::<Vec<_>>(),
                        "signature": format!("0x{}", hex::encode(&ctx.signature))
                    })
                }).collect::<Vec<_>>()
            },
            "input": format!("{}", decoded.2),
            "output": format!("{}", decoded.3)
        })),
        Err(e) => Ok(serde_json::json!({
            "raw_data": data_str,
            "decode_error": e.to_string()
        })),
    }
}

fn decode_withdraw(data_str: &str) -> Result<serde_json::Value> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match Withdraw::abi_decode_data(&data_bytes, true) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "token": format!("0x{:x}", decoded.1),
            "vault_id": format!("0x{:x}", decoded.2),
            "target_amount": format!("{}", decoded.3),
            "amount": format!("{}", decoded.4)
        })),
        Err(e) => Ok(serde_json::json!({
            "raw_data": data_str,
            "decode_error": e.to_string()
        })),
    }
}

fn decode_deposit(data_str: &str) -> Result<serde_json::Value> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match Deposit::abi_decode_data(&data_bytes, true) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "token": format!("0x{:x}", decoded.1),
            "vault_id": format!("0x{:x}", decoded.2),
            "amount": format!("{}", decoded.3)
        })),
        Err(e) => Ok(serde_json::json!({
            "raw_data": data_str,
            "decode_error": e.to_string()
        })),
    }
}

fn decode_remove_order_v2(data_str: &str) -> Result<serde_json::Value> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match RemoveOrderV2::abi_decode_data(&data_bytes, true) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "order_hash": format!("0x{}", hex::encode(decoded.1)),
            "order": {
                "owner": format!("0x{:x}", decoded.2.owner),
                "nonce": format!("0x{:x}", decoded.2.nonce),
                "evaluable": {
                    "interpreter": format!("0x{:x}", decoded.2.evaluable.interpreter),
                    "store": format!("0x{:x}", decoded.2.evaluable.store),
                    "bytecode": format!("0x{}", hex::encode(&decoded.2.evaluable.bytecode))
                },
                "valid_inputs": decoded.2.validInputs.iter().map(|input| {
                    serde_json::json!({
                        "token": format!("0x{:x}", input.token),
                        "decimals": input.decimals,
                        "vault_id": format!("0x{:x}", input.vaultId)
                    })
                }).collect::<Vec<_>>(),
                "valid_outputs": decoded.2.validOutputs.iter().map(|output| {
                    serde_json::json!({
                        "token": format!("0x{:x}", output.token),
                        "decimals": output.decimals,
                        "vault_id": format!("0x{:x}", output.vaultId)
                    })
                }).collect::<Vec<_>>()
            }
        })),
        Err(e) => Ok(serde_json::json!({
            "raw_data": data_str,
            "decode_error": e.to_string()
        })),
    }
}
