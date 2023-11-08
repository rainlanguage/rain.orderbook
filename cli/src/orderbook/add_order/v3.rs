use crate::{
    cli::registry::{EvaluableConfigV2, IOrderBookV3, IParserV1, Io, OrderConfigV2},
    gasoracle::{gas_price_oracle, is_block_native_supported},
};
use anyhow::anyhow;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Bytes, Eip1559TransactionRequest, H160, U256, U64},
    utils::parse_units,
};
use rain_cli_meta::meta::magic::KnownMagic;
use std::{convert::TryFrom, sync::Arc};
use tracing::error;

/// Builds and returns [Eip1559TransactionRequest] instance for `adding an order to the OrderBook`.
/// The integrity of the transaction data is ensured, provided that the input parameters are valid.
/// The transaction can then be submitted to the blockchain via any valid signer.
///
/// # Arguments
/// * `orderbook_address` - Address of the `OrderBook` contract.
/// * `parser_address` - Address of the `RainterpreterExpressionDeployer` contract implementing `IParserV1` interface.
/// * `tokens` - Array of token addresses, to be associated with the order.
/// * `decimals` - Array of token decimals corresponding to the `tokens` array.
/// * `vault_id` - vault_id of the vault to be associated with the order.
/// * `order_string` - String representing rainlang expression for the order.
/// * `order_meta` - String representing metadata for the order.
/// * `rpc_url` - Provider RPC.
/// * `blocknative_api_key` - Optional Blocknative API key.
///
/// # Example
/// ```
/// use std::str::FromStr;
/// use rain_cli_ob::orderbook::add_order::v3::add_ob_order;
/// use ethers::types::{U256, H160, Eip1559TransactionRequest};
///
/// async fn add_order() {
///     let rpc_url = "https://polygon.llamarpc.com/".to_string() ;
///     let orderbook_address = H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap() ;  
///     let parser_address = H160::from_str(&String::from("0x7b463524F7449593959FfeA70BE0301b42Ef7Be2")).unwrap() ;
///     let tokens = [
///         H160::from_str(&String::from("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")).unwrap(),
///         H160::from_str(&String::from("0xc2132D05D31c914a87C6611C10748AEb04B58e8F")).unwrap()
///     ] ;
///
///     let decimals:[u8;2] = [6,6] ;   
///     let order_string = String::from("max-amount ratio : 11e70 1001e15;:;");
///     let order_meta = String::from("");    
///
///     let vault_id = U256::from(H160::random().as_bytes()) ;   
///
///     let order_tx: Eip1559TransactionRequest = add_ob_order(
///        orderbook_address,
///        parser_address.clone(),
///        tokens.to_vec(),
///        decimals.to_vec(),
///        vault_id,
///        order_string,
///        order_meta,
///        rpc_url,
///        None
///     ).await.unwrap() ;  
///     
/// }
/// ```
#[allow(unused_variables)]
pub async fn add_ob_order(
    orderbook_address: H160,
    parser_address: H160,
    tokens: Vec<H160>,
    decimals: Vec<u8>,
    vault_id: U256,
    order_string: String,
    order_meta: String,
    rpc_url: String,
    blocknative_api_key: Option<String>,
) -> anyhow::Result<Eip1559TransactionRequest> {
    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            error!("INVALID RPC URL: {}", err);
            return Err(anyhow!(err));
        }
    };

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let orderbook = IOrderBookV3::new(orderbook_address.clone(), Arc::new(provider.clone()));

    let parser_contract = IParserV1::new(parser_address.clone(), Arc::new(provider.clone()));

    let (sources, constants) = parser_contract
        .parse(Bytes::from(order_string.as_bytes().to_vec()))
        .call()
        .await
        .unwrap();

    let tokens = tokens;
    let decimals = decimals;

    let mut decimals = decimals.iter();

    let io_arr: Vec<_> = tokens
        .iter()
        .map(|x| Io {
            token: *x,
            decimals: *decimals.next().unwrap(),
            vault_id: vault_id.clone(),
        })
        .collect();

    let eval_config = EvaluableConfigV2 {
        deployer: parser_address,
        bytecode: sources,
        constants: constants,
    };

    let rain_magic_number = KnownMagic::RainMetaDocumentV1.to_prefix_bytes().to_vec();

    let meta_bytes = Bytes::from(rain_magic_number);

    let order_config = OrderConfigV2 {
        valid_inputs: io_arr.clone(),
        valid_outputs: io_arr.clone(),
        evaluable_config: eval_config,
        meta: meta_bytes,
    };

    let order_tx = orderbook.add_order(order_config);

    let order_tx_data: Bytes = order_tx.calldata().unwrap();

    let mut order_tx = Eip1559TransactionRequest::new();
    order_tx.to = Some(orderbook_address.into());
    order_tx.value = Some(U256::zero());
    order_tx.data = Some(order_tx_data);
    order_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap());

    if is_block_native_supported(chain_id) {
        let (max_priority, max_fee) = gas_price_oracle(blocknative_api_key, chain_id)
            .await
            .unwrap();
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into();
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into();

        order_tx.max_priority_fee_per_gas = Some(max_priority);
        order_tx.max_fee_per_gas = Some(max_fee);
    }

    Ok(order_tx)
}

