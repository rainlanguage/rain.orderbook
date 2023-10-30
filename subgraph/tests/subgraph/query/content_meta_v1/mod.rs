use self::content_meta_v1::ResponseData;
use super::SG_URL;
use crate::utils::mn_mpz_to_u256;
use anyhow::{anyhow, Result};
use ethers::types::{Bytes, U256};
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/content_meta_v1/content_meta_v1.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct ContentMetaV1;

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentMetaV1Response {
    pub id: Bytes,
    pub raw_bytes: Bytes,
    pub magic_number: U256,
    pub payload: Bytes,
    pub parents: Vec<Bytes>,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
    pub content_language: Option<String>,
}

impl ContentMetaV1Response {
    pub fn from(response: ResponseData) -> ContentMetaV1Response {
        let data = response.content_meta_v1.unwrap();

        let parents: Vec<Bytes> = data.parents.iter().map(|meta| meta.id.clone()).collect();

        ContentMetaV1Response {
            id: data.id,
            raw_bytes: data.raw_bytes,
            magic_number: mn_mpz_to_u256(&data.magic_number),
            payload: data.payload,
            parents,
            content_type: data.content_type,
            content_encoding: data.content_encoding,
            content_language: data.content_language,
        }
    }
}

pub async fn get_content_meta_v1(id: &Bytes) -> Result<ContentMetaV1Response> {
    let variables = content_meta_v1::Variables {
        content_meta: id.to_string().into(),
    };

    let request_body = ContentMetaV1::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post((*SG_URL).clone())
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<content_meta_v1::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response: ContentMetaV1Response = ContentMetaV1Response::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get query")),
    }
}
