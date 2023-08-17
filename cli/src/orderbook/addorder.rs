use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units, prelude::SignerMiddleware} ; 

use ethers_signers::Ledger;
use spinners::{Spinner, Spinners};
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};


use crate::{cli::registry::{IOrderBookV2, IParserV1, Io, EvaluableConfig, OrderConfig}, gasoracle::{is_block_native_supported, gas_price_oracle}};

#[allow(unused_variables)]
pub async fn add_ob_order(
    orderbook_address : H160,
    parser_address : H160,
    tokens : Vec<String>,
    decimals : Vec<u8>,
    order_string: String ,
    order_meta : String ,
    rpc_url : String,
    wallet : Ledger,
    blocknative_api_key : Option<String>
) -> anyhow::Result<()> {

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider"); 

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64(); 

    let orderbook = IOrderBookV2::new(orderbook_address.clone(), Arc::new(provider.clone())); 

    let parser_contract = IParserV1::new(parser_address.clone(),Arc::new(provider.clone())) ;   

    let (sources,constants) = parser_contract.parse(
        Bytes::from(
            order_string.as_bytes().to_vec()
        )
    ).call().await.unwrap() ; 

    let tokens = tokens ;
    let decimals = decimals ;

    let vault_id = U256::from(H160::random().as_bytes()) ;   
    let mut decimals = decimals.iter() ;

    let io_arr: Vec<_> = tokens.iter().map(|x| {
        Io {
            token : H160::from_str(x).unwrap() ,
            decimals : *decimals.next().unwrap(),
            vault_id : vault_id.clone()

        }
    }).collect() ; 

    let eval_config = EvaluableConfig {
        deployer : parser_address,
        sources : sources ,
        constants : constants
    } ;

    let rain_magic_number = String::from("ff0a89c674ee7874") ; 
    
    // TODO cbor encode order_meta 
    let meta_string = hex::decode(
        // format!("{}{}",
        format!("{}",
        rain_magic_number,
        // hex::encode(order_meta)
        )
    ).unwrap();

    let meta_bytes = Bytes::from(meta_string) ; 

    let order_config = OrderConfig {
        valid_inputs : io_arr.clone() ,
        valid_outputs : io_arr.clone(),
        evaluable_config : eval_config ,
        meta : meta_bytes
    } ; 

    let order_tx = orderbook.add_order(order_config) ; 

    let order_tx_data: Bytes = order_tx.calldata().unwrap() ;

    let mut order_tx = Eip1559TransactionRequest::new();
    order_tx.to = Some(orderbook_address.into());
    order_tx.value = Some(U256::zero());
    order_tx.data = Some(order_tx_data);
    order_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 
    
    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        order_tx.max_priority_fee_per_gas = Some(max_priority);
        order_tx.max_fee_per_gas = Some(max_fee);
    }
    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await?;  

    println!("\n-----------------------------------\nAdding order to Orderbook\n");
    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Awaiting confirmation from wallet...".into(),
    );  
    let order_tx = client.send_transaction(order_tx, None).await;   
    sp.stop() ;   

    match order_tx {
        Ok(order_tx) => {
            let mut sp = Spinner::new(
                Spinners::from_str("Dots9").unwrap(),
                "Transaction submitted. Awaiting block confirmations...".into(),
            );
            let order_receipt = order_tx.confirmations(1).await?.unwrap();  
            let order_msg = format!(
                "{}{}{}" ,
                String::from("\nOrder added !!\n#################################\n✅ Hash : "),
                format!("0x{}",hex::encode(order_receipt.transaction_hash.as_bytes().to_vec())), 
                String::from("\n-----------------------------------\n")
            ) ; 
            sp.stop_with_message(order_msg.into()); 
        }
        Err(_) => {
            println!("\n❌ Transaction Rejected.");
        }
    }

    Ok(())
}