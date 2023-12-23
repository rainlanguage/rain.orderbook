use alloy_primitives::{Address, Bytes, U256};
use anyhow::anyhow;
use crate::registry::{IExpressionDeployerV3, IParserV1};
use std::str::FromStr;
use ethers::{
    providers::{Http,Provider},
    types::H160
};
use std::sync::Arc;


pub async fn get_disp(
    deployer_npe2: Address,
    rpc_url: String
) -> anyhow::Result<(Address, Address, Address)>  {

    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            return Err(anyhow!(err));
        }
    }; 

    let deployer_npe2_address = H160::from_str(&deployer_npe2.to_string()).unwrap();
    let deployer_npe2 = IExpressionDeployerV3::new(deployer_npe2_address, Arc::new(provider.clone()));

    let interpreter: H160 =  deployer_npe2.i_interpreter().call().await.unwrap();
    let store: H160 =  deployer_npe2.i_store().call().await.unwrap(); 
    let parser: H160 =  deployer_npe2.i_parser().call().await.unwrap(); 

    let store = Address::new(store.to_fixed_bytes());
    let intepreter = Address::new(interpreter.to_fixed_bytes());
    let parser = Address::new(parser.to_fixed_bytes());

    Ok((store, intepreter, parser))

} 


pub async fn parse_rainstring(
    parser_address: Address,
    rainstring: String,
    rpc_url: String
) -> anyhow::Result<(Bytes,Vec<U256>)>  {

    let provider = match Provider::<Http>::try_from(rpc_url.clone()) {
        Ok(provider) => provider,
        Err(err) => {
            return Err(anyhow!(err));
        }
    }; 

    let parser_address = H160::from_str(&parser_address.to_string()).unwrap();
    let rain_parser = IParserV1::new(parser_address, Arc::new(provider.clone()));

    let (sources, constants) = rain_parser
        .parse(ethers::types::Bytes::from(
            rainstring.as_bytes().to_vec(),
        ))
        .call()
        .await
        .unwrap();

    let bytecode_npe2 = Bytes::from(sources.to_vec());

    let mut constants_npe2: Vec<U256> = vec![]; 
    
    for i in constants.into_iter() {
        constants_npe2.push(U256::from_str(i.to_string().as_str()).unwrap());
    } 

    Ok((bytecode_npe2,constants_npe2))

}