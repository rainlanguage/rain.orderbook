use super::LocalDb;
use alloy::primitives::keccak256;
use alloy::{
    hex,
    sol_types::{SolEvent, SolValue},
};
use rain_orderbook_bindings::{
    IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearV3, DepositV2, OrderV4, RemoveOrderV3, TakeOrderV3,
        WithdrawV2,
    },
    OrderBook::MetaV1_2,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    #[error("ABI decode error: {0}")]
    AbiDecode(String),
    #[error("Alice input IO index {index} out of bounds (max: {max})")]
    AliceInputIOIndexOutOfBounds { index: u64, max: usize },
    #[error("Alice output IO index {index} out of bounds (max: {max})")]
    AliceOutputIOIndexOutOfBounds { index: u64, max: usize },
    #[error("Bob input IO index {index} out of bounds (max: {max})")]
    BobInputIOIndexOutOfBounds { index: u64, max: usize },
    #[error("Bob output IO index {index} out of bounds (max: {max})")]
    BobOutputIOIndexOutOfBounds { index: u64, max: usize },
    #[error("Order hash computation failed: {0}")]
    OrderHashComputation(String),
    #[error("Expected array of events")]
    InvalidJsonStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_type: String,
    pub block_number: String,
    pub block_timestamp: String,
    pub transaction_hash: String,
    pub log_index: String,
    pub decoded_data: serde_json::Value,
}

impl LocalDb {
    pub fn decode_events(
        &self,
        json_data: serde_json::Value,
    ) -> Result<serde_json::Value, DecodeError> {
        let events = json_data
            .as_array()
            .ok_or(DecodeError::InvalidJsonStructure)?;

        let mut topic_map = HashMap::new();
        topic_map.insert(AddOrderV3::SIGNATURE_HASH.to_string(), "AddOrderV3");
        topic_map.insert(TakeOrderV3::SIGNATURE_HASH.to_string(), "TakeOrderV3");
        topic_map.insert(WithdrawV2::SIGNATURE_HASH.to_string(), "WithdrawV2");
        topic_map.insert(DepositV2::SIGNATURE_HASH.to_string(), "DepositV2");
        topic_map.insert(RemoveOrderV3::SIGNATURE_HASH.to_string(), "RemoveOrderV3");
        topic_map.insert(ClearV3::SIGNATURE_HASH.to_string(), "ClearV3");
        topic_map.insert(AfterClearV2::SIGNATURE_HASH.to_string(), "AfterClearV2");
        topic_map.insert(MetaV1_2::SIGNATURE_HASH.to_string(), "MetaV1_2");

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
                            "AddOrderV3" => decode_add_order_v3(data_str)?,
                            "TakeOrderV3" => decode_take_order_v3(data_str)?,
                            "WithdrawV2" => decode_withdraw_v2(data_str)?,
                            "DepositV2" => decode_deposit_v2(data_str)?,
                            "RemoveOrderV3" => decode_remove_order_v3(data_str)?,
                            "ClearV3" => decode_clear_v3(data_str)?,
                            "AfterClearV2" => decode_after_clear_v2(data_str)?,
                            "MetaV1_2" => decode_meta_v1_2(data_str)?,
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
                                .and_then(|t| t.as_str())
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

        Ok(serde_json::json!(decoded_events))
    }
}

fn compute_order_hash(order: &OrderV4) -> Result<String, DecodeError> {
    let encoded = order.abi_encode();
    let hash = keccak256(&encoded);
    Ok(format!("0x{}", hex::encode(hash)))
}