#[cfg(test)]
pub mod test {
    use crate::{
        cli::registry::{EvaluableConfigV2, Io},
        orderbook::add_order::v3::add_ob_order,
    };
    use ethers::providers::{Http, Middleware, Provider};
    use ethers::{
        abi::{ParamType, Token},
        types::{transaction::eip2718::TypedTransaction, Bytes, H160, U256},
    };
    use rain_cli_meta::meta::magic::KnownMagic;
    use std::str::FromStr;

    #[tokio::test]
    pub async fn test_add_order() -> anyhow::Result<()> {
        let rpc_url = "https://polygon.llamarpc.com/".to_string();
        let orderbook_address =
            H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap();
        let parser_address =
            H160::from_str(&String::from("0x7b463524F7449593959FfeA70BE0301b42Ef7Be2")).unwrap();

        let tokens = [
            H160::random(),
            H160::random(),
            H160::random(),
            H160::random(),
            H160::random(),
        ];

        let decimals: [u8; 5] = [6, 6, 12, 18, 9];
        let order_string = String::from("max-amount ratio : 11e70 1001e15;:;");
        let order_meta = String::from("");

        let vault_id = U256::from(H160::random().as_bytes());

        let order_tx = add_ob_order(
            orderbook_address,
            parser_address.clone(),
            tokens.to_vec(),
            decimals.to_vec(),
            vault_id,
            order_string,
            order_meta,
            rpc_url,
            None,
        )
        .await
        .unwrap();

        let tx_bytes = order_tx.data.unwrap().to_vec();
        let tx_bytes = &tx_bytes[4..];

        let io_tuple = ParamType::Tuple(
            [ParamType::Address, ParamType::Uint(8), ParamType::Uint(256)].to_vec(),
        );

        let evaluable_config_tuple = ParamType::Tuple(
            [
                ParamType::Address,
                ParamType::Bytes,
                ParamType::Array(Box::new(ParamType::Uint(256))),
            ]
            .to_vec(),
        );

        let order_tuple = ParamType::Tuple(
            [
                ParamType::Array(Box::new(io_tuple.clone())),
                ParamType::Array(Box::new(io_tuple.clone())),
                evaluable_config_tuple,
                ParamType::Bytes,
            ]
            .to_vec(),
        );

        let order_abi = [order_tuple];

        let decoded_data = ethers::abi::decode(&order_abi, tx_bytes).unwrap();

        let actual_order = match &decoded_data[0] {
            Token::Tuple(tuple) => tuple,
            _ => panic!("Unable To Decode Order"),
        };

        let input_vaults = match &actual_order[0] {
            Token::Array(input_vault) => input_vault,
            _ => panic!("Invalid input vaults"),
        };

        let ouput_vaults = match &actual_order[1] {
            Token::Array(output_vaults) => output_vaults,
            _ => panic!("Invalid input vaults"),
        };

        let evaulable = match &actual_order[2] {
            Token::Tuple(evaluable) => evaluable,
            _ => panic!("Invalid evaluable"),
        };

        let meta = match &actual_order[3] {
            Token::Bytes(meta) => meta,
            _ => panic!("Invalid meta"),
        };

        let rain_magic_number = KnownMagic::RainMetaDocumentV1.to_prefix_bytes().to_vec();
        let expected_meta = Bytes::from(rain_magic_number);
        let actual_meta = Bytes::from(meta.clone());
        assert_eq!(expected_meta, actual_meta);

        let actual_ip_vaults = desturcture_vault(input_vaults);
        let actual_op_vaults = desturcture_vault(ouput_vaults);
        let expected_vaults = construct_io(tokens.to_vec(), decimals.to_vec(), vault_id);
        check_io(actual_ip_vaults, expected_vaults.clone());
        check_io(actual_op_vaults, expected_vaults);

        let actual_evaluable = destructure_evaluable_config(evaulable);
        let expected_evaluable = construct_evaluable(
            parser_address,
            Bytes::from_str(&String::from(
                "0x020000000c02020002010000000100000100000000",
            ))
            .unwrap(),
            vec![
                U256::from_dec_str(
                    "110000000000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                U256::from_dec_str("1001000000000000000").unwrap(),
            ],
        );

        check_evaluable(expected_evaluable, actual_evaluable);

        Ok(())
    }

    #[tokio::test]
    pub async fn test_add_order_estimate() -> anyhow::Result<()> {
        let rpc_url = "https://polygon.llamarpc.com/".to_string();
        let orderbook_address =
            H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap();
        let parser_address =
            H160::from_str(&String::from("0x7b463524F7449593959FfeA70BE0301b42Ef7Be2")).unwrap();
        let from_address =
            H160::from_str(&String::from("0xF977814e90dA44bFA03b6295A0616a897441aceC")).unwrap();

        let tokens = [H160::random(), H160::random()];

        let decimals: [u8; 2] = [6, 6];
        let order_string = String::from("max-amount ratio : 11e70 1001e15;:;");
        let order_meta = String::from("");

        let vault_id = U256::from(H160::random().as_bytes());

        let mut order_tx = add_ob_order(
            orderbook_address,
            parser_address.clone(),
            tokens.to_vec(),
            decimals.to_vec(),
            vault_id,
            order_string,
            order_meta,
            rpc_url.clone(),
            None,
        )
        .await
        .unwrap();

        order_tx.from = Some(from_address.into());

        let provider = Provider::<Http>::try_from(rpc_url.clone()).unwrap();
        let deposit_tx = TypedTransaction::Eip1559(order_tx.clone());
        let estimate = provider.estimate_gas(&deposit_tx, None).await.unwrap();
        assert!(estimate > U256::zero());

        Ok(())
    }

    #[tokio::test]
    pub async fn test_rainlang_parse_order() -> anyhow::Result<()> {
        let rpc_url = "https://polygon.llamarpc.com/".to_string();
        let orderbook_address =
            H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap();
        let parser_address =
            H160::from_str(&String::from("0x7b463524F7449593959FfeA70BE0301b42Ef7Be2")).unwrap();

        let tokens = [H160::random(), H160::random()];

        let decimals: [u8; 2] = [6, 6];
        let order_string = String::from(
            "order-hash: context<1 0>(),       
            delay: int-add(block-timestamp() 3600),
            comparison: less-than(order-hash delay),
            max-amount ratio : 11e70 1001e15 ;
            :;",
        );
        let order_meta = String::from("");

        let vault_id = U256::from(H160::random().as_bytes());

        let order_tx = add_ob_order(
            orderbook_address,
            parser_address.clone(),
            tokens.to_vec(),
            decimals.to_vec(),
            vault_id,
            order_string,
            order_meta,
            rpc_url,
            None,
        )
        .await
        .unwrap();

        let tx_bytes = order_tx.data.unwrap().to_vec();
        let tx_bytes = &tx_bytes[4..];

        let io_tuple = ParamType::Tuple(
            [ParamType::Address, ParamType::Uint(8), ParamType::Uint(256)].to_vec(),
        );

        let evaluable_config_tuple = ParamType::Tuple(
            [
                ParamType::Address,
                ParamType::Bytes,
                ParamType::Array(Box::new(ParamType::Uint(256))),
            ]
            .to_vec(),
        );

        let order_tuple = ParamType::Tuple(
            [
                ParamType::Array(Box::new(io_tuple.clone())),
                ParamType::Array(Box::new(io_tuple.clone())),
                evaluable_config_tuple,
                ParamType::Bytes,
            ]
            .to_vec(),
        );

        let order_abi = [order_tuple];

        let decoded_data = ethers::abi::decode(&order_abi, tx_bytes).unwrap();

        let actual_order = match &decoded_data[0] {
            Token::Tuple(tuple) => tuple,
            _ => panic!("Unable To Decode Order"),
        };

        let evaulable = match &actual_order[2] {
            Token::Tuple(evaluable) => evaluable,
            _ => panic!("Invalid evaluable"),
        };

        let actual_evaluable = destructure_evaluable_config(evaulable);
        let expected_evaluable = construct_evaluable(
            parser_address,
            Bytes::from_str(&String::from("0x02000000280905000502000001010000000800000019020000000000010000000012020000010000010100000200000000")).unwrap(),
            vec![
                U256::from_dec_str("3600").unwrap(),
                U256::from_dec_str("110000000000000000000000000000000000000000000000000000000000000000000000").unwrap(),
                U256::from_dec_str("1001000000000000000").unwrap(),
            ]
        ) ;

        check_evaluable(expected_evaluable, actual_evaluable);

        Ok(())
    }

    pub fn construct_io(tokens: Vec<H160>, decimals: Vec<u8>, vault_id: U256) -> Vec<Io> {
        let mut io_arr: Vec<Io> = vec![];
        for (i, token) in tokens.iter().enumerate() {
            io_arr.push(Io {
                token: token.clone(),
                decimals: decimals[i].clone(),
                vault_id: vault_id.clone(),
            })
        }
        io_arr
    }

    pub fn destructure_io(token: &Token) -> Io {
        let tuple = match token {
            Token::Tuple(tuple) => tuple,
            _ => panic!("Invalid IO"),
        };

        let token_address = match tuple[0] {
            Token::Address(address) => address,
            _ => panic!("Invalid address"),
        };
        let token_decimal = match tuple[1] {
            Token::Uint(decimal) => decimal,
            _ => panic!("Invalid address"),
        };
        let vault_id = match tuple[2] {
            Token::Uint(vault_id) => vault_id,
            _ => panic!("Invalid address"),
        };

        Io {
            token: token_address,
            decimals: u8::from_str(token_decimal.to_string().as_str()).unwrap(),
            vault_id: vault_id,
        }
    }

    pub fn desturcture_vault(tokens: &Vec<Token>) -> Vec<Io> {
        let mut io_arr: Vec<Io> = vec![];
        for token in tokens {
            let io = destructure_io(token);
            io_arr.push(io);
        }
        io_arr
    }

    pub fn check_io(expected: Vec<Io>, actual: Vec<Io>) {
        for (i, io) in expected.iter().enumerate() {
            assert_eq!(io.token, actual[i].token);
            assert_eq!(io.decimals, actual[i].decimals);
            assert_eq!(io.vault_id, actual[i].vault_id);
        }
    }

    pub fn construct_evaluable(
        deployer: H160,
        bytecode: Bytes,
        constants: Vec<U256>,
    ) -> EvaluableConfigV2 {
        EvaluableConfigV2 {
            deployer: deployer,
            bytecode: bytecode,
            constants: constants,
        }
    }

    pub fn destructure_evaluable_config(token: &Vec<Token>) -> EvaluableConfigV2 {
        let expression_deployer = match &token[0] {
            Token::Address(address) => address,
            _ => panic!("Invalid address"),
        };
        let bytcode = match &token[1] {
            Token::Bytes(bytes) => bytes,
            _ => panic!("Invalid bytecode"),
        };

        let constants = match &token[2] {
            Token::Array(constants) => constants,
            _ => panic!("Invalid constants"),
        };

        let mut actual_constants: Vec<U256> = vec![];
        for constant in constants {
            let constant = match constant {
                Token::Uint(constant) => constant,
                _ => panic!("Invalid constant"),
            };
            actual_constants.push(constant.clone());
        }

        EvaluableConfigV2 {
            deployer: expression_deployer.clone(),
            bytecode: Bytes::from(bytcode.clone()),
            constants: actual_constants.clone(),
        }
    }

    pub fn check_evaluable(expected: EvaluableConfigV2, actual: EvaluableConfigV2) {
        assert_eq!(expected.deployer, actual.deployer);
        assert_eq!(expected.bytecode, actual.bytecode);
        for (i, constant) in expected.constants.iter().enumerate() {
            assert_eq!(constant.clone(), actual.constants[i]);
        }
    }
}
