use self::vault::ResponseData;
use super::SG_URL;
use anyhow::{anyhow, Result};
use ethers::types::{Address, Bytes};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/vault/vault.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Vault;

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultResponse {
    pub id: String,
    pub owner: Address,
    pub token_vaults: Vec<String>,
    pub deposits: Vec<String>,
    pub withdraws: Vec<String>,
}

impl VaultResponse {
    pub fn from(response: ResponseData) -> VaultResponse {
        let data = response.vault.unwrap();

        let token_vaults: Vec<String> = data
            .token_vaults
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let deposits: Vec<String> = data
            .deposits
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        let withdraws: Vec<String> = data
            .withdraws
            .unwrap()
            .iter()
            .map(|data| data.id.clone())
            .collect();

        VaultResponse {
            id: data.id,
            owner: Address::from_slice(&data.owner.id),
            token_vaults,
            deposits,
            withdraws,
        }
    }
}

pub async fn get_vault(id: &String) -> Result<VaultResponse> {
    let variables = vault::Variables {
        id: id.to_string().into(),
    };

    let request_body = Vault::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<vault::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = VaultResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
