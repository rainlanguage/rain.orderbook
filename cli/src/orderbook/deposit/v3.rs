use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units} ; 
use std::{convert::TryFrom, sync::Arc};

use crate::{cli::registry::IOrderBookV3, gasoracle::{is_block_native_supported, gas_price_oracle}};

pub async fn deposit_token( 
    deposit_token_address : H160 ,
    deposit_token_amount : U256 ,
    deposit_vault_id : U256,
    orderbook_address : H160 ,
    rpc_url : String ,
    blocknative_api_key : Option<String>
) -> anyhow::Result<Eip1559TransactionRequest> { 
    
    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");   

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64(); 

    let orderbook = IOrderBookV3::new(orderbook_address.clone(), Arc::new(provider.clone())); 

    let deposit_tx = orderbook.deposit(deposit_token_address,deposit_vault_id,deposit_token_amount) ; 
    let deposit_data: Bytes = deposit_tx.calldata().unwrap() ;

    let mut deposit_tx = Eip1559TransactionRequest::new();
    deposit_tx.to = Some(orderbook_address.into());
    deposit_tx.value = Some(U256::zero());
    deposit_tx.data = Some(deposit_data);
    deposit_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        deposit_tx.max_priority_fee_per_gas = Some(max_priority);
        deposit_tx.max_fee_per_gas = Some(max_fee);
    }
    
    Ok(deposit_tx)
}

#[cfg(test)] 
mod test {
    
}