use alloy_primitives::bytes::Bytes;
use rain_orderbook_common::fork::fork_parse_rainlang;
use crate::error::CommandResult;

#[tauri::command]
pub async fn fork_parse(
    rainlang: &str,
    front_matter: &str,
    fork_url: &str,
    fork_block_number: u64,
) -> CommandResult<Bytes> {
    Ok(
        fork_parse_rainlang(rainlang, front_matter, fork_url, fork_block_number)
            .await
            .map(Bytes::from)?,
    )
}
