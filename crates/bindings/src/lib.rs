use alloy_sol_types::sol;

sol!(IOrderBookV3, "../../out/IOrderBookV3.sol/IOrderBookV3.json");

#[cfg(test)]
pub mod test {
    use crate::IOrderBookV3::depositCall;
    use alloy_sol_types::{SolCall, SolError};
    use alloy_primitives::{hex, Address, U256, keccak256};

    #[test]
    fn test_deposit() {
        assert_call_signature::<depositCall>("deposit(address,uint256,uint256)");

        let call = depositCall {
            token: Address::repeat_byte(0x11),
            amount: U256::from(1),
            vaultId: U256::from(1),
        };
        let call_data = call.abi_encode();

        let expected_call_data = hex! (
            "0efe6a8b"
            "0000000000000000000000001111111111111111111111111111111111111111"
            "0000000000000000000000000000000000000000000000000000000000000001"
            "0000000000000000000000000000000000000000000000000000000000000001"
        );

        assert_eq!(
            call_data,
            expected_call_data
        );
    }

    fn assert_call_signature<T: SolCall>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SELECTOR, keccak256(expected)[..4]);
    }
    
    fn assert_error_signature<T: SolError>(expected: &str) {
        assert_eq!(T::SIGNATURE, expected);
        assert_eq!(T::SELECTOR, keccak256(expected)[..4]);
    }
}