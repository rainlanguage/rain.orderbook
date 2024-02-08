use std::ops::Add;

use crate::error::ForkEvalError;
use crate::fork::{fork_call, FROM_ADDRESS};
use alloy_primitives::hex::decode;
use alloy_primitives::{Address, BlockNumber, Bytes, U256};
use alloy_sol_types::{SolCall, SolType};
use rain_interpreter_bindings::DeployerISP::{iInterpreterCall, iParserCall, iStoreCall};
use rain_interpreter_bindings::IExpressionDeployerV3::deployExpression2Call;
use rain_interpreter_bindings::IInterpreterV2::eval2Call;
use rain_interpreter_bindings::IParserV1::parseCall;
use rain_interpreter_eval::{CreateEncodedDispatch, CreateNamespace};

#[derive(Default, Debug, Clone)]
struct OrderConfig {
    orderbook_address: Address,
    sender: Address,
    order_hash: B256,
    order_owner: Address,
    counterparty: Address,
    input_tokens: Vec<Address>,
    output_tokens: Vec<Address>,
}

pub async fn fork_eval_order(
    rainlang_string: &str,
    source_index: u16,
    order_config: &OrderConfig,
    deployer: Address,
    fork_url: &str,
    fork_block_number: u64,
) -> Result<(), ForkEvalError> {
    let parser_address = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        deployer.as_slice(),
        &iParserCall {}.abi_encode(),
    )
    .await??
    .result;

    let store = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        deployer.as_slice(),
        &iParserCall {}.abi_encode(),
    )
    .await??
    .result;

    let interpreter = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        deployer.as_slice(),
        &iInterpreterCall {}.abi_encode(),
    )
    .await??
    .result;

    let calldata = parseCall {
        data: rainlang_string.as_bytes().to_vec(),
    }
    .abi_encode();
    let expression_config = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        &parser_address,
        &calldata,
    )
    .await??
    .result;

    let mut calldata = deployExpression2Call::SELECTOR.to_vec();
    calldata.extend_from_slice(&expression_config);
    let deploy_return = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        deployer.as_slice(),
        &calldata,
    )
    .await??;

    let deploy_return_decoded =
        deployExpression2Call::abi_decode_returns(&deploy_return.result, true).unwrap();

    let dispatch = CreateEncodedDispatch::encode(deploy_return_decoded.expression, source_index);
    let qualified_namespace = CreateNamespace::qualify_namespace(
        order_config.order_owner,
        order_config.orderbook_address,
    );

    let eval_args = eval2Call {
        store: Address::from_slice(&store),
        namespace: qualified_namespace,
        dispatch,
        context: build_eval_context(order_config),
        inputs: vec![],
    };
    let calldata = eval_args.abi_encode();

    let result = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        &interpreter,
        &calldata,
    );
    Ok(())
}

fn build_eval_context(config: &OrderConfig) -> Vec<Vec<U256>> {
    let mut context: Vec<InnerVec, Global> = Vec::new();

    // Base context
    let base_context = vec![Default::default(); 2];
    context.push(base_context);

    // Calling context
    let mut calling_context = Vec::new();
    calling_context.push(U256::from(config.order_hash));
    calling_context.push(U256::from(config.order_owner.as_slice()));
    calling_context.push(U256::from(config.counterparty.as_slice()));
    context.push(calling_context);

    // Calculations context
    let calculations_context = Vec::new();
    context.push(calculations_context);

    // Inputs context
    let inputs_context = config
        .input_tokens
        .iter()
        .map(|&token| U256::from(token.as_slice()))
        .collect();
    context.push(inputs_context);

    // Outputs context
    let outputs_context = config
        .output_tokens
        .iter()
        .map(|&token| U256::from(token.as_slice()))
        .collect();
    context.push(outputs_context);

    context
}
