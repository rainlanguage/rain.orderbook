use self::token_vault_take_order::ResponseData;
use super::SG_URL;
use anyhow::{anyhow, Result};
use graphql_client::{GraphQLQuery, Response};
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/token_vault_take_order/token_vault_take_order.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenVaultTakeOrder;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenVaultTakeOrderResponse {
    pub id: String,
    pub was_output: bool,
    pub was_input: bool,
    pub take_order: String,
    pub token_vault: String,
}

impl TokenVaultTakeOrderResponse {
    pub fn from(response: ResponseData) -> TokenVaultTakeOrderResponse {
        let data = response.token_vault_take_order.unwrap();

        TokenVaultTakeOrderResponse {
            id: data.id,
            was_output: data.was_output,
            was_input: data.was_input,
            take_order: data.take_order.id,
            token_vault: data.token_vault.id,
        }
    }
}

pub async fn get_token_vault_take_order(id: &String) -> Result<TokenVaultTakeOrderResponse> {
    let variables = token_vault_take_order::Variables {
        id: id.to_string().into(),
    };

    let request_body = TokenVaultTakeOrder::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response = TokenVaultTakeOrderResponse::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
