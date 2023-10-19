use self::order::{OrderOrder, OrderOrderOwner, ResponseData};
use super::SG_URL;
use crate::{subgraph::wait, utils::mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/order/order.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Order;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderResponse {
    pub id: Bytes,
    pub order_hash: Bytes,
    pub owner: Address,
    pub interpreter: Address,
    pub interpreter_store: Address,
    pub expression_deployer: Address,
    pub expression: Address,
    pub order_active: bool,
    pub handle_i_o: bool,
    pub meta: Bytes,

    pub valid_inputs: Vec<Bytes>,
    pub valid_outputs: Vec<Bytes>,
    pub order_jsonstring: String,
    pub expression_jsonstring: String,

    pub transaction: Bytes,
    pub emitter: Address,
    pub timestamp: U256,

    pub take_orders: Vec<Bytes>,
    pub orders_clears: Vec<Bytes>,
}

impl OrderResponse {
    // pub fn from(response: ResponseData) -> OrderResponse {
    pub fn from(response: ResponseData) -> () {
        let data = response.order.unwrap();

        // Check here.
        let owner: Bytes = data.owner.id;
        println!("owner: {}", owner);
        let emitter = data.emitter.id;

        let meta = data.meta.unwrap().id;

        let valid_inputs = data.valid_inputs.unwrap().get(0).unwrap().id.clone();
        let valid_outputs = data.valid_outputs.unwrap().get(0).unwrap().id.clone();

        let transaction = data.transaction.id;

        let take_orders = data.take_orders.unwrap().get(0).unwrap().id.clone();
        let orders_clears = data.orders_clears.unwrap().get(0).unwrap().id.clone();

        // OrderResponse {
        //     id: data.id
        //     order_active: data.order_active,
        //     owner: data.owner
        // }
        ()
    }
}

// pub async fn get_content_meta_v1(id: Bytes) -> Result<OrderResponse> {
pub async fn get_content_meta_v1(id: Bytes) -> Result<()> {
    wait().await?;

    let variables = order::Variables {
        id: id.to_string().into(),
    };

    let request_body = Order::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<order::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = ();
            // let response: OrderResponse = OrderResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
