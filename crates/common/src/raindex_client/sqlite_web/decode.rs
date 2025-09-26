use crate::hyper_rpc::LogEntryResponse;
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
use serde::Serialize;
use std::fmt::LowerHex;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum EventType {
    AddOrderV3,
    TakeOrderV3,
    WithdrawV2,
    DepositV2,
    RemoveOrderV3,
    ClearV3,
    AfterClearV2,
    MetaV1_2,
    Unknown,
}

impl EventType {
    fn from_topic(topic: &str) -> Self {
        if let Ok(bytes) = hex::decode(topic) {
            if bytes == AddOrderV3::SIGNATURE_HASH.as_slice() {
                return Self::AddOrderV3;
            }
            if bytes == TakeOrderV3::SIGNATURE_HASH.as_slice() {
                return Self::TakeOrderV3;
            }
            if bytes == WithdrawV2::SIGNATURE_HASH.as_slice() {
                return Self::WithdrawV2;
            }
            if bytes == DepositV2::SIGNATURE_HASH.as_slice() {
                return Self::DepositV2;
            }
            if bytes == RemoveOrderV3::SIGNATURE_HASH.as_slice() {
                return Self::RemoveOrderV3;
            }
            if bytes == ClearV3::SIGNATURE_HASH.as_slice() {
                return Self::ClearV3;
            }
            if bytes == AfterClearV2::SIGNATURE_HASH.as_slice() {
                return Self::AfterClearV2;
            }
            if bytes == MetaV1_2::SIGNATURE_HASH.as_slice() {
                return Self::MetaV1_2;
            }
        }

        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(bound(serialize = "T: Serialize"))]
pub struct DecodedEventData<T> {
    pub event_type: EventType,
    pub block_number: String,
    pub block_timestamp: String,
    pub transaction_hash: String,
    pub log_index: String,
    pub decoded_data: T,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DecodedEvent {
    AddOrderV3(AddOrderV3Decoded),
    TakeOrderV3(TakeOrderV3Decoded),
    WithdrawV2(WithdrawV2Decoded),
    DepositV2(DepositV2Decoded),
    RemoveOrderV3(RemoveOrderV3Decoded),
    ClearV3(ClearV3Decoded),
    AfterClearV2(AfterClearV2Decoded),
    MetaV1_2(MetaV1_2Decoded),
    Unknown(UnknownEventDecoded),
}

pub fn decode_events(
    events: &[LogEntryResponse],
) -> Result<Vec<DecodedEventData<DecodedEvent>>, DecodeError> {
    let mut decoded_events = Vec::with_capacity(events.len());

    for event in events {
        let Some(topic0) = event.topics.first() else {
            continue;
        };
        if event.data.trim().is_empty() {
            continue;
        }
        let event_type = EventType::from_topic(topic0);

        let decoded_data = match event_type {
            EventType::AddOrderV3 => DecodedEvent::AddOrderV3(decode_add_order_v3(&event.data)?),
            EventType::TakeOrderV3 => DecodedEvent::TakeOrderV3(decode_take_order_v3(&event.data)?),
            EventType::WithdrawV2 => DecodedEvent::WithdrawV2(decode_withdraw_v2(&event.data)?),
            EventType::DepositV2 => DecodedEvent::DepositV2(decode_deposit_v2(&event.data)?),
            EventType::RemoveOrderV3 => {
                DecodedEvent::RemoveOrderV3(decode_remove_order_v3(&event.data)?)
            }
            EventType::ClearV3 => DecodedEvent::ClearV3(decode_clear_v3(&event.data)?),
            EventType::AfterClearV2 => {
                DecodedEvent::AfterClearV2(decode_after_clear_v2(&event.data)?)
            }
            EventType::MetaV1_2 => DecodedEvent::MetaV1_2(decode_meta_v1_2(&event.data)?),
            EventType::Unknown => DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: event.data.clone(),
                note: "Unknown event type - could not decode".to_string(),
            }),
        };

        decoded_events.push(DecodedEventData {
            event_type,
            block_number: if event.block_number.is_empty() {
                "0x0".to_string()
            } else {
                event.block_number.clone()
            },
            block_timestamp: match event.block_timestamp.clone() {
                Some(ts) if !ts.is_empty() => ts,
                _ => "0x0".to_string(),
            },
            transaction_hash: event.transaction_hash.clone(),
            log_index: if event.log_index.is_empty() {
                "0x0".to_string()
            } else {
                event.log_index.clone()
            },
            decoded_data,
        });
    }

