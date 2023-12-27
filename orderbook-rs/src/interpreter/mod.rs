use crate::{registry::{IExpressionDeployerV3, IParserV1}, errors::RainOrderbookError};
use alloy_primitives::{Address, Bytes, U256};
use ethers::{
    providers::{Http, Provider},
    types::H160,
};
use std::str::FromStr;
use std::sync::Arc;

/// Get RainInterpreterNPE2, RainterpreterStoreNPE2 and RainterpreterParserNPE2 addresses corresponding to a RainterpreterExpressionDeployerNPE2 contract.
///
/// # Arguments
/// * `deployer_npe2` - Address of RainterpreterExpressionDeployerNPE2.
/// * `rpc_url` - Network RPC URL.
///
pub async fn get_disp(
    deployer_npe2: Address,
    rpc_url: String,
) -> Result<(Address, Address, Address),RainOrderbookError> {
    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,  
        Err(err) => { 
            return Err(RainOrderbookError::InvalidRPC { source: err })
        }
    };

    let deployer_npe2_address = match H160::from_str(&deployer_npe2.to_string()){
        Ok(deployer) => deployer,
        Err(err) => {
            return Err(RainOrderbookError::InvalidAddress { source: err })
        }
    };
    let deployer_npe2 =
        IExpressionDeployerV3::new(deployer_npe2_address, Arc::new(provider.clone()));

    let interpreter = match deployer_npe2.i_interpreter().call().await {
        Ok(i_interpreter) => i_interpreter,
        Err(err) => { 
            return Err(RainOrderbookError::InvalidContractFunctionCall { source: err })
        }
    };
    let store = match deployer_npe2.i_store().call().await {
        Ok(i_store) => i_store,
        Err(err) => {
            return Err(RainOrderbookError::InvalidContractFunctionCall { source: err })
        }
    };
    let parser = match deployer_npe2.i_parser().call().await {
        Ok(i_parser) => i_parser,
        Err(err) => {
            return Err(RainOrderbookError::InvalidContractFunctionCall { source: err })
        }
    };

    let store = Address::new(store.to_fixed_bytes());
    let intepreter = Address::new(interpreter.to_fixed_bytes());
    let parser = Address::new(parser.to_fixed_bytes());

    Ok((store, intepreter, parser))
}

/// Parses rainlang expression string with RainterpreterParserNPE2 and returns the expression bytecode and constants
///
/// # Arguments
/// * `parser_address` - RainterpreterParserNPE2 address.
/// * `rainstring` - Rainlang Expression string.
/// * `rpc_url` - Network RPC URL.
pub async fn parse_rainstring(
    parser_address: Address,
    rainstring: String,
    rpc_url: String,
) -> Result<(Bytes, Vec<U256>),RainOrderbookError> {
    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            return Err(RainOrderbookError::InvalidRPC { source: err })
        }
    };

    let parser_address = match H160::from_str(&parser_address.to_string()) {
        Ok(parser) => parser,
        Err(err) => {
            return Err(RainOrderbookError::InvalidAddress { source: err })
        }
    };
    let rain_parser = IParserV1::new(parser_address, Arc::new(provider.clone()));

    let (sources, constants) = match rain_parser
        .parse(ethers::types::Bytes::from(rainstring.as_bytes().to_vec()))
        .call()
        .await
    {
        Ok(parse_result) => parse_result,
        Err(err) => {
            return Err(RainOrderbookError::InvalidContractFunctionCall { source: err })
        }
    };

    let bytecode_npe2 = Bytes::from(sources.to_vec());

    let mut constants_npe2: Vec<U256> = vec![];

    for i in constants.into_iter() {
        constants_npe2.push(U256::from_str(i.to_string().as_str()).unwrap());
    }

    Ok((bytecode_npe2, constants_npe2))
}
