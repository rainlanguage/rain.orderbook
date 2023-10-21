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
    reseponse_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct IO;

#[derive(Serialize, Deserialize, Debug)]
pub struct IOResponse {
    pub id: String,
    pub token: Address,
    pub decimals: u8,
    pub vault: String,
    pub vaultId: U256,
    pub order: Bytes,
    pub tokenVault: String,
    pub index: u8,
}

impl IOResponse {
    pub fn from(response: ResponseData) -> IOResponse {
        let data = response.io.unwrap();

        let id = data.id;

        IOResponse {
            id: "".to_string(),
            token: Address::from([0u8; 20]),
            decimals: 0,
            vault: "".to_string(),
            vaultId: U256::from(0),
            order: Bytes::from(vec![0]),
            tokenVault: "".to_string(),
            index: 0,
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
