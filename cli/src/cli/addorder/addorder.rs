use ethers::types::{H160, U256,Bytes};
use ethers_signers::Ledger;
use spinners::{Spinner, Spinners};
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};
use ethers:: prelude::SignerMiddleware;
use ethers::providers::{Provider, Http} ;

use crate::cli::registry::{IOrderBookV2, IParserV1, Io, EvaluableConfig, OrderConfig};

#[allow(unused_variables)]
pub async fn add_ob_order(
    orderbook_address : H160,
    parser_address : H160,
    tokens : Vec<String>,
    decimals : Vec<u8>,
    order_string: String ,
    order_meta : String ,
    rpc_url : String,
    wallet : Ledger
) -> anyhow::Result<()> {

    let provider = Provider::<Http>::try_from(rpc_url)
    .expect("\n❌Could not instantiate HTTP Provider");   

    let client = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet).await?; 

    let orderbook = IOrderBookV2::new(orderbook_address, Arc::new(client)); 

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

    let mut sp = Spinner::new(
        Spinners::from_str("Dots9").unwrap(),
        "Adding order...".into(),
    ); 

    let order_tx = orderbook.add_order(order_config) ; 

    let order_pending_tx = order_tx.send().await?;

    let order_receipt = order_pending_tx.confirmations(3).await?.unwrap(); 

    let order_msg = format!(
        "{}{}{}" ,
        String::from("\nOrder added !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(order_receipt.transaction_hash.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    sp.stop_with_message(order_msg.into()); 


    Ok(())
}