use alloy_primitives::bytes::Bytes;
use rain_orderbook_common::fork::fork_parse_rainlang;
use crate::error::CommandResult;

#[tauri::command]
pub async fn fork_parse(
    frontmatter: &str,
    rainlang: &str,
    rpc_url: &str,
    block_number: u64,
) -> CommandResult<Bytes> {
    Ok(
        fork_parse_rainlang(frontmatter, rainlang, rpc_url, block_number)
            .await
            .map(Bytes::from)?,
    )
}
