use self::order_clear_state_change::ResponseData;
use super::SG_URL;
use crate::utils::mn_mpz_to_u256;
use anyhow::{anyhow, Result};
use ethers::types::U256;
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/order_clear_state_change/order_clear_state_change.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderClearStateChange;

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderClearStateChangeResponse {
    pub id: String,
    pub order_clear: String,
    pub a_output: U256,
    pub b_output: U256,
    pub a_input: U256,
    pub b_input: U256,
}

impl OrderClearStateChangeResponse {
    pub fn from(response: ResponseData) -> OrderClearStateChangeResponse {
        let data = response.order_clear_state_change.unwrap();

        OrderClearStateChangeResponse {
            id: data.id,
            order_clear: data.order_clear.id,
            a_output: mn_mpz_to_u256(&data.a_output),
            b_output: mn_mpz_to_u256(&data.b_output),
            a_input: mn_mpz_to_u256(&data.a_input),
            b_input: mn_mpz_to_u256(&data.b_input),
        }
    }
}

pub async fn get_order_clear_state_change(id: &String) -> Result<OrderClearStateChangeResponse> {
    let variables = order_clear_state_change::Variables {
        id: id.to_string().into(),
    };

    let request_body = OrderClearStateChange::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = OrderClearStateChangeResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
