use anyhow::{anyhow, Result};
use graphql_client::GraphQLQuery;
use graphql_client::Response;
use once_cell::sync::Lazy;
use reqwest::Url;
use rust_bigint::BigInt;
use serde_bytes::ByteBuf as Bytes;

use crate::orders::orders_query::{Order_filter, OrdersQueryOrders};

static BASE_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://api.thegraph.com/subgraphs/name/siddharth2207/rainorderbook").unwrap()
});

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/orders/orders.schema.json",
    query_path = "src/orders/orders.graphql",
    response_derives = "Debug, Serialize, Deserialize, Clone"
)]
pub struct OrdersQuery;

pub async fn query(where_filter: Option<Order_filter>) -> Result<Vec<OrdersQueryOrders>> {
    let variables = orders_query::Variables { where_filter };
    let request_body = OrdersQuery::build_query(variables);

    let client = reqwest::Client::new();
    let res = client
        .post((*BASE_URL).clone())
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<orders_query::ResponseData> = res.json().await?;
    match response_body {
        Response {
            data: Some(orders_query::ResponseData { orders }),
            ..
        } => Ok(orders),
        _ => Err(anyhow!("Failed to get orders")),
    }
}
