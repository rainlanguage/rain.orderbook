use graphql_client::GraphQLQuery;
use graphql_client::Response;
use reqwest::Url;
use rust_bigint::BigInt;
use serde_bytes::ByteBuf as Bytes;

use once_cell::sync::Lazy; 
 
 
static BASE_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://api.thegraph.com/subgraphs/name/siddharth2207/nhstestbot").unwrap()
});

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/subgraph/schema/orders.schema.json",
    query_path = "src/subgraph/queries/orders.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct OrdersQuery;

pub async fn query() -> anyhow::Result<()> {
    let variables = orders_query::Variables {};
    let request_body = OrdersQuery::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*BASE_URL).clone())
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<orders_query::ResponseData> = res.json().await?; 

    let order_data = response_body.data.unwrap().orders.pop().unwrap() ; 



    println!("order_data : {:#?}" ,order_data ) ; 
    // match response_body {
    //     Response {
    //         data: Some(orders_query::ResponseData { orders }),
    //         ..
    //     } => {
    //         dbg!(&orders);
    //     }
    //     _ => {
    //         tracing::warn!("Failed to get orders");
    //     }
    // }
    Ok(())
}
