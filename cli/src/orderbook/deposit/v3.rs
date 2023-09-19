use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units} ; 
use std::{convert::TryFrom, sync::Arc};
use tracing::error;
use anyhow::anyhow;
use crate::{cli::registry::IOrderBookV3, gasoracle::{is_block_native_supported, gas_price_oracle}};

/// Builds and returns [Eip1559TransactionRequest] instance for `depositing tokens into OrderBook`.
/// The integrity of the transaction data is ensured, provided that the input parameters are valid.
/// The transaction can then be submitted to the blockchain via any valid signer. 
/// 
/// # Arguments
/// * `deposit_token_address` - Address of the ERC20 token to be deposited.
/// * `deposit_token_amount` - Fully denominated amount of tokens to be deposited.
/// * `deposit_vault_id` - Vault ID of the vault to be deposited into.
/// * `orderbook_address` - Address of the OrderBook contract. 
/// * `rpc_url` - Provider RPC URL. 
/// * `blocknative_api_key` - Optional Blocknative API key. 
/// 
/// # Example
/// 
/// ```
/// use std::str::FromStr;
/// use rain_cli_ob::orderbook::deposit::v3::deposit_tokens; 
/// use ethers::types::{U256, H160, Eip1559TransactionRequest};
/// 
/// async fn deposit() {
///     let rpc_url = "https://polygon.llamarpc.com/".to_string() ;
///     let orderbook_address = H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap() ;  
///     let deposit_token_address = H160::from_str(&String::from("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174")).unwrap() ; 
///     let deposit_token_amount = U256::from_dec_str("1000000").unwrap();
///     let deposit_vault_id = U256::from(H160::random().as_bytes()) ; 
///
///     let deposit_tx = deposit_tokens(
///         deposit_token_address,
///         deposit_token_amount,
///         deposit_vault_id,
///         orderbook_address,
///         rpc_url,
///         None
///     ).await.unwrap() ;  
///     
/// }
/// ```
pub async fn deposit_tokens( 
    deposit_token_address : H160 ,
    deposit_token_amount : U256 ,
    deposit_vault_id : U256,
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
    use std::str::FromStr;
    use ethers::{types::{U256, H160}, abi::{ParamType, Token}};
    use crate::orderbook::deposit::v3::deposit_tokens; 

    #[tokio::test]
    pub async fn test_deposit() { 

        let rpc_url = "https://polygon.llamarpc.com".to_string() ;
        let orderbook_address = H160::from_str(&String::from("0xFb8a0C401C9d11fDecCdDDCBf89bFFA84681281d")).unwrap() ; 
        let deposit_token_address = H160::random() ; 
        let deposit_token_amount = U256::from(H160::random().as_bytes());
        let deposit_vault_id = U256::from(H160::random().as_bytes()) ; 

        let deposit_tx = deposit_tokens(
            deposit_token_address,
            deposit_token_amount,
            deposit_vault_id,
            orderbook_address,
            rpc_url,
            None
        ).await.unwrap() ;  

        let tx_bytes = deposit_tx.data.unwrap().to_vec() ; 
        let tx_bytes = &tx_bytes[4..];  
        
        let dep_param = [
            ParamType::Address,
            ParamType::Uint(256),
            ParamType::Uint(256),
        ] ;  

        let decoded_data = ethers::abi::decode(&dep_param, tx_bytes).unwrap() ;   
        
        match decoded_data[0] {
            Token::Address(address) => assert_eq!(address,deposit_token_address),
            _ => {}
        }

        match decoded_data[1] {
            Token::Uint(vault_id) => assert_eq!(vault_id,deposit_vault_id),
            _ => {}
        }
        
        match decoded_data[0] {
            Token::Uint(amount) => assert_eq!(amount,deposit_token_amount),
            _ => {}
        }
        
    } 

}