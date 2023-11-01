use self::context_entity::ResponseData;
use super::SG_URL;
use crate::utils::{bytes_to_h256, hex_string_to_bytes, mn_mpz_to_u256};
use anyhow::{anyhow, Result};
use ethers::types::TxHash;
use ethers::types::{Address, Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/context_entity/context_entity.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ContextEntity;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextEntityResponse {
    pub id: String,
    pub caller: Address,
    pub calling_context: Option<Vec<U256>>,
    pub calculations_context: Option<Vec<U256>>,
    pub vault_inputs_context: Option<Vec<U256>>,
    pub vault_outputs_context: Option<Vec<U256>>,
    pub signed_context: Option<Vec<SignedContext>>,
    pub emitter: Address,
    pub transaction: TxHash,
    pub timestamp: U256,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedContext {
    pub id: String,
    pub signer: Address,
    pub context: Option<Vec<U256>>,
}

impl ContextEntityResponse {
    pub fn from(response: ResponseData) -> ContextEntityResponse {
        let data = response.context_entity.unwrap();

        let caller = Address::from_slice(&data.caller.id);

        let calling_context = match data.calling_context {
            Some(values) => {
                let values_collected: Vec<U256> =
                    values.iter().map(|value| mn_mpz_to_u256(&value)).collect();

                Some(values_collected)
            }
            None => None,
        };

        let calculations_context = match data.calculations_context {
            Some(values) => {
                let values_collected: Vec<U256> =
                    values.iter().map(|value| mn_mpz_to_u256(&value)).collect();

                Some(values_collected)
            }
            None => None,
        };

        let vault_inputs_context = match data.vault_inputs_context {
            Some(values) => {
                let values_collected: Vec<U256> =
                    values.iter().map(|value| mn_mpz_to_u256(&value)).collect();

                Some(values_collected)
            }
            None => None,
        };

        let vault_outputs_context = match data.vault_outputs_context {
            Some(values) => {
                let values_collected: Vec<U256> =
                    values.iter().map(|value| mn_mpz_to_u256(&value)).collect();

                Some(values_collected)
            }
            None => None,
        };

        let signed_context = match data.signed_context {
            Some(values) => {
                let values_collected: Vec<SignedContext> = values
                    .iter()
                    .map(|value| SignedContext {
                        id: value.id.clone(),
                        signer: Address::from_slice(&value.signer),
                        context: match value.context.clone() {
                            Some(values) => {
                                let values_collected: Vec<U256> =
                                    values.iter().map(|value| mn_mpz_to_u256(&value)).collect();

                                Some(values_collected)
                            }
                            None => None,
                        },
                    })
                    .collect();

                Some(values_collected)
            }
            None => None,
        };

        let emitter = Address::from_slice(&data.emitter.id);
        let transaction = bytes_to_h256(&hex_string_to_bytes(&data.transaction.id).unwrap());

        ContextEntityResponse {
            id: data.id,
            caller,
            calling_context,
            calculations_context,
            vault_inputs_context,
            vault_outputs_context,
            signed_context,
            emitter,
            transaction,
            timestamp: mn_mpz_to_u256(&data.timestamp),
        }
    }
}

pub async fn get_context_entity(id: &String) -> Result<ContextEntityResponse> {
    let variables = context_entity::Variables {
        id: id.to_string().into(),
    };

    let request_body = ContextEntity::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = ContextEntityResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
