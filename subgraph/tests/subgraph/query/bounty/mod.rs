use self::bounty::ResponseData;
use super::SG_URL;
use crate::utils::{bytes_to_h256, hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes, TxHash, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

// use bigdecimal::BigDecimal;
type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/bounty/bounty.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Bounty;

#[derive(Serialize, Deserialize, Debug)]
pub struct BountyResponse {
    pub id: String,
    pub clearer: Address,
    pub order_clear: String,
    pub bounty_vault_a: String,
    pub bounty_vault_b: String,
    pub bounty_token_a: Address,
    pub bounty_token_b: Address,
    pub bounty_amount_a: Option<U256>,
    pub bounty_amount_a_display: Option<String>,
    pub bounty_amount_b: Option<U256>,
    pub bounty_amount_b_display: Option<String>,
    pub transaction: TxHash,
    pub emitter: Address,
    pub timestamp: U256,
}

impl BountyResponse {
    pub fn from(response: ResponseData) -> BountyResponse {
        let data = response.bounty.unwrap();

        let clearer = Address::from_slice(&data.clearer.id);

        let bounty_token_a =
            Address::from_slice(&hex_string_to_bytes(&data.bounty_token_a.id).unwrap());
        let bounty_token_b =
            Address::from_slice(&hex_string_to_bytes(&data.bounty_token_b.id).unwrap());

        let bounty_amount_a = match data.bounty_amount_a {
            Some(value) => Some(mn_mpz_to_u256(&value)),
            None => None,
        };
        let bounty_amount_b = match data.bounty_amount_b {
            Some(value) => Some(mn_mpz_to_u256(&value)),
            None => None,
        };

        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        BountyResponse {
            id: data.id,
            clearer,
            order_clear: data.order_clear.id,
            bounty_vault_a: data.bounty_vault_a.id,
            bounty_vault_b: data.bounty_vault_b.id,
            bounty_token_a,
            bounty_token_b,
            bounty_amount_a,
            bounty_amount_a_display: data.bounty_amount_a_display,
            bounty_amount_b,
            bounty_amount_b_display: data.bounty_amount_b_display,
            transaction,
            emitter,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_bounty(id: &String) -> Result<BountyResponse> {
    let variables = bounty::Variables {
        id: id.to_string().into(),
    };

    let request_body = Bounty::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = BountyResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
