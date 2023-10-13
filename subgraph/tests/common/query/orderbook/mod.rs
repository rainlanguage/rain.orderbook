use self::meta_board::ResponseData;
use crate::common::wait::wait;
use anyhow::anyhow;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Url;
use rust_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf as Bytes;
use std::fmt::Debug;
use std::str::FromStr;
use web3::types::{Address, H160, U256};

#[derive(GraphQLQuery)]
#[graphql(
    // schema_path = "tests/common/query/schema.json",
    query_path = "tests/common/query/orderbook/orderbook.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]

pub struct MetaBoard;

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaV1Response {
    pub id: Address,
    pub address: Address,
    pub meta_count: U256,
    pub metas: Vec<String>,
}

impl MetaV1Response {
    pub fn from(response: ResponseData) -> MetaV1Response {
        let meta_board = response.meta_board.unwrap();
        let metas = meta_board.metas.unwrap();

        MetaV1Response {
            id: H160::from_str(&String::from_utf8(meta_board.id.to_vec()).unwrap()).unwrap(),
            address: H160::from_str(&String::from_utf8(meta_board.address.to_vec()).unwrap())
                .unwrap(),
            meta_count: U256::from_dec_str(&meta_board.meta_count.to_str_radix(16)).unwrap(),
            metas: metas
                .iter()
                .map(|meta| String::from_utf8_lossy(&meta.id.to_vec()).to_string())
                .collect(),
        }
    }
}

pub async fn get_meta_board(meta_board_id: &str) -> anyhow::Result<MetaV1Response> {
    wait().await?;

    let url = Url::from_str(&"http://localhost:8000/subgraphs/name/test/test")?;
    let variables = meta_board::Variables {
        metaboard: meta_board_id.to_string().into(),
    };
    let request_body = MetaBoard::build_query(variables);
    let client = reqwest::Client::new();
    let res = client.post(url.clone()).json(&request_body).send().await?;

    let response_body: Response<meta_board::ResponseData> = res.json().await?;

    match response_body.data {
        Some(data) => {
            let response: MetaV1Response = MetaV1Response::from(data);
            Ok(response)
        }
        None => Err(anyhow!("Failed to get metaboard")),
    }
}
