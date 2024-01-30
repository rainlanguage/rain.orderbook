#[cfg(test)]
pub mod test {
    use alloy_primitives::{hex, keccak256, Address, FixedBytes, U256};
    use alloy_sol_types::{abi::token::WordToken, SolCall, SolError, SolEvent};
    use rain_orderbook_bindings::IOrderBookV3::*;

    #[test]
    fn test_add_order_function() {
        assert_call_signature::<addOrderCall>("addOrder(((address,uint8,uint256)[],(address,uint8,uint256)[],(address,bytes,uint256[]),bytes))");

        let call = addOrderCall {
            config: OrderConfigV2 {
                validInputs: vec![IO {
                    token: Address::repeat_byte(0x11),
                    decimals: 16,
                    vaultId: U256::from(8),
                }],
                validOutputs: vec![IO {
                    token: Address::repeat_byte(0x22),
                    decimals: 16,
                    vaultId: U256::from(9),
                }],
                evaluableConfig: EvaluableConfigV3 {
                    deployer: Address::repeat_byte(0x33),
                    bytecode: vec![0x1b; 32],
                    constants: vec![U256::from(1)],
                },
                meta: vec![255u8; 32],
            },
        };
        let call_data = call.abi_encode();

        let expected_call_data = hex! (
            "847a1bc9"
            "0000000000000000000000000000000000000000000000000000000000000020" // ?? offset to start of config (1*32)
            "0000000000000000000000000000000000000000000000000000000000000080" // ?? @todo - what is this offset? (4*32)
            "0000000000000000000000000000000000000000000000000000000000000100" // ?? @todo - what is this offset? (5*32)
            "0000000000000000000000000000000000000000000000000000000000000180" // ?? @todo - what is this offset? (9*32)
            "0000000000000000000000000000000000000000000000000000000000000260" // ?? @todo - what is this offset? (13*32)
            "0000000000000000000000000000000000000000000000000000000000000001" // length of validInputs (1)
            "0000000000000000000000001111111111111111111111111111111111111111" // validInputs[0].token
            "0000000000000000000000000000000000000000000000000000000000000010" // validInputs[0].decimals
            "0000000000000000000000000000000000000000000000000000000000000008" // validInputs[0].vaultId
            "0000000000000000000000000000000000000000000000000000000000000001" // length of validOutputs (1)
            "0000000000000000000000002222222222222222222222222222222222222222" // validOutputs[0].token
            "0000000000000000000000000000000000000000000000000000000000000010" // validOtuputs[0].decimals
            "0000000000000000000000000000000000000000000000000000000000000009" // validOutputs[0].vaultId
            "0000000000000000000000003333333333333333333333333333333333333333" // evaluableConfig.deployer
            "0000000000000000000000000000000000000000000000000000000000000060" // offset to start of evaluableConfig.bytecode (3*32)
            "00000000000000000000000000000000000000000000000000000000000000a0" // offset to start of evaluableConfig.constants (5*32)
            "0000000000000000000000000000000000000000000000000000000000000020" // length of evaluableConfig.bytecode (32)
            "1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b" // evaluableConfig.bytecode
            "0000000000000000000000000000000000000000000000000000000000000001" // length of evaluableConfig.constants
            "0000000000000000000000000000000000000000000000000000000000000001" // evaulableConfig.contstants[1]
            "0000000000000000000000000000000000000000000000000000000000000020" // length of meta (32)
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" // meta
        );

        assert_eq!(call_data, expected_call_data);
    }

    #[test]
    fn test_add_order_error_no_handle_io() {
        assert_error_signature::<OrderNoHandleIO>("OrderNoHandleIO()");
        assert_eq!(
            OrderNoHandleIO::abi_decode_raw(&[], true),
            Ok(OrderNoHandleIO {})
        );
    }

    #[test]
    fn test_add_order_error_no_inputs() {
        assert_error_signature::<OrderNoInputs>("OrderNoInputs()");
        assert_eq!(
            OrderNoInputs::abi_decode_raw(&[], true),
            Ok(OrderNoInputs {})
        );
    }

    #[test]
    fn test_add_order_error_no_outputs() {
        assert_error_signature::<OrderNoOutputs>("OrderNoOutputs()");
        assert_eq!(
            OrderNoOutputs::abi_decode_raw(&[], true),
            Ok(OrderNoOutputs {})
        );
    }

    #[test]
    fn test_add_order_error_no_sources() {
        assert_error_signature::<OrderNoSources>("OrderNoSources()");
        assert_eq!(
            OrderNoSources::abi_decode_raw(&[], true),
            Ok(OrderNoSources {})
        );
    }

    #[test]
    fn test_add_order_event() {
        assert_event_signature::<AddOrder>("AddOrder(address,address,(address,bool,(address,address,address),(address,uint8,uint256)[],(address,uint8,uint256)[]),bytes32)");
        let add_order_event = AddOrder {
            sender: Address::repeat_byte(0x11),
            expressionDeployer: Address::repeat_byte(0x22),
            order: OrderV2 {
                owner: Address::repeat_byte(0x33),
                handleIO: true,
                evaluable: EvaluableV2 {
                    interpreter: Address::repeat_byte(0x44),
                    store: Address::repeat_byte(0x55),
                    expression: Address::repeat_byte(0x66),
                },
                validInputs: vec![IO {
                    token: Address::repeat_byte(0x77),
                    decimals: 16,
                    vaultId: U256::from(8),
                }],
                validOutputs: vec![IO {
                    token: Address::repeat_byte(0x88),
                    decimals: 16,
                    vaultId: U256::from(9),
                }],
            },
            orderHash: FixedBytes::from([255; 32]),
        };
        assert_eq!(
            add_order_event.encode_topics_array::<1>(),
            [WordToken(AddOrder::SIGNATURE_HASH)]
        );

        assert_eq!(
            add_order_event.encode_data(),
            hex!(
                "0000000000000000000000001111111111111111111111111111111111111111" // sender
                "0000000000000000000000002222222222222222222222222222222222222222" // expressionDeployer
                "0000000000000000000000000000000000000000000000000000000000000080" // offset to start of order (3*32)
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" // orderHash
                "0000000000000000000000003333333333333333333333333333333333333333" // order.owner
                "0000000000000000000000000000000000000000000000000000000000000001" // order.handleIO
                "0000000000000000000000004444444444444444444444444444444444444444" // order.evaluable.intpreter
                "0000000000000000000000005555555555555555555555555555555555555555" // order.evaluable.store
                "0000000000000000000000006666666666666666666666666666666666666666" // order.evaluable.expression
                "00000000000000000000000000000000000000000000000000000000000000e0" // offset to start of validInputs (beginning at offset to start of order) (7*32)
                "0000000000000000000000000000000000000000000000000000000000000160" // offset to start of validOutputs (beginning at offset to start of order) (7*32)
                "0000000000000000000000000000000000000000000000000000000000000001" // length of validInputs
                "0000000000000000000000007777777777777777777777777777777777777777" // validInputs[0].token
                "0000000000000000000000000000000000000000000000000000000000000010" // validInputs[0].decimals
                "0000000000000000000000000000000000000000000000000000000000000008" // validInputs[0].vaultId
                "0000000000000000000000000000000000000000000000000000000000000001" // length of validOutputs
                "0000000000000000000000008888888888888888888888888888888888888888" // validOutputs[0].token
                "0000000000000000000000000000000000000000000000000000000000000010" // validOutputs[0].decimals
                "0000000000000000000000000000000000000000000000000000000000000009" // validOutputs[0].vaultId
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
