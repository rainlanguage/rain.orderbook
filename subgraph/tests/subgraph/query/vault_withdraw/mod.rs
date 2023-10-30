use self::vault_withdraw::ResponseData;
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
    query_path = "tests/subgraph/query/vault_withdraw/vault_withdraw.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct VaultWithdraw;

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultWithdrawResponse {
    pub id: String,
    pub sender: Address,
    pub token: Address,
    pub vault_id: U256,
    pub vault: String,
    pub requested_amount: U256,
    pub requested_amount_display: String,
    pub amount: U256,
    pub amount_display: String,
    pub token_vault: String,
    pub emitter: Address,
    pub transaction: TxHash,
    pub timestamp: U256,
}

impl VaultWithdrawResponse {
    pub fn from(response: ResponseData) -> VaultWithdrawResponse {
        let data = response.vault_withdraw.unwrap();

        let sender = Address::from_slice(&data.sender.id);
        let token = Address::from_slice(&hex_string_to_bytes(&data.token.id).unwrap());
        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        VaultWithdrawResponse {
            id: data.id,
            sender,
            token,
            vault_id: mn_mpz_to_u256(&data.vault_id),
            vault: data.vault.id,
            requested_amount: mn_mpz_to_u256(&data.requested_amount),
            requested_amount_display: data.requested_amount_display,
            amount: mn_mpz_to_u256(&data.amount),
            amount_display: data.amount_display,
            token_vault: data.token_vault.id,
            emitter,
            transaction,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_vault_withdraw(id: &String) -> Result<VaultWithdrawResponse> {
    let variables = vault_withdraw::Variables {
        id: id.to_string().into(),
    };

    let request_body = VaultWithdraw::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = VaultWithdrawResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
