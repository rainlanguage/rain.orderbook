use alloy_primitives::Address;
use alloy_sol_types::SolCall;

use crate::{registry::IOrderBookV3::{IO, EvaluableConfigV3, OrderConfigV2, self}, interpreter::{get_disp, parse_rainstring}};


pub async fn add_ob_order(
    deployer_address: Address,
    input_vaults: Vec<IO>,
    output_vaults: Vec<IO>,
    rainlang_order_string: String,
    rpc_url: String
) -> anyhow::Result<Vec<u8>> {
    

   
    let (_,_,rain_parser) = get_disp(deployer_address.clone(),rpc_url.clone()).await.unwrap();
    let (bytecode,constants) = parse_rainstring(
        rain_parser,
        rainlang_order_string,
        rpc_url.clone()
    ).await.unwrap(); 

    let evaluable_config = EvaluableConfigV3{
        deployer: deployer_address,
        bytecode: bytecode.to_vec(),
        constants: constants
    };

    let order_config = OrderConfigV2{
        validInputs: input_vaults,
        validOutputs: output_vaults,
        evaluableConfig: evaluable_config,
        meta: vec![]
    };

    let add_order_data = IOrderBookV3::addOrderCall{
        config: order_config
    }.abi_encode();

    Ok(add_order_data)
}   