    Ok(decoded_events)
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderEvaluableDecoded {
    pub interpreter: String,
    pub store: String,
    pub bytecode: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderIoDecoded {
    pub token: String,
    pub vault_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderDecoded {
    pub owner: String,
    pub nonce: String,
    pub evaluable: OrderEvaluableDecoded,
    pub valid_inputs: Vec<OrderIoDecoded>,
    pub valid_outputs: Vec<OrderIoDecoded>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddOrderV3Decoded {
    pub sender: String,
    pub order_hash: String,
    pub order: OrderDecoded,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignedContextDecoded {
    pub signer: String,
    pub context: Vec<String>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TakeOrderConfigDecoded {
    pub order: OrderDecoded,
    pub input_io_index: String,
    pub output_io_index: String,
    pub signed_context: Vec<SignedContextDecoded>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TakeOrderV3Decoded {
    pub sender: String,
    pub config: TakeOrderConfigDecoded,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WithdrawV2Decoded {
    pub sender: String,
    pub token: String,
    pub vault_id: String,
    pub target_amount: String,
    pub withdraw_amount: String,
    pub withdraw_amount_uint256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepositV2Decoded {
    pub sender: String,
    pub token: String,
    pub vault_id: String,
    pub deposit_amount_uint256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveOrderV3Decoded {
    pub sender: String,
    pub order_hash: String,
    pub order: OrderDecoded,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClearV3Decoded {
    pub sender: String,
    pub alice_owner: String,
    pub bob_owner: String,
    pub alice_order_hash: String,
    pub bob_order_hash: String,
    pub alice_input_vault_id: String,
    pub alice_output_vault_id: String,
    pub bob_input_vault_id: String,
    pub bob_output_vault_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AfterClearV2Decoded {
    pub sender: String,
    pub alice_input: String,
    pub alice_output: String,
    pub bob_input: String,
    pub bob_output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaV1_2Decoded {
    pub sender: String,
    pub subject: String,
    pub meta: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnknownEventDecoded {
    pub raw_data: String,
    pub note: String,
}

fn to_prefixed_hex<T>(value: T) -> String
where
    T: LowerHex,
{
    format!("0x{:x}", value)
}

fn compute_order_hash(order: &OrderV4) -> Result<String, DecodeError> {
    let encoded = order.abi_encode();
    let hash = keccak256(&encoded);
    Ok(hex::encode_prefixed(hash))
}

fn order_from_v4(order: &OrderV4) -> OrderDecoded {
    OrderDecoded {
        owner: to_prefixed_hex(order.owner),
        nonce: to_prefixed_hex(order.nonce),
        evaluable: OrderEvaluableDecoded {
            interpreter: to_prefixed_hex(order.evaluable.interpreter),
            store: to_prefixed_hex(order.evaluable.store),
            bytecode: hex::encode_prefixed(&order.evaluable.bytecode),
        },
        valid_inputs: order
            .validInputs
            .iter()
            .map(|input| OrderIoDecoded {
                token: to_prefixed_hex(input.token),
                vault_id: to_prefixed_hex(input.vaultId),
            })
            .collect(),
        valid_outputs: order
            .validOutputs
            .iter()
            .map(|output| OrderIoDecoded {
                token: to_prefixed_hex(output.token),
                vault_id: to_prefixed_hex(output.vaultId),
            })
            .collect(),
    }
}

fn decode_add_order_v3(data_str: &str) -> Result<AddOrderV3Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match AddOrderV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(AddOrderV3Decoded {
            sender: to_prefixed_hex(decoded.0),
            order_hash: hex::encode_prefixed(decoded.1),
            order: order_from_v4(&decoded.2),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_take_order_v3(data_str: &str) -> Result<TakeOrderV3Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match TakeOrderV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(TakeOrderV3Decoded {
            sender: to_prefixed_hex(decoded.0),
            config: TakeOrderConfigDecoded {
                order: order_from_v4(&decoded.1.order),
                input_io_index: to_prefixed_hex(decoded.1.inputIOIndex),
                output_io_index: to_prefixed_hex(decoded.1.outputIOIndex),
                signed_context: decoded
                    .1
                    .signedContext
                    .iter()
                    .map(|ctx| SignedContextDecoded {
                        signer: to_prefixed_hex(ctx.signer),
                        context: ctx.context.iter().map(|c| to_prefixed_hex(*c)).collect(),
                        signature: hex::encode_prefixed(&ctx.signature),
                    })
                    .collect(),
            },
            input: to_prefixed_hex(decoded.2),
            output: to_prefixed_hex(decoded.3),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_withdraw_v2(data_str: &str) -> Result<WithdrawV2Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match WithdrawV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(WithdrawV2Decoded {
            sender: to_prefixed_hex(decoded.0),
            token: to_prefixed_hex(decoded.1),
            vault_id: to_prefixed_hex(decoded.2),
            target_amount: to_prefixed_hex(decoded.3),
            withdraw_amount: to_prefixed_hex(decoded.4),
            withdraw_amount_uint256: to_prefixed_hex(decoded.5),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_deposit_v2(data_str: &str) -> Result<DepositV2Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match DepositV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(DepositV2Decoded {
            sender: to_prefixed_hex(decoded.0),
            token: to_prefixed_hex(decoded.1),
            vault_id: to_prefixed_hex(decoded.2),
            deposit_amount_uint256: to_prefixed_hex(decoded.3),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_remove_order_v3(data_str: &str) -> Result<RemoveOrderV3Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match RemoveOrderV3::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(RemoveOrderV3Decoded {
            sender: to_prefixed_hex(decoded.0),
            order_hash: hex::encode_prefixed(decoded.1),
            order: order_from_v4(&decoded.2),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_clear_v3(data_str: &str) -> Result<ClearV3Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

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

            Ok(ClearV3Decoded {
                sender: to_prefixed_hex(decoded.0),
                alice_owner: to_prefixed_hex(decoded.1.owner),
                bob_owner: to_prefixed_hex(decoded.2.owner),
                alice_order_hash,
                bob_order_hash,
                alice_input_vault_id,
                alice_output_vault_id,
                bob_input_vault_id,
                bob_output_vault_id,
            })
        }
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_after_clear_v2(data_str: &str) -> Result<AfterClearV2Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match AfterClearV2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(AfterClearV2Decoded {
            sender: to_prefixed_hex(decoded.0),
            alice_input: to_prefixed_hex(decoded.1.aliceInput),
            alice_output: to_prefixed_hex(decoded.1.aliceOutput),
            bob_input: to_prefixed_hex(decoded.1.bobInput),
            bob_output: to_prefixed_hex(decoded.1.bobOutput),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

fn decode_meta_v1_2(data_str: &str) -> Result<MetaV1_2Decoded, DecodeError> {
    let data_bytes = hex::decode(data_str).map_err(DecodeError::HexDecode)?;

    match MetaV1_2::abi_decode_data(&data_bytes) {
        Ok(decoded) => Ok(MetaV1_2Decoded {
            sender: to_prefixed_hex(decoded.0),
            subject: to_prefixed_hex(decoded.1),
            meta: hex::encode_prefixed(&decoded.2),
        }),
        Err(e) => Err(DecodeError::AbiDecode(e.to_string())),
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use crate::hyper_rpc::LogEntryResponse;
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

    fn new_log_entry(
        topic: String,
        data: String,
        block_number: &str,
        block_timestamp: Option<&str>,
        transaction_hash: &str,
        log_index: &str,
    ) -> LogEntryResponse {
        LogEntryResponse {
            address: "0x0000000000000000000000000000000000000000".to_string(),
            topics: vec![topic],
            data,
            block_number: block_number.to_string(),
            block_timestamp: block_timestamp.map(|ts| ts.to_string()),
            transaction_hash: transaction_hash.to_string(),
            transaction_index: "0x0".to_string(),
            block_hash: "0x0".to_string(),
            log_index: log_index.to_string(),
            removed: false,
        }
    }

    fn decode_events_json(events: Vec<LogEntryResponse>) -> serde_json::Value {
        let decoded = decode_events(&events).unwrap();
        serde_json::to_value(decoded).unwrap()
    }

    fn build_clear_log(
        sender: Address,
        alice: OrderV4,
        bob: OrderV4,
        clear_config: ClearConfigV2,
    ) -> LogEntryResponse {
        let event_data = ClearV3 {
            sender,
            alice,
            bob,
            clearConfig: clear_config,
        };

        let encoded_data = event_data.encode_data();
        new_log_entry(
            format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x123456",
            Some("0x64b8c123"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x0",
        )
    }

    fn create_add_order_v3_event_data() -> LogEntryResponse {
        let sender = Address::from([7u8; 20]);
        let order_hash = FixedBytes::<32>::from([8u8; 32]);
        let order = create_sample_order_v4();

        let event_data = AddOrderV3 {
            sender,
            orderHash: order_hash,
            order,
        };

        let encoded_data = event_data.encode_data();
        new_log_entry(
            format!("0x{}", hex::encode(AddOrderV3::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x123456",
            Some("0x64b8c123"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x0",
        )
    }

    fn create_take_order_v3_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(TakeOrderV3::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x123457",
            Some("0x64b8c124"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567891",
            "0x1",
        )
    }

    fn create_withdraw_v2_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(WithdrawV2::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x123458",
            Some("0x64b8c125"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567892",
            "0x2",
        )
    }

    fn create_deposit_v2_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(DepositV2::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x123459",
            Some("0x64b8c126"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567893",
            "0x3",
        )
    }

    fn create_remove_order_v3_event_data() -> LogEntryResponse {
        let sender = Address::from([15u8; 20]);
        let order_hash = FixedBytes::<32>::from([16u8; 32]);
        let order = create_sample_order_v4();

        let event_data = RemoveOrderV3 {
            sender,
            orderHash: order_hash,
            order,
        };

        let encoded_data = event_data.encode_data();
        new_log_entry(
            format!("0x{}", hex::encode(RemoveOrderV3::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x12345a",
            Some("0x64b8c127"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567894",
            "0x4",
        )
    }

    fn create_clear_v3_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(ClearV3::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x12345b",
            Some("0x64b8c128"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567895",
            "0x5",
        )
    }

    fn create_after_clear_v2_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(AfterClearV2::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x12345c",
            Some("0x64b8c129"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567896",
            "0x6",
        )
    }

    fn create_meta_v1_2_event_data() -> LogEntryResponse {
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
        new_log_entry(
            format!("0x{}", hex::encode(MetaV1_2::SIGNATURE_HASH)),
            format!("0x{}", hex::encode(encoded_data)),
            "0x12345d",
            Some("0x64b8c12a"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567897",
            "0x7",
        )
    }

    #[test]
    fn test_add_order_v3_decode() {
        let event_data = create_add_order_v3_event_data();
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event["event_type"], "TakeOrderV3");
        assert_eq!(
            decoded_event["decoded_data"]["sender"],
            "0x0909090909090909090909090909090909090909"
        );
        assert_eq!(
            decoded_event["decoded_data"]["input"],
            "0x00000000000000000000000000000000000000000000000000000000000003e8"
        );
        assert_eq!(
            decoded_event["decoded_data"]["output"],
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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let decoded_result = decode_events_json(vec![event_data]);

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
        let mut invalid_event = create_add_order_v3_event_data();
        invalid_event.data = "0xinvalidhex".to_string();

        let result = decode_events(&[invalid_event]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::HexDecode(_)));
    }

    #[test]
    fn test_malformed_abi_data_decode() {
        let mut malformed_event = create_add_order_v3_event_data();
        malformed_event.data = "0x1234".to_string();

        let result = decode_events(&[malformed_event]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DecodeError::AbiDecode(_)));
    }

    #[test]
    fn test_unknown_event_type_decode() {
        let unknown_topic = "0x".to_owned() + &"f".repeat(64);
        let mut unknown_event = create_add_order_v3_event_data();
        unknown_event.topics = vec![unknown_topic];
        unknown_event.data = "0x1234567890abcdef".to_string();

        let decoded_result = decode_events_json(vec![unknown_event]);

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
        let decoded_result = decode_events_json(Vec::new());

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_empty_topics_array() {
        let mut event_empty_topics = create_add_order_v3_event_data();
        event_empty_topics.topics.clear();
        event_empty_topics.data = "0x1234567890abcdef".to_string();

        let decoded_result = decode_events_json(vec![event_empty_topics]);

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_missing_data_field() {
        let mut event_no_data = create_add_order_v3_event_data();
        event_no_data.data = String::new();

        let decoded_result = decode_events_json(vec![event_no_data]);

        let decoded_events = decoded_result.as_array().unwrap();
        assert_eq!(decoded_events.len(), 0);
    }

    #[test]
    fn test_event_with_valid_data_missing_metadata() {
        let mut minimal_event = create_add_order_v3_event_data();
        minimal_event.block_number.clear();
        minimal_event.block_timestamp = None;
        minimal_event.transaction_hash.clear();
        minimal_event.log_index.clear();

        let decoded_result = decode_events_json(vec![minimal_event]);

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

        let clear_event = build_clear_log(sender, alice_order, bob_order, clear_config);
        let result = decode_events(&[clear_event]);

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

        let clear_event = build_clear_log(sender, alice_order, bob_order, clear_config);
        let result = decode_events(&[clear_event]);

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

        let clear_event = build_clear_log(sender, alice_order, bob_order, clear_config);
        let result = decode_events(&[clear_event]);

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
        let clear_event = build_clear_log(sender, alice_order, bob_order, clear_config);
        let result = decode_events(&[clear_event]);

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
        let clear_event = build_clear_log(sender, alice_order, bob_order, clear_config);
        let result = decode_events(&[clear_event]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            DecodeError::AliceInputIOIndexOutOfBounds { index: 0, max: 0 }
        ));
    }
}
