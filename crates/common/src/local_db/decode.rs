use crate::local_db::LocalDbError;
use crate::rpc_client::LogEntryResponse;
use alloy::{
    hex,
    primitives::{Address, Bytes, FixedBytes, B256},
    sol_types::{abi::token::WordToken, SolEvent},
};
use core::convert::{TryFrom, TryInto};
use rain_orderbook_bindings::{
    IInterpreterStoreV3::Set,
    IOrderBookV5::{
        AddOrderV3, AfterClearV2, ClearV3, DepositV2, RemoveOrderV3, TakeOrderV3, WithdrawV2,
    },
    OrderBook::MetaV1_2,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    #[error("ABI decode error: {0}")]
    AbiDecode(String),
    #[error("log at index {index} missing required field {field}")]
    MissingRequiredField { field: &'static str, index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    AddOrderV3,
    TakeOrderV3,
    WithdrawV2,
    DepositV2,
    RemoveOrderV3,
    ClearV3,
    AfterClearV2,
    MetaV1_2,
    InterpreterStoreSet,
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
            if bytes == Set::SIGNATURE_HASH.as_slice() {
                return Self::InterpreterStoreSet;
            }
        }

        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct DecodedEventData<T> {
    pub event_type: EventType,
    pub block_number: String,
    pub block_timestamp: String,
    pub transaction_hash: Bytes,
    pub log_index: String,
    pub decoded_data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecodedEvent {
    AddOrderV3(Box<AddOrderV3>),
    TakeOrderV3(Box<TakeOrderV3>),
    WithdrawV2(Box<WithdrawV2>),
    DepositV2(Box<DepositV2>),
    RemoveOrderV3(Box<RemoveOrderV3>),
    ClearV3(Box<ClearV3>),
    AfterClearV2(Box<AfterClearV2>),
    MetaV1_2(Box<MetaV1_2>),
    InterpreterStoreSet(Box<InterpreterStoreSetEvent>),
    Unknown(UnknownEventDecoded),
}

pub fn decode_events(
    events: &[LogEntryResponse],
) -> Result<Vec<DecodedEventData<DecodedEvent>>, DecodeError> {
    events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let topic0 = event
                .topics
                .first()
                .ok_or(DecodeError::MissingRequiredField {
                    field: "topic0",
                    index,
                })?;
            let trimmed_data = event.data.trim();
            let data_without_prefix = trimmed_data
                .strip_prefix("0x")
                .or_else(|| trimmed_data.strip_prefix("0X"))
                .unwrap_or(trimmed_data);
            if data_without_prefix.trim().is_empty() {
                return Err(DecodeError::MissingRequiredField {
                    field: "data",
                    index,
                });
            }
            let event_type = EventType::from_topic(topic0);

            let decoded_data = match event_type {
                EventType::AddOrderV3 => {
                    DecodedEvent::AddOrderV3(Box::new(decode_event::<AddOrderV3>(event)?))
                }
                EventType::TakeOrderV3 => {
                    DecodedEvent::TakeOrderV3(Box::new(decode_event::<TakeOrderV3>(event)?))
                }
                EventType::WithdrawV2 => {
                    DecodedEvent::WithdrawV2(Box::new(decode_event::<WithdrawV2>(event)?))
                }
                EventType::DepositV2 => {
                    DecodedEvent::DepositV2(Box::new(decode_event::<DepositV2>(event)?))
                }
                EventType::RemoveOrderV3 => {
                    DecodedEvent::RemoveOrderV3(Box::new(decode_event::<RemoveOrderV3>(event)?))
                }
                EventType::ClearV3 => {
                    DecodedEvent::ClearV3(Box::new(decode_event::<ClearV3>(event)?))
                }
                EventType::AfterClearV2 => {
                    DecodedEvent::AfterClearV2(Box::new(decode_event::<AfterClearV2>(event)?))
                }
                EventType::MetaV1_2 => {
                    DecodedEvent::MetaV1_2(Box::new(decode_event::<MetaV1_2>(event)?))
                }
                EventType::InterpreterStoreSet => {
                    DecodedEvent::InterpreterStoreSet(Box::new(decode_store_set_event(event)?))
                }
                EventType::Unknown => DecodedEvent::Unknown(UnknownEventDecoded {
                    raw_data: event.data.clone(),
                    note: "Unknown event type - could not decode".to_string(),
                }),
            };

            Ok(DecodedEventData {
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
                transaction_hash: Bytes::from_str(&event.transaction_hash)?,
                log_index: if event.log_index.is_empty() {
                    "0x0".to_string()
                } else {
                    event.log_index.clone()
                },
                decoded_data,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownEventDecoded {
    pub raw_data: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterStoreSetEvent {
    pub store_address: Address,
    pub namespace: FixedBytes<32>,
    pub key: FixedBytes<32>,
    pub value: FixedBytes<32>,
}

fn decode_event<E: SolEvent>(event: &LogEntryResponse) -> Result<E, DecodeError> {
    let topics = event
        .topics
        .iter()
        .map(|topic| {
            let bytes = hex::decode(topic).map_err(DecodeError::HexDecode)?;
            let b256 = B256::try_from(bytes.as_slice())
                .map_err(|_| DecodeError::AbiDecode("topic length != 32 bytes".to_string()))?;
            Ok::<WordToken, DecodeError>(WordToken::from(b256))
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let data = hex::decode(&event.data).map_err(DecodeError::HexDecode)?;
    E::decode_raw_log(topics, &data).map_err(|err| DecodeError::AbiDecode(err.to_string()))
}

fn decode_store_set_event(
    event: &LogEntryResponse,
) -> Result<InterpreterStoreSetEvent, DecodeError> {
    let data = hex::decode(event.data.trim_start_matches("0x")).map_err(DecodeError::HexDecode)?;
    if data.len() < 96 {
        return Err(DecodeError::AbiDecode(
            "Set event data is too short".to_string(),
        ));
    }

    let namespace_bytes: [u8; 32] = data[0..32]
        .try_into()
        .map_err(|_| DecodeError::AbiDecode("Invalid namespace length".to_string()))?;
    let key_bytes: [u8; 32] = data[32..64]
        .try_into()
        .map_err(|_| DecodeError::AbiDecode("Invalid key length".to_string()))?;
    let value_bytes: [u8; 32] = data[64..96]
        .try_into()
        .map_err(|_| DecodeError::AbiDecode("Invalid value length".to_string()))?;

    let store_address = Address::from_str(&event.address).map_err(|err| {
        DecodeError::AbiDecode(format!("Invalid store address {}: {}", event.address, err))
    })?;

    Ok(InterpreterStoreSetEvent {
        store_address,
        namespace: FixedBytes::from(namespace_bytes),
        key: FixedBytes::from(key_bytes),
        value: FixedBytes::from(value_bytes),
    })
}

pub fn sort_decoded_events_by_block_and_log(
    events: &mut [DecodedEventData<DecodedEvent>],
) -> Result<(), LocalDbError> {
    let mut keyed = Vec::with_capacity(events.len());
    for (idx, event) in events.iter().enumerate() {
        let block = parse_u64_hex_or_dec(&event.block_number).map_err(|err| {
            LocalDbError::InvalidBlockNumberString {
                value: event.block_number.clone(),
                source: err,
            }
        })?;
        let log_index = parse_u64_hex_or_dec(&event.log_index).map_err(|err| {
            LocalDbError::InvalidLogIndex {
                value: event.log_index.clone(),
                source: err,
            }
        })?;
        keyed.push((idx, block, log_index));
    }

    keyed.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.0.cmp(&b.0))
    });

    let original = events.to_vec();
    for (position, (idx, _, _)) in keyed.into_iter().enumerate() {
        events[position] = original[idx].clone();
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
mod test_helpers {
    use super::*;
    use crate::local_db::LocalDbError;
    use crate::rpc_client::LogEntryResponse;
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
    use std::str::FromStr;

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

    fn fixed_u256(value: u64) -> FixedBytes<32> {
        FixedBytes::<32>::from(U256::from(value).to_be_bytes::<32>())
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

    fn decode_events_vec(events: Vec<LogEntryResponse>) -> Vec<DecodedEventData<DecodedEvent>> {
        decode_events(&events).unwrap()
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

    fn create_store_set_log() -> LogEntryResponse {
        let namespace = [0x33u8; 32];
        let key = [0x44u8; 32];
        let value = [0x55u8; 32];

        let mut data = Vec::with_capacity(96);
        data.extend_from_slice(&namespace);
        data.extend_from_slice(&key);
        data.extend_from_slice(&value);

        let encoded = format!("0x{}", hex::encode(data));
        let mut entry = new_log_entry(
            Set::SIGNATURE_HASH.to_string(),
            encoded,
            "0x12345c",
            Some("0x64b8c129"),
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567896",
            "0x6",
        );
        entry.address = "0x0123456789abcdef0123456789abcdef01234567".to_string();
        entry
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
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::AddOrderV3);

        let DecodedEvent::AddOrderV3(add_order) = &decoded_event.decoded_data else {
            panic!("expected AddOrderV3 decoded data");
        };

        let add_order = add_order.as_ref();
        assert_eq!(add_order.sender, Address::from([7u8; 20]));
        assert_eq!(add_order.orderHash, FixedBytes::<32>::from([8u8; 32]));

        let expected_order = create_sample_order_v4();
        assert_eq!(add_order.order, expected_order);
    }

    #[test]
    fn test_take_order_v3_decode() {
        let event_data = create_take_order_v3_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::TakeOrderV3);

        let DecodedEvent::TakeOrderV3(take_order) = &decoded_event.decoded_data else {
            panic!("expected TakeOrderV3 decoded data");
        };

        let take_order = take_order.as_ref();
        assert_eq!(take_order.sender, Address::from([9u8; 20]));
        assert_eq!(take_order.input, fixed_u256(1000));
        assert_eq!(take_order.output, fixed_u256(2000));

        let expected_order = create_sample_order_v4();
        assert_eq!(take_order.config.order, expected_order);
        assert_eq!(take_order.config.inputIOIndex, U256::from(0));
        assert_eq!(take_order.config.outputIOIndex, U256::from(0));

        let signed_context = &take_order.config.signedContext;
        assert_eq!(signed_context.len(), 1);
        let ctx = &signed_context[0];
        assert_eq!(ctx.signer, Address::from([10u8; 20]));
        assert_eq!(ctx.context, vec![fixed_u256(42), fixed_u256(43)]);
        assert_eq!(ctx.signature, Bytes::from(vec![0x11, 0x22, 0x33]));
    }

    #[test]
    fn test_withdraw_v2_decode() {
        let event_data = create_withdraw_v2_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::WithdrawV2);

        let DecodedEvent::WithdrawV2(withdraw) = &decoded_event.decoded_data else {
            panic!("expected WithdrawV2 decoded data");
        };

        let withdraw = withdraw.as_ref();
        assert_eq!(withdraw.sender, Address::from([11u8; 20]));
        assert_eq!(withdraw.token, Address::from([12u8; 20]));
        assert_eq!(withdraw.vaultId, fixed_u256(500));
        assert_eq!(withdraw.targetAmount, fixed_u256(3000));
        assert_eq!(withdraw.withdrawAmount, fixed_u256(2500));
        assert_eq!(withdraw.withdrawAmountUint256, U256::from(2500));
    }

    #[test]
    fn test_deposit_v2_decode() {
        let event_data = create_deposit_v2_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::DepositV2);

        let DecodedEvent::DepositV2(deposit) = &decoded_event.decoded_data else {
            panic!("expected DepositV2 decoded data");
        };

        let deposit = deposit.as_ref();
        assert_eq!(deposit.sender, Address::from([13u8; 20]));
        assert_eq!(deposit.token, Address::from([14u8; 20]));
        assert_eq!(deposit.vaultId, fixed_u256(600));
        assert_eq!(deposit.depositAmountUint256, U256::from(4000));
    }

    #[test]
    fn test_remove_order_v3_decode() {
        let event_data = create_remove_order_v3_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::RemoveOrderV3);

        let DecodedEvent::RemoveOrderV3(remove_order) = &decoded_event.decoded_data else {
            panic!("expected RemoveOrderV3 decoded data");
        };

        let remove_order = remove_order.as_ref();
        assert_eq!(remove_order.sender, Address::from([15u8; 20]));
        assert_eq!(remove_order.orderHash, FixedBytes::<32>::from([16u8; 32]));

        let expected_order = create_sample_order_v4();
        assert_eq!(remove_order.order, expected_order);
    }

    #[test]
    fn test_set_decode() {
        let log_entry = create_store_set_log();
        let decoded = decode_events(&[log_entry]).unwrap();

        assert_eq!(decoded.len(), 1);
        let event = &decoded[0];
        assert_eq!(event.event_type, EventType::InterpreterStoreSet);

        match &event.decoded_data {
            DecodedEvent::InterpreterStoreSet(data) => {
                assert_eq!(
                    data.store_address,
                    Address::from_str("0x0123456789abcdef0123456789abcdef01234567").unwrap()
                );
                assert_eq!(data.namespace, FixedBytes::<32>::from([0x33u8; 32]));
                assert_eq!(data.key, FixedBytes::<32>::from([0x44u8; 32]));
                assert_eq!(data.value, FixedBytes::<32>::from([0x55u8; 32]));
            }
            other => panic!("expected InterpreterStoreSet, got {other:?}"),
        }
    }

    #[test]
    fn test_clear_v3_decode() {
        let event_data = create_clear_v3_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::ClearV3);

        let DecodedEvent::ClearV3(clear) = &decoded_event.decoded_data else {
            panic!("expected ClearV3 decoded data");
        };

        let clear = clear.as_ref();
        assert_eq!(clear.sender, Address::from([17u8; 20]));

        let expected_alice = create_sample_order_v4();
        assert_eq!(clear.alice, expected_alice);

        let expected_bob = OrderV4 {
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
        assert_eq!(clear.bob, expected_bob);

        let expected_config = ClearConfigV2 {
            aliceInputIOIndex: U256::from(0),
            aliceOutputIOIndex: U256::from(0),
            bobInputIOIndex: U256::from(0),
            bobOutputIOIndex: U256::from(0),
            aliceBountyVaultId: U256::from(0).into(),
            bobBountyVaultId: U256::from(0).into(),
        };
        assert_eq!(clear.clearConfig, expected_config);
    }

    #[test]
    fn test_after_clear_v2_decode() {
        let event_data = create_after_clear_v2_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::AfterClearV2);

        let DecodedEvent::AfterClearV2(after_clear) = &decoded_event.decoded_data else {
            panic!("expected AfterClearV2 decoded data");
        };

        let after_clear = after_clear.as_ref();
        assert_eq!(after_clear.sender, Address::from([23u8; 20]));
        assert_eq!(after_clear.clearStateChange.aliceInput, fixed_u256(5000));
        assert_eq!(after_clear.clearStateChange.aliceOutput, fixed_u256(6000));
        assert_eq!(after_clear.clearStateChange.bobInput, fixed_u256(7000));
        assert_eq!(after_clear.clearStateChange.bobOutput, fixed_u256(8000));
    }

    #[test]
    fn test_meta_v1_2_decode() {
        let event_data = create_meta_v1_2_event_data();
        let decoded_events = decode_events_vec(vec![event_data]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::MetaV1_2);

        let DecodedEvent::MetaV1_2(meta_event) = &decoded_event.decoded_data else {
            panic!("expected MetaV1_2 decoded data");
        };

        let meta_event = meta_event.as_ref();
        assert_eq!(meta_event.sender, Address::from([24u8; 20]));
        let mut expected_subject = [0u8; 32];
        expected_subject[12..32].copy_from_slice(&Address::from([25u8; 20])[..]);
        assert_eq!(meta_event.subject, FixedBytes::<32>::from(expected_subject));
        assert_eq!(
            meta_event.meta,
            Bytes::from(vec![0x09, 0x0a, 0x0b, 0x0c, 0x0d])
        );
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

        let decoded_events = decode_events_vec(vec![unknown_event]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::Unknown);

        let DecodedEvent::Unknown(unknown) = &decoded_event.decoded_data else {
            panic!("expected Unknown decoded data");
        };

        assert_eq!(unknown.raw_data, "0x1234567890abcdef");
        assert_eq!(unknown.note, "Unknown event type - could not decode");
    }

    #[test]
    fn test_empty_events_array() {
        let decoded_events = decode_events_vec(Vec::new());
        assert!(decoded_events.is_empty());
    }

    #[test]
    fn test_event_empty_topics_array() {
        let mut event_empty_topics = create_add_order_v3_event_data();
        event_empty_topics.topics.clear();
        event_empty_topics.data = "0x1234567890abcdef".to_string();

        let result = decode_events(&[event_empty_topics]);
        assert!(matches!(
            result,
            Err(DecodeError::MissingRequiredField { field, index })
                if field == "topic0" && index == 0
        ));
    }

    #[test]
    fn test_event_missing_data_field() {
        let mut event_no_data = create_add_order_v3_event_data();
        event_no_data.data = String::new();

        let result = decode_events(&[event_no_data]);
        assert!(matches!(
            result,
            Err(DecodeError::MissingRequiredField { field, index })
                if field == "data" && index == 0
        ));
    }

    #[test]
    fn test_event_zero_length_hex_data() {
        for hex_prefix in ["0x", "0X"] {
            let mut event_no_data = create_add_order_v3_event_data();
            event_no_data.data = hex_prefix.to_string();

            let result = decode_events(&[event_no_data]);
            assert!(matches!(
                result,
                Err(DecodeError::MissingRequiredField { field, index })
                    if field == "data" && index == 0
            ));
        }
    }

    #[test]
    fn test_event_with_valid_data_missing_metadata() {
        let mut minimal_event = create_add_order_v3_event_data();
        minimal_event.block_number.clear();
        minimal_event.block_timestamp = None;
        minimal_event.transaction_hash.clear();
        minimal_event.log_index.clear();

        let decoded_events = decode_events_vec(vec![minimal_event]);
        assert_eq!(decoded_events.len(), 1);

        let decoded_event = &decoded_events[0];
        assert_eq!(decoded_event.event_type, EventType::AddOrderV3);
        assert_eq!(decoded_event.block_number, "0x0");
        assert_eq!(decoded_event.block_timestamp, "0x0");
        assert_eq!(decoded_event.transaction_hash, Bytes::from_str("").unwrap());
        assert_eq!(decoded_event.log_index, "0x0");
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
        let result = decode_events(&[clear_event]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].decoded_data, DecodedEvent::ClearV3(_)));
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
        let result = decode_events(&[clear_event]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].decoded_data, DecodedEvent::ClearV3(_)));
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
        let result = decode_events(&[clear_event]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].decoded_data, DecodedEvent::ClearV3(_)));
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
        let result = decode_events(&[clear_event]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].decoded_data, DecodedEvent::ClearV3(_)));
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
        let result = decode_events(&[clear_event]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].decoded_data, DecodedEvent::ClearV3(_)));
    }

    fn mk_event(block: &str, log_index: &str, tx: &str) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: EventType::Unknown,
            block_number: block.to_string(),
            block_timestamp: "0x0".to_string(),
            transaction_hash: Bytes::from_str(tx).unwrap(),
            log_index: log_index.to_string(),
            decoded_data: DecodedEvent::Unknown(UnknownEventDecoded {
                raw_data: "0x".to_string(),
                note: "".to_string(),
            }),
        }
    }

    #[test]
    fn sort_events_by_block_and_log_orders_stably() {
        let mut events = vec![
            mk_event("0x2", "0x1", "0x10"),
            mk_event("0x1", "0x2", "0x20"),
            mk_event("0x1", "0x1", "0x30"),
        ];
        sort_decoded_events_by_block_and_log(&mut events).unwrap();
        assert_eq!(events[0].transaction_hash, Bytes::from_str("0x30").unwrap());
        assert_eq!(events[1].transaction_hash, Bytes::from_str("0x20").unwrap());
        assert_eq!(events[2].transaction_hash, Bytes::from_str("0x10").unwrap());
    }

    #[test]
    fn sort_events_preserves_relative_order_for_identical_keys() {
        let mut events = vec![
            mk_event("0x1", "0x1", "0x01"),
            mk_event("0x1", "0x1", "0x02"),
            mk_event("0x1", "0x1", "0x03"),
        ];
        let expected_order: Vec<_> = events
            .iter()
            .map(|event| event.transaction_hash.clone())
            .collect();
        sort_decoded_events_by_block_and_log(&mut events).unwrap();
        let actual_order: Vec<_> = events
            .iter()
            .map(|event| event.transaction_hash.clone())
            .collect();
        assert_eq!(actual_order, expected_order);
    }

    #[test]
    fn sort_events_returns_error_without_mutating_on_invalid_block() {
        let mut events = vec![mk_event("bad-block", "0x1", "0x01")];
        let expected_block = events[0].block_number.clone();
        let expected_log = events[0].log_index.clone();
        let expected_hash = events[0].transaction_hash.clone();
        let err = sort_decoded_events_by_block_and_log(&mut events).unwrap_err();
        match err {
            LocalDbError::InvalidBlockNumberString { value, .. } => {
                assert_eq!(value, "bad-block")
            }
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(events[0].block_number, expected_block);
        assert_eq!(events[0].log_index, expected_log);
        assert_eq!(events[0].transaction_hash, expected_hash);
    }

    #[test]
    fn sort_events_returns_error_without_mutating_on_invalid_log_index() {
        let mut events = vec![mk_event("0x1", "oops", "0x01")];
        let expected_block = events[0].block_number.clone();
        let expected_log = events[0].log_index.clone();
        let expected_hash = events[0].transaction_hash.clone();
        let err = sort_decoded_events_by_block_and_log(&mut events).unwrap_err();
        match err {
            LocalDbError::InvalidLogIndex { value, .. } => assert_eq!(value, "oops"),
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(events[0].block_number, expected_block);
        assert_eq!(events[0].log_index, expected_log);
        assert_eq!(events[0].transaction_hash, expected_hash);
    }

    #[test]
    fn parse_u64_hex_or_dec_variants() {
        assert_eq!(parse_u64_hex_or_dec("0x0").unwrap(), 0);
        assert_eq!(parse_u64_hex_or_dec("0x1a").unwrap(), 26);
        assert_eq!(parse_u64_hex_or_dec("26").unwrap(), 26);
        assert!(parse_u64_hex_or_dec("garbage").is_err());
        assert_eq!(parse_u64_hex_or_dec("  0x2A  ").unwrap(), 42);
        assert_eq!(parse_u64_hex_or_dec("0XFF").unwrap(), 255);
        assert_eq!(parse_u64_hex_or_dec("  42 ").unwrap(), 42);
        let max_hex = format!("0x{:x}", u64::MAX);
        assert_eq!(parse_u64_hex_or_dec(&max_hex).unwrap(), u64::MAX);
        let max_dec = u64::MAX.to_string();
        assert_eq!(parse_u64_hex_or_dec(&max_dec).unwrap(), u64::MAX);
    }
}
