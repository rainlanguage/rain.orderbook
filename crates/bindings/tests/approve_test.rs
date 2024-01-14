#[cfg(test)]
pub mod test {
    use alloy_primitives::{hex, keccak256, Address, FixedBytes, U256};
    use alloy_sol_types::{abi::token::WordToken, SolCall, SolEvent};
    use rain_orderbook_bindings::IERC20::*;

    #[test]
    fn test_approve_function() {
        assert_call_signature::<approveCall>("approve(address,uint256)");

        let call = approveCall {
            spender: Address::repeat_byte(0x11),
            amount: U256::from(1),
        };
        let call_data = call.abi_encode();

        let expected_call_data = hex! (
            "095ea7b3"
            "0000000000000000000000001111111111111111111111111111111111111111" // spender
            "0000000000000000000000000000000000000000000000000000000000000001" // amount
        );

        assert_eq!(call_data, expected_call_data);
    }

    #[test]
    fn test_approval_event() {
        assert_event_signature::<Approval>("Approval(address,address,uint256)");
        let deposit_event = Approval {
            owner: Address::repeat_byte(0x11),
            spender: Address::repeat_byte(0x22),
            value: U256::from(1),
        };

        let expected_owner =
            hex!("0000000000000000000000001111111111111111111111111111111111111111"); // owner
        let expected_spender =
            hex!("0000000000000000000000002222222222222222222222222222222222222222"); // spender
        assert_eq!(
            deposit_event.encode_topics_array::<3>(),
            [
                WordToken(Approval::SIGNATURE_HASH),
                WordToken(FixedBytes::from(expected_owner)),
                WordToken(FixedBytes::from(expected_spender))
            ]
        );
        assert_eq!(
            deposit_event.encode_data(),
            hex!(
                "0000000000000000000000000000000000000000000000000000000000000001" // value
            )
        )
    }

    fn assert_call_signature<T: SolCall>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SELECTOR, keccak256(expected)[..4]);
    }

    fn assert_event_signature<T: SolEvent>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SIGNATURE_HASH, keccak256(expected));
    }
}
