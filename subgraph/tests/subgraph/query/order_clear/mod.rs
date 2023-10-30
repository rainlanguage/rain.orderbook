use self::order_clear::ResponseData;
use super::SG_URL;
use crate::utils::{bytes_to_h256, hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::TxHash;
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/order_clear/order_clear.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderClear;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderClearResponse {
    pub id: String,
    pub sender: Address,
    pub clearer: Address,
    pub order_a: Bytes,
    pub order_b: Bytes,
    pub owners: [Address; 2],
    pub a_input_io_index: U256,
    pub a_output_io_index: U256,
    pub b_input_io_index: U256,
    pub b_output_io_index: U256,
    pub bounty: String,
    pub state_change: String,
    pub transaction: TxHash,
    pub emitter: Address,
    pub timestamp: U256,
}

impl OrderClearResponse {
    pub fn from(response: ResponseData) -> OrderClearResponse {
        let data = response.order_clear.unwrap();

        let sender = Address::from_slice(&data.sender.id);
        let clearer = Address::from_slice(&data.clearer.id);
        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        let owners_vec = data.owners.unwrap();
        let owners: [Address; 2] = [
            Address::from_slice(&owners_vec.get(0).unwrap().id),
            Address::from_slice(&owners_vec.get(1).unwrap().id),
        ];

        OrderClearResponse {
            id: data.id,
            sender,
            clearer,
            order_a: hex_string_to_bytes(&data.order_a.id).expect("not a hex value"),
            order_b: hex_string_to_bytes(&data.order_b.id).expect("not a hex value"),
            owners,
            a_input_io_index: mn_mpz_to_u256(&data.a_input_io_index),
            a_output_io_index: mn_mpz_to_u256(&data.a_output_io_index),
            b_input_io_index: mn_mpz_to_u256(&data.b_input_io_index),
            b_output_io_index: mn_mpz_to_u256(&data.b_output_io_index),
            bounty: data.bounty.id,
            state_change: data.state_change.id,
            transaction,
            emitter,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_order_clear(id: &String) -> Result<OrderClearResponse> {
    let variables = order_clear::Variables {
        id: id.to_string().into(),
    };

    let request_body = OrderClear::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = OrderClearResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
