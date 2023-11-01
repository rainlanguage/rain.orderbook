use self::sync_status::Health;
use crate::utils::get_block_number;
use anyhow::{anyhow, format_err};
use graphql_client::{GraphQLQuery, Response};
use reqwest::Url;
use rust_bigint::BigInt;
use std::thread;
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::timeout;
// use web3::types::U256;
use ethers::types::U256;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/subgraph/wait/schema.json",
    query_path = "tests/subgraph/wait/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]

pub struct SyncStatus;

pub async fn wait() -> anyhow::Result<bool> {
    let block_number = get_block_number().await?;

    // let _ = get_orderbook().await.expect("cannot get OB in waiting");

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
                return Err(anyhow!("wait function timeout in sync"));
            }
        } else {
            println!("Errors : {:?}", response_body.errors.unwrap());
        }
        thread::sleep(Duration::from_secs(1));
    }
}

/// Check if the subgraph node is live to be able to deploy subgraphs
pub async fn _check_subgraph_node() -> bool {
    let client = reqwest::Client::new();

    let url = "http://localhost:8030";

    let mut retries = 0;
    // Max retries allowed
    let max_retries = 6;
    // Retry interval
    let retry_interval = Duration::from_secs(5);

    loop {
        retries += 1;
        // Send an HTTP GET request with a timeout
        let response = timeout(Duration::from_secs(5), client.get(url).send())
            .await
            .expect("No reqyest sent to the url");

        match response {
            Ok(res) if (res.status().is_success()) => {
                return true;
            }
            _ => {
                if retries >= max_retries {
                    if retries >= max_retries {
                        println!("Max retries reached. Exiting.");
                        // return Err(reqwest::Error::from("Max retries reached"));
                        return false;
                    }
                }
                println!(
                    "Retry attempt {} failed. Retrying in {} seconds...",
                    retries,
                    retry_interval.as_secs()
                );
                tokio::time::sleep(retry_interval).await;
            }
        }
    }
}
