use crate::error::CommandResult;
use alloy_primitives::{bytes::Bytes, Address};
use rain_orderbook_common::rainlang::parse_rainlang_on_fork;

#[tauri::command]
pub async fn parse_dotrain(
    rainlang: &str,
    rpc_url: &str,
    block_number: u64,
    deployer: Address,
) -> CommandResult<Bytes> {
    Ok(parse_rainlang_on_fork(rainlang, rpc_url, Some(block_number), deployer).await?)
}
