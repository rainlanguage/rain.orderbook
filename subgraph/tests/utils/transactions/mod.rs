use crate::{
    generated::{
        AddOrderCall, DepositCall, ERC20Mock, EvaluableConfigV2, Io, OrderConfigV2,
        RainterpreterExpressionDeployer,
    },
    utils::{generate_random_u256, mock_rain_doc},
};
use ethers::{
    contract::EthCall,
    core::{abi::AbiEncode, k256::ecdsa::SigningKey},
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, Bytes, U256},
};

pub async fn mint_tokens(
    amount: &U256,
    target: &Address,
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> anyhow::Result<()> {
    token.mint(*target, *amount).send().await?.await?;

    Ok(())
}

pub async fn approve_tokens(
    amount: &U256,
    spender: &Address,
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> anyhow::Result<()> {
    token.approve(*spender, *amount).send().await?.await?;

    Ok(())
}

pub async fn generate_order_config(
    expression_deployer: &RainterpreterExpressionDeployer<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    >,
    token_input: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id_input: Option<U256>,
    token_output: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id_output: Option<U256>,
) -> OrderConfigV2 {
    let io_input = generate_io(token_input, vault_id_input).await;
    let io_output = generate_io(token_output, vault_id_output).await;

    let eval_config = generate_eval_config(expression_deployer).await;

    // Build the OrderConfig and return it
    OrderConfigV2 {
        valid_inputs: vec![io_input],
        valid_outputs: vec![io_output],
        evaluable_config: eval_config,
        meta: mock_rain_doc(),
    }
}

async fn generate_io(
    token: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    vault_id: Option<U256>,
) -> Io {
    // Build the IO and return it
    Io {
        token: token.address(),
        decimals: token.decimals().await.unwrap(),
        vault_id: vault_id.unwrap_or(generate_random_u256()),
    }
}

async fn generate_eval_config(
    expression_deployer: &RainterpreterExpressionDeployer<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    >,
) -> EvaluableConfigV2 {
    let data_parse = Bytes::from_static(b"_ _ _:block-timestamp() chain-id() block-number();:;");
    let (bytecode, constants) = expression_deployer
        .parse(data_parse.clone())
        .await
        .expect("cannot get value from parse");

    // Build the EvaluableConfig and return it
    EvaluableConfigV2 {
        deployer: expression_deployer.address(),
        bytecode,
        constants,
    }
}

/// From given orders, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_add_order(orders: Vec<&OrderConfigV2>) -> Vec<Bytes> {
    let selector: [u8; 4] = AddOrderCall::selector();

    let tuple_bytes: [u8; 32] = byte_for_tuples();

    let mut data: Vec<Bytes> = Vec::new();

    for order in orders {
        // The OrderConfigV2 from abigen implemented the `AbiEncode` trait, so
        // it could be easily encoded
        let encoded_order: Vec<u8> = AbiEncode::encode(order.to_owned());

        // Create a new Vec<u8> that will contain the function selector and the
        // current encoded_order
        let mut order_bytes: Vec<u8> = Vec::new();

        // Add selector to the new Vec
        order_bytes.extend_from_slice(&selector);
        order_bytes.extend_from_slice(&tuple_bytes);

        // Add encoded_order to the new Vec
        order_bytes.extend(encoded_order);

        let order_data = Bytes::from(order_bytes);

        // Push the order bytes
        data.push(order_data);
    }

    return data;
}

/// From given arguments, encode them to a collection of Bytes to be used with multicall
pub fn generate_multi_deposit(
    tokens: Vec<&Address>,
    vault_ids: Vec<&U256>,
    amounts: Vec<&U256>,
) -> Vec<Bytes> {
    if tokens.len() != vault_ids.len() || tokens.len() != amounts.len() {
        panic!("Mismatch length between provide data");
    }

    let selector: [u8; 4] = DepositCall::selector();
    println!("Selector: {}", Bytes::from(selector));

    let tuple_bytes: [u8; 32] = byte_for_tuples();

    let mut data: Vec<Bytes> = Vec::new();

    for token in tokens {
        let vault_id = vault_ids.get(0).unwrap().to_owned().to_owned();
        let amount = amounts.get(0).unwrap().to_owned().to_owned();

        let call_config = DepositCall {
            token: token.to_owned(),
            vault_id,
            amount,
        };

        let encoded_call = AbiEncode::encode(call_config);
        println!("encoded_call: {}", Bytes::from(encoded_call));
    }

    return data;
}

/// The extra 32 bytes for the start of the tuples.
///
/// `*TODO*`: Search why the encode function not give this
fn byte_for_tuples() -> [u8; 32] {
    let mut result = [0u8; 32]; // Initialize an array with all elements set to 0
    result[31] = 32; // Set the last element to 32
    result
}
