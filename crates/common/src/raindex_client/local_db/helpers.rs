use super::decode::{DecodedEvent, DecodedEventData};
use super::LocalDbError;
use alloy::{hex, primitives::Address};
use std::collections::HashMap;

pub fn ensure_deposit_decimals_available(
    events: &[DecodedEventData<DecodedEvent>],
    decimals_by_addr: &HashMap<Address, u8>,
) -> Result<(), LocalDbError> {
    for event in events {
        if let DecodedEvent::DepositV2(decoded) = &event.decoded_data {
            if !decimals_by_addr.contains_key(&decoded.token) {
                return Err(LocalDbError::CustomError(format!(
                    "Missing decimals for token {} required to compute deposit_amount",
                    hex::encode_prefixed(decoded.token)
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use rain_orderbook_bindings::IOrderBookV5::DepositV2;

    fn wrap_deposit(token: Address) -> DecodedEventData<DecodedEvent> {
        DecodedEventData {
            event_type: super::super::decode::EventType::DepositV2,
            block_number: String::new(),
            block_timestamp: String::new(),
            transaction_hash: String::new(),
            log_index: String::new(),
            decoded_data: DecodedEvent::DepositV2(Box::new(DepositV2 {
                sender: Address::from([0u8; 20]),
                token,
                vaultId: U256::ZERO.into(),
                depositAmountUint256: U256::from(1u64),
            })),
        }
    }

    #[test]
    fn succeeds_when_decimals_present() {
        let token = Address::from([1u8; 20]);
        let events = vec![wrap_deposit(token)];
        let mut map = HashMap::new();
        map.insert(token, 18u8);

        assert!(ensure_deposit_decimals_available(&events, &map).is_ok());
    }

    #[test]
    fn errors_when_decimals_missing() {
        let token = Address::from([2u8; 20]);
        let events = vec![wrap_deposit(token)];
        let map = HashMap::new();

        let err = ensure_deposit_decimals_available(&events, &map).unwrap_err();
        assert!(matches!(err, LocalDbError::CustomError(_)));
    }
}
