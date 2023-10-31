use self::token_vault::ResponseData;
use super::SG_URL;
use crate::utils::{hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

// use bigdecimal::BigDecimal;
type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/token_vault/token_vault.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenVault;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenVaultResponse {
    pub id: String,
    pub owner: Address,
    pub vault: String,
    pub vault_id: U256,
    pub token: Address,
    pub balance: U256,
    pub balance_display: String,
    pub orders: Vec<Bytes>,
    pub take_orders: Vec<String>,
    pub orders_clears: Vec<String>,
}

impl TokenVaultResponse {
    pub fn from(response: ResponseData) -> TokenVaultResponse {
        let data = response.token_vault.unwrap();

        let owner = Address::from_slice(&data.owner.id);
        let token = Address::from_slice(
            hex_string_to_bytes(&data.token.id)
                .unwrap()
                .to_vec()
                .as_slice(),
        );

        let orders: Vec<Bytes> = data
            .orders
            .unwrap()
            .iter()
            .map(|data| hex_string_to_bytes(&data.id).unwrap())
            .collect();

        let take_orders: Vec<String> = data
            .take_orders
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let orders_clears: Vec<String> = data
            .order_clears
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        TokenVaultResponse {
            id: data.id,
            owner,
            vault: data.vault.id,
            vault_id: mn_mpz_to_u256(&data.vault_id),
            token,
            balance: mn_mpz_to_u256(&data.balance),
            balance_display: data.balance_display,
            orders,
            take_orders,
            orders_clears,
        }
    }
}

pub async fn get_token_vault(id: &String) -> Result<TokenVaultResponse> {
    let variables = token_vault::Variables {
        id: id.to_string().into(),
    };

    let request_body = TokenVault::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = TokenVaultResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
