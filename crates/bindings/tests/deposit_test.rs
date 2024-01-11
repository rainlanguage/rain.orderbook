#[cfg(test)]
pub mod test {
    use rain_orderbook_bindings::IOrderBookV3::*;
    use alloy_primitives::{hex, keccak256, Address, U256};
    use alloy_sol_types::{abi::token::WordToken, SolCall, SolError, SolEvent};

    #[test]
    fn test_deposit_function() {
        assert_call_signature::<depositCall>("deposit(address,uint256,uint256)");

        let call = depositCall {
            token: Address::repeat_byte(0x11),
            amount: U256::from(1),
            vaultId: U256::from(1),
        };
        let call_data = call.abi_encode();

        let expected_call_data = hex! (
            "0efe6a8b"
            "0000000000000000000000001111111111111111111111111111111111111111" // token
            "0000000000000000000000000000000000000000000000000000000000000001" // amount
            "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
        );

        assert_eq!(call_data, expected_call_data);
    }

    #[test]
    fn test_deposit_error() {
        assert_error_signature::<ZeroDepositAmount>("ZeroDepositAmount(address,address,uint256)");
        let call_data = hex!(
            "0000000000000000000000001111111111111111111111111111111111111111" // sender
            "0000000000000000000000002222222222222222222222222222222222222222" // token
            "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
        );
        assert_eq!(
            ZeroDepositAmount::abi_decode_raw(&call_data, true),
            Ok(ZeroDepositAmount {
                sender: Address::repeat_byte(0x11),
                token: Address::repeat_byte(0x22),
                vaultId: U256::from(1),
            })
        );
    }

    #[test]
    fn test_deposit_event() {
        assert_event_signature::<Deposit>("Deposit(address,address,uint256,uint256)");
        assert!(!Deposit::ANONYMOUS);
        let deposit_event = Deposit {
            sender: Address::repeat_byte(0x11),
            token: Address::repeat_byte(0x22),
            vaultId: U256::from(1),
            amount: U256::from(1),
        };
        assert_eq!(
            deposit_event.encode_topics_array::<1>(),
            [WordToken(Deposit::SIGNATURE_HASH)]
        );
        assert_eq!(
            deposit_event.encode_data(),
            hex!(
                "0000000000000000000000001111111111111111111111111111111111111111" // sender
                "0000000000000000000000002222222222222222222222222222222222222222" // token
                "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
                "0000000000000000000000000000000000000000000000000000000000000001" // amount
            )
        )
    }

    fn assert_call_signature<T: SolCall>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SELECTOR, keccak256(expected)[..4]);
    }

    fn assert_error_signature<T: SolError>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SELECTOR, keccak256(expected)[..4]);
    }

    fn assert_event_signature<T: SolEvent>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SIGNATURE_HASH, keccak256(expected));
    }
}
