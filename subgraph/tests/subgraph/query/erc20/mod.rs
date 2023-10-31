use self::erc20::ResponseData;
use super::SG_URL;
use crate::utils::{hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::{Address, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

// use bigdecimal::BigDecimal;
type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/erc20/erc20.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ERC20;

#[derive(Serialize, Deserialize, Debug)]
pub struct ERC20Response {
    pub id: Address,
    pub name: String,
    pub symbol: String,
    pub total_supply: U256,
    pub total_supply_display: String,
    pub decimals: u8,
}

impl ERC20Response {
    pub fn from(response: ResponseData) -> ERC20Response {
        let data = response.erc20.unwrap();

        let id = Address::from_slice(&hex_string_to_bytes(&data.id).unwrap());

        ERC20Response {
            id,
            name: data.name,
            symbol: data.symbol,
            total_supply: mn_mpz_to_u256(&data.total_supply),
            total_supply_display: data.total_supply_display,
            decimals: data.decimals as u8,
        }
    }
}

pub async fn get_erc20(id: &Address) -> Result<ERC20Response> {
    let variables = erc20::Variables {
        id: Some(format!("{:?}", id)),
    };

    let request_body = ERC20::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = ERC20Response::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
