use crate::{
    generated::{ERC20Mock, EvaluableConfigV2, Io, OrderConfigV2, RainterpreterExpressionDeployer},
    utils::mock_rain_doc,
};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Bytes, U256},
};

pub async fn generate_order_config(
    expression_deployer: &RainterpreterExpressionDeployer<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    >,
    token_input: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    token_output: &ERC20Mock<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
) -> OrderConfigV2 {
    let io_input = generate_io(token_input).await;
    let io_output = generate_io(token_output).await;

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
) -> Io {
    // Build the IO and return it
    Io {
        token: token.address(),
        decimals: token.decimals().await.unwrap(),
        vault_id: U256::from(0),
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
