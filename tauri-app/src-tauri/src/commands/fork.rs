use crate::error::CommandResult;
use alloy_primitives::bytes::Bytes;
use rain_orderbook_common::fork::parse_dotrain_fork;

#[tauri::command]
pub async fn parse_dotrain(
    frontmatter: &str,
    rainlang: &str,
    rpc_url: &str,
    block_number: u64,
) -> CommandResult<Bytes> {
    Ok(
        parse_dotrain_fork(frontmatter, rainlang, rpc_url, block_number)
            .await
            .map(Bytes::from)?,
    )
}
