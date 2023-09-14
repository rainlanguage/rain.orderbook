use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units} ; 
use std::{convert::TryFrom, sync::Arc};
use tracing::error;
use anyhow::anyhow;
use crate::{cli::registry::{Order, IOrderBookV3}, gasoracle::{is_block_native_supported, gas_price_oracle}}; 


pub async fn remove_order(
    order_to_remove : Order ,  
    orderbook_address : H160 ,
    rpc_url : String ,
    blocknative_api_key : Option<String>
) -> anyhow::Result<Eip1559TransactionRequest> {

    let provider = match Provider::<Http>::try_from(rpc_url.clone()){
        Ok(provider) => {
            provider
        },
        Err(err) => {
            error!("INVALID RPC URL: {}",err) ; 
            return Err(anyhow!(err)) ;
        }
    } ;
    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let orderbook = IOrderBookV3::new(orderbook_address.clone(), Arc::new(provider.clone())); 

    let remove_order_tx = orderbook.remove_order(order_to_remove) ; 

    let remove_order_data: Bytes = remove_order_tx.calldata().unwrap() ;

    let mut remove_order_tx = Eip1559TransactionRequest::new();
    remove_order_tx.to = Some(orderbook_address.into());
    remove_order_tx.value = Some(U256::zero());
    remove_order_tx.data = Some(remove_order_data);
    remove_order_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        remove_order_tx.max_priority_fee_per_gas = Some(max_priority);
        remove_order_tx.max_fee_per_gas = Some(max_fee);
    }

    Ok(remove_order_tx)

}