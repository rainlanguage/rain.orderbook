use ethers::types::Bytes;
use ethers::types::H160;
use ethers::types::U256;
use ethers::utils::format_units;
use graphql_client::GraphQLQuery;
use graphql_client::Response;
use rust_bigint::BigInt;


use std::str::FromStr;
 
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/subgraph/schema/orders.schema.json",
    query_path = "src/subgraph/queries/orders.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct OrdersQuery;

pub async fn get_orders(sg_uri : String) -> anyhow::Result<Vec<Vec<String>>> { 

    let variables = orders_query::Variables {};
    let request_body = OrdersQuery::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post(sg_uri.clone())
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<orders_query::ResponseData> = res.json().await?;  


    let orders: Vec<_> = response_body.data.unwrap().orders.iter().map(|order| {
        let mut order_row: Vec<String> = vec![] ; 
        order_row.push(order.id.clone()) ;  

        // println!("order : {:#?}", &order.transaction.id);

        let order_owner = &*order.owner.id ; 
        let order_owner = hex::encode(order_owner.to_vec()) ;
        let order_owner = H160::from_str(order_owner.as_str()).unwrap().to_string() ;
        order_row.push(order_owner) ; 

        let ip_io: Vec<String>  = order.valid_inputs.as_ref().unwrap().iter().map(|x| {
            let token_symbol = &x.token.symbol ; 
            let vault_balance = U256::from_dec_str(&x.token_vault.balance.to_string()).unwrap()  ; 
            let token_decimals = u32::from_str(&x.token.decimals.to_string()).unwrap();  
            let token_amount = format_units(vault_balance,token_decimals).unwrap().to_string() ;  

            let ret_str = format!(
                "{}{}{}{}" ,
                token_symbol.to_string(),
                " : ",
                token_amount,
                "\n"
            ); 
            
            ret_str
        }).collect() ;    

        let ip_io = ip_io.join("") ;

        let op_io: Vec<String>  = order.valid_outputs.as_ref().unwrap().iter().map(|x| {
            let token_symbol = &x.token.symbol ; 
            let vault_balance = U256::from_dec_str(&x.token_vault.balance.to_string()).unwrap()  ; 
            let token_decimals = u32::from_str(&x.token.decimals.to_string()).unwrap();  
            let token_amount = format_units(vault_balance,token_decimals).unwrap().to_string() ;  

            let ret_str =format!(
                "{}{}{}{}" ,
                token_symbol.to_string(),
                " : ",
                token_amount,
                "\n"
            ) ;
            ret_str
             
        }).collect() ;
        let op_io = op_io.join("") ;
        order_row.push(ip_io) ;
        order_row.push(op_io) ;

        order_row
        

    }).collect() ;  
   
    Ok(orders)
}


