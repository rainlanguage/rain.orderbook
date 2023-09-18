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

#[cfg(test)] 
mod test { 
    use std::{str::FromStr, sync::Arc};
    use ethers::{types::{U256, H160, Bytes}, abi::{ParamType, Token}};
    use crate::orderbook::{remove_order::v3::remove_order, add_order::v3::test::check_io};
    use crate::cli::registry::{Io, Order, Evaluable}; 
    use crate::orderbook::add_order::v3::test::desturcture_vault ;

    #[tokio::test]
    pub async fn test_remove_order() -> anyhow::Result<()>  { 

        let rpc_url = "https://polygon.llamarpc.com".to_string() ;
        let orderbook_address = H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap() ; 

        let interpreter = H160::random(); 
        let store = H160::random(); 
        let expression = H160::random(); 
        let owner = H160::random() ; 

        let vault_id = U256::from(H160::random().as_bytes()) ;
        
        let tokens = [
            String::from("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"),
            String::from("0xc2132D05D31c914a87C6611C10748AEb04B58e8F")
        ] ; 

        let decimals:[u8;2] = [6,6] ;  
        
        let tokens = tokens ;
        let decimals = decimals ;
        
        let mut decimals = decimals.iter() ;
        
        let io_arr: Vec<_> = tokens.iter().map(|x| {
            Io {
                token : H160::from_str(x).unwrap() ,
                decimals : *decimals.next().unwrap(),
                vault_id : vault_id.clone()
        
            }
        }).collect() ;  

        let evaluable =  Evaluable{
            interpreter ,
            store ,
            expression ,
        } ;

        let handle_io = false ;
        
        let order_config = Order {
           owner : owner.clone() ,
           handle_io : handle_io.clone(),
           evaluable : evaluable.clone(),
            valid_inputs : io_arr.clone(),
            valid_outputs : io_arr.clone()
        } ;  
        
        let remove_order_tx = remove_order(
            order_config,
            orderbook_address,
            rpc_url,
            None
        ).await.unwrap() ; 

        let tx_bytes = remove_order_tx.data.unwrap().to_vec() ;
        let tx_bytes = &tx_bytes[4..];  

        let evaluable_tuple = ParamType::Tuple([
            ParamType::Address,
            ParamType::Address,
            ParamType::Address,
        ].to_vec()) ;   

        let io_tuple = ParamType::Tuple([
            ParamType::Address,
            ParamType::Uint(8),
            ParamType::Uint(256),
        ].to_vec()) ;

        let order_tuple = ParamType::Tuple([
            ParamType::Address,
            ParamType::Bool,
            evaluable_tuple,
            ParamType::Array(Box::new(io_tuple.clone())),
            ParamType::Array(Box::new(io_tuple.clone())),
        ].to_vec()) ;

        let remove_order_abi = [order_tuple] ;
        
        let decoded_data = ethers::abi::decode(&remove_order_abi, tx_bytes).unwrap() ;  

        println!("decoded_data : {:#?}",decoded_data) ; 

        let actual_order = match &decoded_data[0] {
            Token::Tuple(tuple) => tuple,
            _ => panic!("Unable To Decode Order") 
        } ;  

        let input_vaults = match &actual_order[3]{
            Token::Array(input_vault) => input_vault,
            _ => panic!("Invalid input vaults") 
        };
        
        let ouput_vaults = match &actual_order[4]{
            Token::Array(output_vaults) => output_vaults ,
            _ => panic!("Invalid input vaults")
        } ;   
         
        let actual_ip_vaults = desturcture_vault(input_vaults) ;
        let actual_op_vaults = desturcture_vault(ouput_vaults) ;  
        check_io(actual_ip_vaults,io_arr.clone()) ;
        check_io(actual_op_vaults,io_arr.clone()) ; 

        let actual_evaluable = match &actual_order[2] {
            Token::Tuple(evaluable) => evaluable,
            _ => panic!("Unable To Decode Evaluable") 
        } ; 
        let actual_evaluable = destructure_evaluable(actual_evaluable) ;  

        assert_eq!(actual_evaluable.interpreter, evaluable.interpreter) ;
        assert_eq!(actual_evaluable.store, evaluable.store) ;
        assert_eq!(actual_evaluable.expression, evaluable.expression) ; 

        let actual_owner = match &actual_order[0] {
            Token::Address(owner) => owner,
            _ => panic!("Unable To Decode Owner") 
        } ; 

        let actual_handle_io = match &actual_order[1] {
            Token::Bool(handle_io) => handle_io,
            _ => panic!("Unable To Decode Handle IO") 
        } ;  

        assert_eq!(actual_owner.clone(), owner) ; 
        assert_eq!(actual_handle_io.clone(), handle_io) ; 

        Ok(())
        
    }  

    pub fn destructure_evaluable(token : &Vec<Token>) -> Evaluable {
        let interpreter = match &token[0] {
            Token::Address(address) => address ,
            _ => panic!("Invalid address")
        } ;
        let store = match &token[1] {
            Token::Address(address) => address ,
            _ => panic!("Invalid address")
        } ;
        let expression = match &token[2] {
            Token::Address(address) => address ,
            _ => panic!("Invalid address")
        } ;

        Evaluable{
            interpreter : interpreter.clone() ,
            store : store.clone() ,
            expression : expression.clone() ,
        } 

    }

}

