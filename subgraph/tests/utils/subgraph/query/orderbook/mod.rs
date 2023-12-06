use self::order_book::{OrderBookOrderBook, OrderBookOrderBookMeta, ResponseData, Variables};
use super::send_request;
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes};
use graphql_client::{GraphQLQuery, Response};
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/utils/subgraph/query/schema.json",
    query_path = "tests/utils/subgraph/query/orderbook/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct OrderBook;

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResponse {
    pub id: Address,
    pub deployer: Address,
    pub address: Address,
    pub meta: Bytes,
}

impl QueryResponse {
    pub fn from(response: ResponseData) -> QueryResponse {
        let orderbook: OrderBookOrderBook = response.order_book.unwrap();

        let meta_bytes = orderbook
            .meta
            .unwrap_or(OrderBookOrderBookMeta {
                id: Bytes::from([0u8, 32]),
            })
            .id;

        QueryResponse {
            id: Address::from_slice(&orderbook.id),
            address: Address::from_slice(&orderbook.address),
            deployer: Address::from_slice(&orderbook.deployer.unwrap_or_default()),
            meta: meta_bytes,
        }
    }
}

pub async fn get_query(id: &Address) -> Result<QueryResponse> {
    let variables = Variables {
        orderbook: format!("{:?}", id).to_string().into(),
    };
    let request_body = OrderBook::build_query(variables);
    let response: Response<ResponseData> = send_request(request_body).await?.json().await?;

    match response.data {
        Some(data) => Ok(QueryResponse::from(data)),
        None => Err(anyhow!("Failed to get query")),
    }
}
