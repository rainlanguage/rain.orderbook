use crate::utils::setup::get_provider;
use crate::utils::utils::_get_block_number;
use anyhow::{anyhow, format_err};
use graphql_client::{GraphQLQuery, Response};
use reqwest::Url;
use rust_bigint::BigInt;
use std::thread;
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use web3::types::U256;

use self::sync_status::Health;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/common/wait/schema.json",
    query_path = "tests/common/wait/query.graphql",
    reseponse_derives = "Debug, Serialize, Deserialize"
)]

pub struct SyncStatus;

pub async fn wait() -> anyhow::Result<bool> {
    // let block_number = get_web3()?.eth().block_number().await?;
    let provider = get_provider().await.expect("cannot get provider");
    let block_number = _get_block_number(provider.clone()).await;

    let url = Url::from_str(&"http://localhost:8030/graphql")?;
    let variables = sync_status::Variables {};

    let request_body = SyncStatus::build_query(variables);
    let client = reqwest::Client::new();
    let deadline = SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(5);

    loop {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let response = client.post(url.clone()).json(&request_body).send().await?;

        let response_body: Response<sync_status::ResponseData> =
            response.json().await.expect("cannot awit json respon sg");

        if let Some(data) = response_body.data.and_then(|data| Some(data)) {
            let sync_data = data
                .indexing_status_for_current_version
                .expect("failed on: getting indexing_status_for_current_version");

            if sync_data.synced {
                let chain = &sync_data.chains[0];

                let latest_block = &chain.latest_block.as_ref().unwrap().number;
                let latest_block = U256::from_dec_str(&latest_block.to_str_radix(16))
                    .unwrap()
                    .as_u64();

                let health = &sync_data.health;

                if latest_block >= block_number.as_u64() {
                    return Ok(true);
                } else if let Health::failed = health {
                    return Err(format_err!("Fatal error : {:?}", response_body.errors));
                } else if deadline < current_time {
                    return Err(anyhow!("wait function timeout"));
                }
            } else if deadline < current_time {
                return Err(anyhow!("wait function timeout"));
            }
        } else {
            println!("Errors : {:?}", response_body.errors.unwrap());
        }
        thread::sleep(Duration::from_secs(1));
    }
}
