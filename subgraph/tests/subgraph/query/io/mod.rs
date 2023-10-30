use self::io::ResponseData;
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
    query_path = "tests/subgraph/query/io/io.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct IO;

#[derive(Serialize, Deserialize, Debug)]
pub struct IOResponse {
    pub id: String,
    pub token: Address,
    pub decimals: u8,
    pub vault_id: U256,
    pub order: Bytes,
    pub index: u8,
    pub vault: String,
    pub token_vault: String,
}

impl IOResponse {
    pub fn from(response: ResponseData) -> IOResponse {
        let data = response.io.unwrap();

        let token = Address::from_slice(
            hex_string_to_bytes(&data.token.id)
                .unwrap()
                .to_vec()
                .as_slice(),
        );

        IOResponse {
            id: data.id,
            token,
            decimals: data.decimals as u8,
            vault: data.vault.id,
            vault_id: mn_mpz_to_u256(&data.vault_id),
            order: hex_string_to_bytes(&data.order.id).expect("not a hex value"),
            token_vault: data.token_vault.id,
            index: data.index as u8,
        }
    }
}

pub async fn get_i_o(id: &String) -> Result<IOResponse> {
    let variables = io::Variables {
        id: id.to_string().into(),
    };

    let request_body = IO::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = IOResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
