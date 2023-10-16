use self::rain_meta_v1::{RainMetaV1RainMetaV1Content, ResponseData};
use crate::subgraph::wait;
use anyhow::{anyhow, Result};
use ethers::types::Bytes;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/query/schema.json",
    query_path = "tests/subgraph/query/rain_meta_v1/rain_meta_v1.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]
pub struct RainMetaV1;

#[derive(Serialize, Deserialize, Debug)]
pub struct RainMetaV1Response {
    pub id: Bytes,
    pub meta_bytes: Bytes,
    pub content: Vec<Bytes>,
}

impl RainMetaV1Response {
    pub fn from(response: ResponseData) -> RainMetaV1Response {
        let data = response.rain_meta_v1.unwrap();

        let content_data: Vec<RainMetaV1RainMetaV1Content> =
            data.content.unwrap_or(vec![RainMetaV1RainMetaV1Content {
                id: Bytes::from([0u8, 32]),
            }]);

        let content: Vec<Bytes> = content_data.iter().map(|meta| meta.id.clone()).collect();

        RainMetaV1Response {
            id: data.id,
            meta_bytes: data.meta_bytes,
            content,
        }
    }
}

pub async fn get_rain_meta_v1(rain_meta_id: Bytes) -> Result<RainMetaV1Response> {
    wait().await?;

    let url = Url::from_str(&"http://localhost:8000/subgraphs/name/test/test")
        .expect("cannot get the sg url");

    let variables = rain_meta_v1::Variables {
        rain_meta: rain_meta_id.to_string().into(),
    };

    let request_body = RainMetaV1::build_query(variables);
    let client = reqwest::Client::new();
    let res = client.post(url.clone()).json(&request_body).send().await?;

    let response_body: Response<rain_meta_v1::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response: RainMetaV1Response = RainMetaV1Response::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get metaboard")),
    }
}
