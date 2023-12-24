use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use anyhow::anyhow;
use crate::{
    interpreter::{get_disp, parse_rainstring},
    registry::IOrderBookV3::{self, EvaluableConfigV3, OrderConfigV2, IO},
};
use tracing::error;

pub async fn add_ob_order(
    deployer_address: Address,
    input_vaults: Vec<IO>,
    output_vaults: Vec<IO>,
    rainlang_order_string: String,
    rpc_url: String,
) -> anyhow::Result<Vec<u8>> {
    let (_, _, rain_parser) = match get_disp(deployer_address.clone(), rpc_url.clone())
        .await
        {
            Ok(parse_result) => parse_result,
            Err(err) => {
                error!("DISP");
                return Err(anyhow!(err));
            }
        };
    let (bytecode, constants) =
        match parse_rainstring(rain_parser, rainlang_order_string, rpc_url.clone())
            .await
            {
                Ok(parse_result) => parse_result,
                Err(err) => {
                    error!("FAILED TO PARSE");
                    return Err(anyhow!(err));
                }
            };

    let evaluable_config = EvaluableConfigV3 {
        deployer: deployer_address,
        bytecode: bytecode.to_vec(),
        constants: constants,
    };

    let order_config = OrderConfigV2 {
        validInputs: input_vaults,
        validOutputs: output_vaults,
        evaluableConfig: evaluable_config,
        meta: vec![],
    };

    let add_order_data = IOrderBookV3::addOrderCall {
        config: order_config,
    }
    .abi_encode();

    Ok(add_order_data)
}
