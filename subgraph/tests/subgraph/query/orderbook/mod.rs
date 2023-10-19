use self::order_book::ResponseData;
use crate::subgraph::wait;
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes};
use graphql_client::{GraphQLQuery, Response};
use serde::{Deserialize, Serialize};
use super::SG_URL;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/orderbook/orderbook.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]
pub struct OrderBook;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderBookResponse {
    pub id: Address,
    pub deployer: Address,
    pub address: Address,
    pub meta: Bytes,
}

impl OrderBookResponse {
    pub fn from(response: ResponseData) -> OrderBookResponse {
        let orderbook: order_book::OrderBookOrderBook = response.order_book.unwrap();

        let meta_bytes = orderbook
            .meta
            .unwrap_or(order_book::OrderBookOrderBookMeta {
                id: Bytes::from([0u8, 32]),
            })
            .id;

        OrderBookResponse {
            id: Address::from_slice(&orderbook.id),
            address: Address::from_slice(&orderbook.address),
            deployer: Address::from_slice(&orderbook.deployer.unwrap_or_default()),
            meta: meta_bytes,
        }
    }
}

pub async fn get_orderbook_query(orderbook_address: Address) -> Result<OrderBookResponse> {
    wait().await?;

    let variables = order_book::Variables {
        orderbook: format!("{:?}", orderbook_address).to_string().into(),
    };

    let request_body = OrderBook::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<order_book::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = OrderBookResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
