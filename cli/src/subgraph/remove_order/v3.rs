use std::str::FromStr;

use graphql_client::GraphQLQuery;
use graphql_client::Response;
use rust_bigint::BigInt;
use ethers::types::{H160, U256,Bytes} ;

use crate::cli::registry::{Evaluable,Io,Order} ;
 

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/subgraph/schema/orders.schema.json",
    query_path = "src/subgraph/queries/removeorder.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct RemoveOrder;

/// Fetches the order from the subgraph and build the [Order] struct for the order that is to be removed.
/// Returns the [Order] struct representing the order to be removed, which can be passed as an argument
/// to the [remove_order](crate::orderbook::remove_order::v3::remove_order) function.
/// 
/// # Arguments
/// * - `rpc_url`: Subgraph api endpoint.
/// * - `order_id`: ID of the order to be removed.
pub async fn get_remove_order(
    rpc_url : String ,
    order_id : String
) -> anyhow::Result<Order>{  

    let variables = remove_order::Variables{
        id : Some(order_id)
    } ;
    let request_body = RemoveOrder::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((rpc_url).clone())
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<remove_order::ResponseData> = res.json().await?; 

    let order_data: remove_order::RemoveOrderOrders = response_body.data.unwrap().orders.pop().unwrap() ;   
 
    let evaluable_config = Evaluable{
        interpreter : H160::from_str(&order_data.interpreter.to_string()).unwrap(),
        store : H160::from_str(&order_data.interpreter_store.to_string()).unwrap(),
        expression : H160::from_str(&order_data.expression.to_string()).unwrap(),
    } ;

    let ip_io: Vec<_>  = order_data.valid_inputs.unwrap().iter().map(|x| {
        let token_address = H160::from_str(&x.token.id.to_string()).unwrap() ; 
        let token_decimals = u8::from_str(&x.token.decimals.to_string()).unwrap(); 

        let vault_id = x.vault.id.split('-').next().unwrap() ;
        let vault_id = U256::from_dec_str(vault_id).unwrap() ;

        Io{
            token : token_address ,
            decimals: token_decimals,
            vault_id : vault_id
        }
    }).collect() ;

    let op_io: Vec<_>  = order_data.valid_outputs.unwrap().iter().map(|x| {
        let token_address = H160::from_str(&x.token.id.to_string()).unwrap() ; 
        let token_decimals = u8::from_str(&x.token.decimals.to_string()).unwrap(); 

        let vault_id = x.vault.id.split('-').next().unwrap() ;
        let vault_id = U256::from_dec_str(vault_id).unwrap() ;

        Io{
            token : token_address ,
            decimals: token_decimals,
            vault_id : vault_id
        }
    }).collect() ; 

    let owner = H160::from_str(&order_data.owner.id.to_string()).unwrap() ;

    let remove_order = Order {
        owner : owner ,
        handle_io : order_data.handle_io ,
        evaluable : evaluable_config ,
        valid_inputs : ip_io ,
        valid_outputs : op_io
    } ;   

    Ok(remove_order)    

}