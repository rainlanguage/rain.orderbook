use self::order::ResponseData;
use super::SG_URL;
use crate::utils::{hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/order/order.graphql",
    response_derives = "Debug, Serialize, Deserialize"
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
    pub valid_inputs: Vec<String>,
    pub valid_outputs: Vec<String>,
    pub order_json_string: String,
    pub expression_json_string: Option<String>,
    pub transaction: Bytes,
    pub emitter: Address,
    pub timestamp: U256,
    pub take_orders: Vec<String>,
    pub orders_clears: Vec<String>,
}

impl OrderResponse {
    pub fn from(response: ResponseData) -> OrderResponse {
        let data = response.order.unwrap();

        let valid_inputs: Vec<String> = data
            .valid_inputs
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let valid_outputs: Vec<String> = data
            .valid_outputs
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let take_orders: Vec<String> = data
            .take_orders
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let orders_clears: Vec<String> = data
            .orders_clears
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        OrderResponse {
            id: hex_string_to_bytes(&data.id).expect("not a hex value"),
            order_hash: data.order_hash,
            owner: Address::from_slice(&data.owner.id),
            interpreter: Address::from_slice(&data.interpreter),
            interpreter_store: Address::from_slice(&data.interpreter_store),
            expression_deployer: Address::from_slice(&data.expression_deployer),
            expression: Address::from_slice(&data.expression),
            order_active: data.order_active,
            handle_i_o: data.handle_io,
            meta: data.meta.unwrap().id,
            valid_inputs,
            valid_outputs,
            order_json_string: data.order_json_string,
            expression_json_string: data.expression_json_string,
            transaction: hex_string_to_bytes(&data.transaction.id).unwrap(),
            emitter: Address::from_slice(&data.emitter.id),
            timestamp: mn_mpz_to_u256(&data.timestamp),
            take_orders,
            orders_clears,
        }
    }
}

pub async fn get_order(id: &Bytes) -> Result<OrderResponse> {
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
            let response = OrderResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
