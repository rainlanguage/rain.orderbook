use crate::utils::setup::get_rpc_provider;
use self::sync_status::Health;
use anyhow::{anyhow, format_err, Result};
use ethers::types::U64;
use graphql_client::{GraphQLQuery, Response};
use rust_bigint::BigInt;
use std::thread;
use std::{
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/utils/subgraph/wait/schema.json",
    query_path = "tests/utils/subgraph/wait/query.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]

pub struct SyncStatus;

/// Wait for subgraph synchronization
pub async fn wait() -> Result<()> {
    let provider = get_rpc_provider().await?;

    let block_number = provider.get_block_number().await?;
    let url = "http://localhost:8030/graphql";

    let request_body = SyncStatus::build_query(sync_status::Variables {});

    let client = reqwest::Client::new();
    let deadline = SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(5);

    loop {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let response = client.post(url).json(&request_body).send().await?;

        let response_body: Response<sync_status::ResponseData> = response.json().await?;

        match response_body.data {
            Some(data) => {
                let sync_data = data
                    .indexing_status_for_current_version
                    .ok_or(anyhow!("failed to get indexing status"))?;

                if sync_data.synced {
                    let chain = &sync_data.chains[0];

                    let latest_block = &chain
                        .latest_block
                        .as_ref()
                        .ok_or(anyhow!("failed to get latest block in chain status"))?
                        .number;

                    let latest_block = U64::from_str(&latest_block.to_str_radix(16))?;

                    if latest_block >= block_number {
                        return Ok(());
                    } else if let Health::failed = sync_data.health {
                        return Err(format_err!("Fatal error : {:?}", response_body.errors));
                    } else if deadline < current_time {
                        return Err(anyhow!("subgraph wait timeout to sync"));
                    }
                } else if deadline < current_time {
                    return Err(anyhow!("subgraph wait timeout to sync"));
                }
            }
            None => {
                println!(
                    "Errors : {:?}",
                    response_body
                        .errors
                        .clone()
                        .ok_or(anyhow!("no errors found in response"))
                );
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
