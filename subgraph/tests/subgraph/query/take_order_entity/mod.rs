use self::take_order_entity::ResponseData;
use super::SG_URL;
use crate::utils::{bytes_to_h256, hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::TxHash;
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

// use bigdecimal::BigDecimal;
type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/take_order_entity/take_order_entity.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TakeOrderEntity;

#[derive(Serialize, Deserialize, Debug)]
pub struct TakeOrderEntityResponse {
    pub id: String,
    pub sender: Address,
    pub order: Bytes,
    pub input: U256,
    pub input_display: String,
    pub output: U256,
    pub output_display: String,
    pub io_ratio: String,
    pub input_io_index: U256,
    pub output_io_index: U256,
    pub input_token: Address,
    pub output_token: Address,
    pub context: Option<String>,
    pub emitter: Address,
    pub transaction: TxHash,
    pub timestamp: U256,
}

impl TakeOrderEntityResponse {
    pub fn from(response: ResponseData) -> TakeOrderEntityResponse {
        let data = response.take_order_entity.unwrap();

        let sender = Address::from_slice(&data.sender.id);
        let input_token = Address::from_slice(&hex_string_to_bytes(&data.input_token.id).unwrap());
        let output_token =
            Address::from_slice(&hex_string_to_bytes(&data.output_token.id).unwrap());
        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        let context = match data.context {
            Some(value) => Some(value.id),
            None => None,
        };

        TakeOrderEntityResponse {
            id: data.id,
            sender,
            order: hex_string_to_bytes(&data.order.id).unwrap(),
            input: mn_mpz_to_u256(&data.input),
            input_display: data.input_display,
            output: mn_mpz_to_u256(&data.input),
            output_display: data.output_display,
            io_ratio: data.io_ratio,
            input_io_index: mn_mpz_to_u256(&data.input_io_index),
            output_io_index: mn_mpz_to_u256(&data.output_io_index),
            input_token,
            output_token,
            context,
            emitter,
            transaction,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_vault_deposit(id: &String) -> Result<TakeOrderEntityResponse> {
    let variables = take_order_entity::Variables {
        id: id.to_string().into(),
    };

    let request_body = TakeOrderEntity::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = TakeOrderEntityResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
