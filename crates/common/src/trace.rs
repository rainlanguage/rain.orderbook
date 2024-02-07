use crate::fork::fork_call;
use rain_interpreter_bindings::DeployerISP::{iParserCall, iStoreCall};
use rain_interpreter_bindings::IExpressionDeployerV3::deployExpression2Call;
use rain_interpreter_bindings::IInterpreterV2::eval2Call;
use rain_interpreter_bindings::IParserV1::parseCall;

pub async fn fork_eval_order(
    rainlang_string: &str,
    front_matter: &str,
    fork_url: &str,
    fork_block_number: u64,
) -> Result<Bytes, ForkEvalError> {
    let deployer = AddOrderArgs::try_parse_frontmatter(front_matter)?.0;

    let calldata = iParserCall {}.abi_encode();
    let parser_address = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        deployer.as_slice(),
        &calldata,
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

    let expression_address = deploy_return_decoded.expression;

    let eval_args = evalCall {
        store: store,
        namespace: vec![],
        dispatch,
        context,
        signedContext,
    };
    let calldata = eval_args.abi_encode();

    let result = fork_call(
        fork_url,
        fork_block_number,
        &decode(FROM_ADDRESS).unwrap(),
        &store,
        &calldata,
    );
}
