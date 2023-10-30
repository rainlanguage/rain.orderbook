use self::vault_deposit::ResponseData;
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
    query_path = "tests/subgraph/query/vault_deposit/vault_deposit.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VaultDeposit;

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultDepositResponse {
    pub id: String,
    pub sender: Address,
    pub token: Address,
    pub vault_id: U256,
    pub vault: String,
    pub amount: U256,
    pub amount_display: String,
    pub token_vault: String,
    pub emitter: Address,
    pub transaction: TxHash,
    pub timestamp: U256,
}

impl VaultDepositResponse {
    pub fn from(response: ResponseData) -> VaultDepositResponse {
        let data = response.vault_deposit.unwrap();

        let sender = Address::from_slice(&data.sender.id);
        let token = Address::from_slice(&hex_string_to_bytes(&data.token.id).unwrap());
        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        VaultDepositResponse {
            id: data.id,
            sender,
            token,
            vault_id: mn_mpz_to_u256(&data.vault_id),
            vault: data.vault.id,
            amount: mn_mpz_to_u256(&data.amount),
            amount_display: data.amount_display,
            token_vault: data.token_vault.id,
            emitter,
            transaction,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_vault_deposit(id: &String) -> Result<VaultDepositResponse> {
    let variables = vault_deposit::Variables {
        id: id.to_string().into(),
    };

    let request_body = VaultDeposit::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = VaultDepositResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
