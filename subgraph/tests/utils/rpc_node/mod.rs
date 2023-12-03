use crate::utils::get_provider;
use ethers::{
    providers::{Middleware, PendingTransaction},
    types::{Block, TxHash, H256, U64},
};
use reqwest::Client;

/// Mine a single block in localnode
pub async fn mine_block() -> anyhow::Result<()> {
    let json_rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "evm_mine",
        "params": [],
    });

    send_request(json_rpc_request).await?;

    Ok(())
}

/// Increase the time in the localnode by the value given.
/// ### Example
/// ```rust
/// // If the current timestamp is 100 in localnode
/// let time_to_increase = 25;
///
/// // Call the function
/// increase_time(time_to_increase).await?;
///
/// // The new current timestamp will be 125 in localnode
/// ```
pub async fn increase_time(time_to_increase: u64) -> anyhow::Result<()> {
    let json_rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "evm_increaseTime",
        "params": [time_to_increase],
    });

    send_request(json_rpc_request).await?;

    // And mine a new block with the new timestamp
    mine_block().await
}

pub async fn get_block_number() -> anyhow::Result<U64> {
    let provider = get_provider().await?;
    Ok(provider.get_block_number().await?)
}

pub async fn get_block_data_by_tx_hash(tx_hash: &TxHash) -> anyhow::Result<Block<H256>> {
    let provider = get_provider().await?;

    let pending_tx = PendingTransaction::new(*tx_hash, provider);

    let receipt = match pending_tx.await? {
        Some(receipt) => receipt,
        None => return Err(anyhow::Error::msg("receipt not found")),
    };

    let block_number = match receipt.block_number {
        Some(block_number) => block_number,
        None => return Err(anyhow::Error::msg("block number not found")),
    };

    match provider.get_block(block_number).await? {
        Some(block_data) => Ok(block_data),
        None => return Err(anyhow::Error::msg("block data not found")),
    }
}

pub async fn get_block_data_by_number(block_number: U64) -> anyhow::Result<Block<H256>> {
    let provider = get_provider().await?;
    match provider.get_block(block_number).await? {
        Some(block_data) => Ok(block_data),
        None => return Err(anyhow::Error::msg("block data not found")),
    }
}

pub async fn get_current_block_data() -> anyhow::Result<Block<H256>> {
    let current_block = get_block_number().await?;

    get_block_data_by_number(current_block).await
}

async fn send_request(
    json_data_request: serde_json::Value,
) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    let provider = get_provider().await?;

    let response: reqwest::Response = Client::new()
        .post(provider.url().as_str())
        .json(&json_data_request)
        .send()
        .await?;

    if response.status().is_success() {
        let text = response.text().await?;

        match serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text) {
            Ok(parsed_json) => {
                match parsed_json.get("error") {
                    Some(err_value) => match err_value.get("message") {
                        // Return the error message obtained by the RPC node
                        Some(err_message) => Err(anyhow::anyhow!("{}", err_message)),
                        None => Err(anyhow::anyhow!(
                            "Error - No 'message' field in the 'error' object"
                        )),
                    },
                    None => {
                        // If no error key, we can assume that the response was succesfull
                        return Ok(parsed_json);
                    }
                }
            }
            // Return the JSON parse error
            Err(err) => Err(anyhow::anyhow!("Error parsing the response JSON: {}", err)),
        }
    } else {
        Err(anyhow::anyhow!(
            "Failed to communicate with the RPC node. HTTP status code: {}",
            response.status()
        ))
    }
}
