#[cfg(test)]
pub mod test {
    use alloy_primitives::{hex, keccak256, Address, U256};
    use alloy_sol_types::{abi::token::WordToken, SolCall, SolError, SolEvent};
    use rain_orderbook_bindings::IOrderBookV3::*;

    #[test]
    fn test_withdraw_function() {
        assert_call_signature::<withdrawCall>("withdraw(address,uint256,uint256)");

        let call = withdrawCall {
            token: Address::repeat_byte(0x11),
            vaultId: U256::from(1),
            targetAmount: U256::from(1),
        };
        let call_data = call.abi_encode();

        let expected_call_data = hex! (
            "b5c5f672"
            "0000000000000000000000001111111111111111111111111111111111111111" // token
            "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
            "0000000000000000000000000000000000000000000000000000000000000001" // targetAmount
        );

        assert_eq!(call_data, expected_call_data);
    }

    #[test]
    fn test_withdraw_error() {
        assert_error_signature::<ZeroWithdrawTargetAmount>(
            "ZeroWithdrawTargetAmount(address,address,uint256)",
        );
        let call_data = hex!(
            "0000000000000000000000001111111111111111111111111111111111111111" // sender
            "0000000000000000000000002222222222222222222222222222222222222222" // token
            "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
        );
        assert_eq!(
            ZeroWithdrawTargetAmount::abi_decode_raw(&call_data, true),
            Ok(ZeroWithdrawTargetAmount {
                sender: Address::repeat_byte(0x11),
                token: Address::repeat_byte(0x22),
                vaultId: U256::from(1),
            })
        );
    }

    #[test]
    fn test_withdraw_event() {
        assert_event_signature::<Withdraw>("Withdraw(address,address,uint256,uint256,uint256)");
        let withdraw_event = Withdraw {
            sender: Address::repeat_byte(0x11),
            token: Address::repeat_byte(0x22),
            vaultId: U256::from(1),
            targetAmount: U256::from(1),
            amount: U256::from(1),
        };
        assert_eq!(
            withdraw_event.encode_topics_array::<1>(),
            [WordToken(Withdraw::SIGNATURE_HASH)]
        );
        assert_eq!(
            withdraw_event.encode_data(),
            hex!(
                "0000000000000000000000001111111111111111111111111111111111111111" // sender
                "0000000000000000000000002222222222222222222222222222222222222222" // token
                "0000000000000000000000000000000000000000000000000000000000000001" // vaultId
                "0000000000000000000000000000000000000000000000000000000000000001" // targetAmount
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