fn decode_add_order_v3(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match AddOrderV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "order_hash": format!("0x{}", hex::encode(decoded.1)),
            "order_bytes": format!("0x{}", hex::encode(decoded.2.abi_encode())),
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
                        "vault_id": format!("0x{:x}", input.vaultId)
                    })
                }).collect::<Vec<_>>(),
                "valid_outputs": decoded.2.validOutputs.iter().map(|output| {
                    serde_json::json!({
                        "token": format!("0x{:x}", output.token),
                        "vault_id": format!("0x{:x}", output.vaultId)
                    })
                }).collect::<Vec<_>>()
            }
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_take_order_v3(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match TakeOrderV3::abi_decode_data(&data_bytes) {
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
                                "vault_id": format!("0x{:x}", input.vaultId)
                        })
                    }).collect::<Vec<_>>(),
                    "valid_outputs": decoded.1.order.validOutputs.iter().map(|output| {
                        serde_json::json!({
                            "token": format!("0x{:x}", output.token),
                                "vault_id": format!("0x{:x}", output.vaultId)
                        })
                    }).collect::<Vec<_>>()
                },
                "input_io_index": format!("0x{:x}", decoded.1.inputIOIndex),
                "output_io_index": format!("0x{:x}", decoded.1.outputIOIndex),
                "signed_context": decoded.1.signedContext.iter().map(|ctx| {
                    serde_json::json!({
                        "signer": format!("0x{:x}", ctx.signer),
                        "context": ctx.context.iter().map(|c| format!("0x{:x}", c)).collect::<Vec<_>>(),
                        "signature": format!("0x{}", hex::encode(&ctx.signature))
                    })
                }).collect::<Vec<_>>()
            },
            "taker_input": format!("0x{:x}", decoded.2),
            "taker_output": format!("0x{:x}", decoded.3)
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_withdraw_v2(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match WithdrawV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "token": format!("0x{:x}", decoded.1),
            "vault_id": format!("0x{:x}", decoded.2),
            "target_amount": format!("0x{:x}", decoded.3),
            "withdraw_amount": format!("0x{:x}", decoded.4),
            "withdraw_amount_uint256": format!("0x{:x}", decoded.5)
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_deposit_v2(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match DepositV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "token": format!("0x{:x}", decoded.1),
            "vault_id": format!("0x{:x}", decoded.2),
            "deposit_amount_uint256": format!("0x{:x}", decoded.3)
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_remove_order_v3(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match RemoveOrderV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "order_hash": format!("0x{}", hex::encode(decoded.1)),
            "order_bytes": format!("0x{}", hex::encode(decoded.2.abi_encode())),
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
                        "vault_id": format!("0x{:x}", input.vaultId)
                    })
                }).collect::<Vec<_>>(),
                "valid_outputs": decoded.2.validOutputs.iter().map(|output| {
                    serde_json::json!({
                        "token": format!("0x{:x}", output.token),
                        "vault_id": format!("0x{:x}", output.vaultId)
                    })
                }).collect::<Vec<_>>()
            }
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_clear_v3(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match ClearV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => {
            let alice_input_index = decoded.3.aliceInputIOIndex.to::<u64>();
            let alice_input_vault_id =
                if let Some(input) = decoded.1.validInputs.get(alice_input_index as usize) {
                    format!("0x{:x}", input.vaultId)
                } else {
                    return Err(DecodeError::AliceInputIOIndexOutOfBounds {
                        index: alice_input_index,
                        max: decoded.1.validInputs.len(),
                    });
                };

            let alice_output_index = decoded.3.aliceOutputIOIndex.to::<u64>();
            let alice_output_vault_id =
                if let Some(output) = decoded.1.validOutputs.get(alice_output_index as usize) {
                    format!("0x{:x}", output.vaultId)
                } else {
                    return Err(DecodeError::AliceOutputIOIndexOutOfBounds {
                        index: alice_output_index,
                        max: decoded.1.validOutputs.len(),
                    });
                };

            let bob_input_index = decoded.3.bobInputIOIndex.to::<u64>();
            let bob_input_vault_id =
                if let Some(input) = decoded.2.validInputs.get(bob_input_index as usize) {
                    format!("0x{:x}", input.vaultId)
                } else {
                    return Err(DecodeError::BobInputIOIndexOutOfBounds {
                        index: bob_input_index,
                        max: decoded.2.validInputs.len(),
                    });
                };

            let bob_output_index = decoded.3.bobOutputIOIndex.to::<u64>();
            let bob_output_vault_id =
                if let Some(output) = decoded.2.validOutputs.get(bob_output_index as usize) {
                    format!("0x{:x}", output.vaultId)
                } else {
                    return Err(DecodeError::BobOutputIOIndexOutOfBounds {
                        index: bob_output_index,
                        max: decoded.2.validOutputs.len(),
                    });
                };

            let alice_order_hash = compute_order_hash(&decoded.1)?;
            let bob_order_hash = compute_order_hash(&decoded.2)?;

            Ok(serde_json::json!({
                "sender": format!("0x{:x}", decoded.0),
                "alice_owner": format!("0x{:x}", decoded.1.owner),
                "bob_owner": format!("0x{:x}", decoded.2.owner),
                "alice_order_hash": alice_order_hash,
                "bob_order_hash": bob_order_hash,
                "alice_input_io_index": alice_input_index,
                "alice_output_io_index": alice_output_index,
                "alice_bounty_vault_id": format!("0x{:x}", decoded.3.aliceBountyVaultId),
                "bob_input_io_index": bob_input_index,
                "bob_output_io_index": bob_output_index,
                "bob_bounty_vault_id": format!("0x{:x}", decoded.3.bobBountyVaultId),
                "alice_input_vault_id": alice_input_vault_id,
                "alice_output_vault_id": alice_output_vault_id,
                "bob_input_vault_id": bob_input_vault_id,
                "bob_output_vault_id": bob_output_vault_id
            }))
        }
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_after_clear_v2(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match AfterClearV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "alice_input": format!("0x{:x}", decoded.1.aliceInput),
            "alice_output": format!("0x{:x}", decoded.1.aliceOutput),
            "bob_input": format!("0x{:x}", decoded.1.bobInput),
            "bob_output": format!("0x{:x}", decoded.1.bobOutput)
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_meta_v1_2(data_str: &str) -> Result<serde_json::Value, DecodeError> {
    let data_bytes = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))?;

    match MetaV1_2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(serde_json::json!({
            "sender": format!("0x{:x}", decoded.0),
            "subject": format!("0x{:x}", decoded.1),
            "meta": format!("0x{}", hex::encode(&decoded.2))
        })),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use alloy::hex;
    use alloy::primitives::{Address, Bytes, FixedBytes, U256};
    use rain_orderbook_bindings::{
        IOrderBookV5::{
            AddOrderV3, AfterClearV2, ClearConfigV2, ClearStateChangeV2, ClearV3, DepositV2,
            RemoveOrderV3, SignedContextV1, TakeOrderConfigV4, TakeOrderV3, WithdrawV2,
        },
        IOrderBookV5::{EvaluableV4, OrderV4, IOV2},
        OrderBook::MetaV1_2,
    };

    fn create_sample_order_v4() -> OrderV4 {
        OrderV4 {
            owner: Address::from([1u8; 20]),
            nonce: U256::from(1).into(),
            evaluable: EvaluableV4 {
                interpreter: Address::from([2u8; 20]),
                store: Address::from([3u8; 20]),
                bytecode: Bytes::from(vec![0x01, 0x02, 0x03, 0x04]),
            },
            validInputs: vec![
                IOV2 {
                    token: Address::from([4u8; 20]),
                    vaultId: U256::from(100).into(),
                },
                IOV2 {
                    token: Address::from([5u8; 20]),
                    vaultId: U256::from(200).into(),
                },
            ],
            validOutputs: vec![IOV2 {
                token: Address::from([6u8; 20]),
                vaultId: U256::from(300).into(),
            }],
        }
    }

    fn create_add_order_v3_event_data() -> serde_json::Value {
        let sender = Address::from([7u8; 20]);
        let order_hash = FixedBytes::<32>::from([8u8; 32]);
        let order = create_sample_order_v4();

        let event_data = AddOrderV3 {
            sender,
            orderHash: order_hash,
            order,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        })
    }

    fn create_take_order_v3_event_data() -> serde_json::Value {
        let sender = Address::from([9u8; 20]);
        let config = TakeOrderConfigV4 {
            order: create_sample_order_v4(),
            inputIOIndex: U256::from(0),
            outputIOIndex: U256::from(0),
            signedContext: vec![SignedContextV1 {
                signer: Address::from([10u8; 20]),
                context: vec![U256::from(42).into(), U256::from(43).into()],
                signature: Bytes::from(vec![0x11, 0x22, 0x33]),
            }],
        };
        let input = U256::from(1000);
        let output = U256::from(2000);

        let event_data = TakeOrderV3 {
            sender,
            config,
            input: input.into(),
            output: output.into(),
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(TakeOrderV3::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x123457",
            "blockTimestamp": "0x64b8c124",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891",
            "logIndex": "0x1"
        })
    }

    fn create_withdraw_v2_event_data() -> serde_json::Value {
        let sender = Address::from([11u8; 20]);
        let token = Address::from([12u8; 20]);
        let vault_id = U256::from(500);
        let target_amount = U256::from(3000);
        let withdraw_amount = U256::from(2500);
        let withdraw_amount_uint256 = U256::from(2500);

        let event_data = WithdrawV2 {
            sender,
            token,
            vaultId: vault_id.into(),
            targetAmount: target_amount.into(),
            withdrawAmount: withdraw_amount.into(),
            withdrawAmountUint256: withdraw_amount_uint256,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(WithdrawV2::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x123458",
            "blockTimestamp": "0x64b8c125",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892",
            "logIndex": "0x2"
        })
    }

    fn create_deposit_v2_event_data() -> serde_json::Value {
        let sender = Address::from([13u8; 20]);
        let token = Address::from([14u8; 20]);
        let vault_id = U256::from(600);
        let deposit_amount_uint256 = U256::from(4000);

        let event_data = DepositV2 {
            sender,
            token,
            vaultId: vault_id.into(),
            depositAmountUint256: deposit_amount_uint256,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(DepositV2::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x123459",
            "blockTimestamp": "0x64b8c126",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "logIndex": "0x3"
        })
    }

    fn create_remove_order_v3_event_data() -> serde_json::Value {
        let sender = Address::from([15u8; 20]);
        let order_hash = FixedBytes::<32>::from([16u8; 32]);
        let order = create_sample_order_v4();

        let event_data = RemoveOrderV3 {
            sender,
            orderHash: order_hash,
            order,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(RemoveOrderV3::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x12345a",
            "blockTimestamp": "0x64b8c127",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567894",
            "logIndex": "0x4"
        })
    }

    fn create_clear_v3_event_data() -> serde_json::Value {
        let sender = Address::from([17u8; 20]);
        let alice_order = create_sample_order_v4();
        let bob_order = OrderV4 {
            owner: Address::from([18u8; 20]),
            nonce: U256::from(2).into(),
            evaluable: EvaluableV4 {
                interpreter: Address::from([19u8; 20]),
                store: Address::from([20u8; 20]),
                bytecode: Bytes::from(vec![0x05, 0x06, 0x07, 0x08]),
            },
            validInputs: vec![IOV2 {
                token: Address::from([21u8; 20]),
                vaultId: U256::from(700).into(),
            }],
            validOutputs: vec![IOV2 {
                token: Address::from([22u8; 20]),
                vaultId: U256::from(800).into(),
            }],
        };
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x12345b",
            "blockTimestamp": "0x64b8c128",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567895",
            "logIndex": "0x5"
        })
    }

    fn create_after_clear_v2_event_data() -> serde_json::Value {
        let sender = Address::from([23u8; 20]);
        let clear_state_change = ClearStateChangeV2 {
            aliceInput: U256::from(5000).into(),
            aliceOutput: U256::from(6000).into(),
            bobInput: U256::from(7000).into(),
            bobOutput: U256::from(8000).into(),
        };

        let event_data = AfterClearV2 {
            sender,
            clearStateChange: clear_state_change,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AfterClearV2::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x12345c",
            "blockTimestamp": "0x64b8c129",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567896",
            "logIndex": "0x6"
        })
    }

    fn create_meta_v1_2_event_data() -> serde_json::Value {
        let sender = Address::from([24u8; 20]);
        let subject = Address::from([25u8; 20]);
        let meta = Bytes::from(vec![0x09, 0x0a, 0x0b, 0x0c, 0x0d]);

        let event_data = MetaV1_2 {
            sender,
            subject: {
                let mut bytes = [0u8; 32];
                bytes[12..32].copy_from_slice(&subject[..]);
                FixedBytes::<32>::from(bytes)
            },
            meta,
        };

        let encoded_data = event_data.encode_data();

        serde_json::json!({
            "topics": [format!("0x{}", hex::encode(MetaV1_2::SIGNATURE_HASH))],
            "data": format!("0x{}", hex::encode(encoded_data)),
            "blockNumber": "0x12345d",
            "blockTimestamp": "0x64b8c12a",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567897",
            "logIndex": "0x7"
        })
    }

    #[test]
    fn test_add_order_v3_decode() {
        let event_data = create_add_order_v3_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "AddOrderV3");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0707070707070707070707070707070707070707"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order_hash"],
            "0x0808080808080808080808080808080808080808080808080808080808080808"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["owner"],
            "0x0101010101010101010101010101010101010101"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["nonce"],
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["interpreter"],
            "0x0202020202020202020202020202020202020202"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["store"],
            "0x0303030303030303030303030303030303030303"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["bytecode"],
            "0x01020304"
        );

        let valid_inputs = decoded_event["decoded_data"]["order"]["valid_inputs"]
            .as_array()
            .unwrap();
        assert_eq!(valid_inputs.len(), 2);
        assert_eq!(
            valid_inputs[0]["token"],
            "0x0404040404040404040404040404040404040404"
        );
        assert_eq!(
            valid_inputs[0]["vault_id"],
            "0x0000000000000000000000000000000000000000000000000000000000000064"
        );
        assert_eq!(
            valid_inputs[1]["token"],
            "0x0505050505050505050505050505050505050505"
        );
        assert_eq!(
            valid_inputs[1]["vault_id"],
            "0x00000000000000000000000000000000000000000000000000000000000000c8"
        );

        let valid_outputs = decoded_event["decoded_data"]["order"]["valid_outputs"]
            .as_array()
            .unwrap();
        assert_eq!(valid_outputs.len(), 1);
        assert_eq!(
            valid_outputs[0]["token"],
            "0x0606060606060606060606060606060606060606"
        );
        assert_eq!(
            valid_outputs[0]["vault_id"],
            "0x000000000000000000000000000000000000000000000000000000000000012c"
        );
    }

    #[test]
    fn test_take_order_v3_decode() {
        let event_data = create_take_order_v3_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "TakeOrderV3");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0909090909090909090909090909090909090909"
        );
        assert_eq!(
            decoded_event["decoded_data"]["taker_input"],
            "0x00000000000000000000000000000000000000000000000000000000000003e8"
        );
        assert_eq!(
            decoded_event["decoded_data"]["taker_output"],
            "0x00000000000000000000000000000000000000000000000000000000000007d0"
        );

        // Verify config structure
        let config = &decoded_event["decoded_data"]["config"];
        assert_eq!(config["input_io_index"], "0x0");
        assert_eq!(config["output_io_index"], "0x0");

        // Verify order within config
        assert_eq!(
            config["order"]["owner"],
            "0x0101010101010101010101010101010101010101"
        );
        assert_eq!(
            config["order"]["nonce"],
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );

        // Verify signed context
        let signed_context = config["signed_context"].as_array().unwrap();
        assert_eq!(signed_context.len(), 1);
        assert_eq!(
            signed_context[0]["signer"],
            "0x0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"
        );
        assert_eq!(signed_context[0]["signature"], "0x112233");

        let context = signed_context[0]["context"].as_array().unwrap();
        assert_eq!(context.len(), 2);
        assert_eq!(
            context[0],
            "0x000000000000000000000000000000000000000000000000000000000000002a"
        );
        assert_eq!(
            context[1],
            "0x000000000000000000000000000000000000000000000000000000000000002b"
        );
    }

    #[test]
    fn test_withdraw_v2_decode() {
        let event_data = create_withdraw_v2_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "WithdrawV2");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"
        );
        assert_eq!(
            decoded_event["decoded_data"]["token"],
            "0x0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c"
        );
        assert_eq!(
            decoded_event["decoded_data"]["vault_id"],
            "0x00000000000000000000000000000000000000000000000000000000000001f4"
        );
        assert_eq!(
            decoded_event["decoded_data"]["target_amount"],
            "0x0000000000000000000000000000000000000000000000000000000000000bb8"
        );
        assert_eq!(
            decoded_event["decoded_data"]["withdraw_amount"],
            "0x00000000000000000000000000000000000000000000000000000000000009c4"
        );
        assert_eq!(
            decoded_event["decoded_data"]["withdraw_amount_uint256"],
            "0x9c4"
        );
    }

    #[test]
    fn test_deposit_v2_decode() {
        let event_data = create_deposit_v2_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "DepositV2");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d"
        );
        assert_eq!(
            decoded_event["decoded_data"]["token"],
            "0x0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e"
        );
        assert_eq!(
            decoded_event["decoded_data"]["vault_id"],
            "0x0000000000000000000000000000000000000000000000000000000000000258"
        );
        assert_eq!(
            decoded_event["decoded_data"]["deposit_amount_uint256"],
            "0xfa0"
        );
    }

    #[test]
    fn test_remove_order_v3_decode() {
        let event_data = create_remove_order_v3_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "RemoveOrderV3");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order_hash"],
            "0x1010101010101010101010101010101010101010101010101010101010101010"
        );

        // Verify order structure (same as AddOrderV3 but with different sender)
        assert_eq!(
            decoded_event["decoded_data"]["order"]["owner"],
            "0x0101010101010101010101010101010101010101"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["nonce"],
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["interpreter"],
            "0x0202020202020202020202020202020202020202"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["store"],
            "0x0303030303030303030303030303030303030303"
        );
        assert_eq!(
            decoded_event["decoded_data"]["order"]["evaluable"]["bytecode"],
            "0x01020304"
        );
    }

    #[test]
    fn test_clear_v3_decode() {
        let event_data = create_clear_v3_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "ClearV3");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(
            decoded_event["decoded_data"]["alice_owner"],
            "0x0101010101010101010101010101010101010101"
        );
        assert_eq!(
            decoded_event["decoded_data"]["bob_owner"],
            "0x1212121212121212121212121212121212121212"
        );

        // These are computed order hashes, so we just verify they exist and are proper hex
        let alice_hash = decoded_event["decoded_data"]["alice_order_hash"]
            .as_str()
            .unwrap();
        let bob_hash = decoded_event["decoded_data"]["bob_order_hash"]
            .as_str()
            .unwrap();
        assert!(alice_hash.starts_with("0x"));
        assert_eq!(alice_hash.len(), 66); // 0x + 64 hex chars
        assert!(bob_hash.starts_with("0x"));
        assert_eq!(bob_hash.len(), 66);

        // Verify vault IDs are correctly extracted from IO indexes
        assert_eq!(
            decoded_event["decoded_data"]["alice_input_vault_id"],
            "0x0000000000000000000000000000000000000000000000000000000000000064"
        );
        assert_eq!(
            decoded_event["decoded_data"]["alice_output_vault_id"],
            "0x000000000000000000000000000000000000000000000000000000000000012c"
        );
        assert_eq!(
            decoded_event["decoded_data"]["bob_input_vault_id"],
            "0x00000000000000000000000000000000000000000000000000000000000002bc"
        );
        assert_eq!(
            decoded_event["decoded_data"]["bob_output_vault_id"],
            "0x0000000000000000000000000000000000000000000000000000000000000320"
        );
    }

    #[test]
    fn test_after_clear_v2_decode() {
        let event_data = create_after_clear_v2_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "AfterClearV2");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x1717171717171717171717171717171717171717"
        );
        assert_eq!(
            decoded_event["decoded_data"]["alice_input"],
            "0x0000000000000000000000000000000000000000000000000000000000001388"
        );
        assert_eq!(
            decoded_event["decoded_data"]["alice_output"],
            "0x0000000000000000000000000000000000000000000000000000000000001770"
        );
        assert_eq!(
            decoded_event["decoded_data"]["bob_input"],
            "0x0000000000000000000000000000000000000000000000000000000000001b58"
        );
        assert_eq!(
            decoded_event["decoded_data"]["bob_output"],
            "0x0000000000000000000000000000000000000000000000000000000000001f40"
        );
    }

    #[test]
    fn test_meta_v1_2_decode() {
        let event_data = create_meta_v1_2_event_data();
        let events_array = serde_json::json!([event_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "MetaV1_2");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x1818181818181818181818181818181818181818"
        );
        assert_eq!(
            decoded_event["decoded_data"]["subject"],
            "0x0000000000000000000000001919191919191919191919191919191919191919"
        );
        assert_eq!(decoded_event["decoded_data"]["meta"], "0x090a0b0c0d");
    }

    #[test]
    fn test_invalid_hex_data_decode() {
        let invalid_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            "data": "0xinvalidhex",
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([invalid_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::HexDecode(_)));
    }

    #[test]
    fn test_malformed_abi_data_decode() {
        let malformed_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            "data": "0x1234",
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([malformed_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::AbiDecode(_)));
    }

    #[test]
    fn test_unknown_event_type_decode() {
        let unknown_topic = "0x".to_owned() + &"f".repeat(64);
        let unknown_event = serde_json::json!({
            "topics": [unknown_topic],
            "data": "0x1234567890abcdef",
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([unknown_event]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "Unknown");
        assert_eq!(
            decoded_event["decoded_data"]["raw_data"],
            "0x1234567890abcdef"
        );
        assert_eq!(
            decoded_event["decoded_data"]["note"],
            "Unknown event type - could not decode"
        );
    }

    #[test]
    fn test_empty_events_array() {
        let empty_array = serde_json::json!([]);
        let decoded_result = LocalDb::default().decode_events(empty_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_invalid_json_structure() {
        let not_array = serde_json::json!({"not": "an_array"});
        let result = LocalDb::default().decode_events(not_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::InvalidJsonStructure));
    }

    #[test]
    fn test_event_missing_topics() {
        let event_no_topics = serde_json::json!({
            "data": "0x1234567890abcdef",
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([event_no_topics]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_empty_topics_array() {
        let event_empty_topics = serde_json::json!({
            "topics": [],
            "data": "0x1234567890abcdef",
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([event_empty_topics]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_missing_data_field() {
        let event_no_data = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([event_no_data]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_missing_metadata_fields() {
        let event_minimal = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH))],
            "data": "0x1234"
        });

        let events_array = serde_json::json!([event_minimal]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
    }

    #[test]
    fn test_event_with_valid_data_missing_metadata() {
        let event_data = create_add_order_v3_event_data();
        let mut minimal_event = event_data.clone();
        minimal_event.as_object_mut().unwrap().remove("blockNumber");
        minimal_event
            .as_object_mut()
            .unwrap()
            .remove("blockTimestamp");
        minimal_event
            .as_object_mut()
            .unwrap()
            .remove("transactionHash");
        minimal_event.as_object_mut().unwrap().remove("logIndex");

        let events_array = serde_json::json!([minimal_event]);
        let decoded_result = LocalDb::default().decode_events(events_array).unwrap();

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "AddOrderV3");
        assert_eq!(decoded_event["block_number"], "0x0");
        assert_eq!(decoded_event["block_timestamp"], "0x0");
        assert_eq!(decoded_event["transaction_hash"], "");
        assert_eq!(decoded_event["log_index"], "0x0");
    }

    #[test]
    fn test_clear_v3_alice_input_io_index_out_of_bounds() {
        let sender = Address::from([17u8; 20]);
        let alice_order = create_sample_order_v4();
        let bob_order = create_sample_order_v4();
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(5),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        let encoded_data_hex = hex::encode(&encoded_data);
        let clear_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", encoded_data_hex),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([clear_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::AliceInputIOIndexOutOfBounds { index: 5, max: 2 }
        ));
    }

    #[test]
    fn test_clear_v3_alice_output_io_index_out_of_bounds() {
        let sender = Address::from([17u8; 20]);
        let alice_order = create_sample_order_v4();
        let bob_order = create_sample_order_v4();
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(3),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        let encoded_data_hex = hex::encode(&encoded_data);
        let clear_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", encoded_data_hex),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([clear_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::AliceOutputIOIndexOutOfBounds { index: 3, max: 1 }
        ));
    }

    #[test]
    fn test_clear_v3_bob_input_io_index_out_of_bounds() {
        let sender = Address::from([17u8; 20]);
        let alice_order = create_sample_order_v4();
        let bob_order = create_sample_order_v4();
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(10),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        let encoded_data_hex = hex::encode(&encoded_data);
        let clear_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", encoded_data_hex),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([clear_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::BobInputIOIndexOutOfBounds { index: 10, max: 2 }
        ));
    }

    #[test]
    fn test_clear_v3_bob_output_io_index_out_of_bounds() {
        let sender = Address::from([17u8; 20]);
        let alice_order = create_sample_order_v4();
        let bob_order = create_sample_order_v4();
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(7),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        let encoded_data_hex = hex::encode(&encoded_data);
        let clear_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", encoded_data_hex),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([clear_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::BobOutputIOIndexOutOfBounds { index: 7, max: 1 }
        ));
    }

    #[test]
    fn test_clear_v3_empty_valid_inputs_array() {
        let sender = Address::from([17u8; 20]);
        let mut alice_order = create_sample_order_v4();
        alice_order.validInputs = vec![];
        let bob_order = create_sample_order_v4();
        let clear_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };

        let event_data = ClearV3 {
            sender,
            alice: alice_order,
            bob: bob_order,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        let encoded_data_hex = hex::encode(&encoded_data);
        let clear_event = serde_json::json!({
            "topics": [format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH))],
            "data": format!("0x{}", encoded_data_hex),
            "blockNumber": "0x123456",
            "blockTimestamp": "0x64b8c123",
            "transactionHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "logIndex": "0x0"
        });

        let events_array = serde_json::json!([clear_event]);
        let result = LocalDb::default().decode_events(events_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::AliceInputIOIndexOutOfBounds { index: 0, max: 0 }
        ));
    }
}
