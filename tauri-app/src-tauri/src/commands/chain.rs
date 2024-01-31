use crate::error::CommandResult;
use alloy_ethers_typecast::transaction::ReadableClientHttp;

#[tauri::command]
pub async fn get_chainid(rpc_url: String) -> CommandResult<u64> {
    let chain_id = ReadableClientHttp::new_from_url(rpc_url)?
        .get_chainid()
        .await?;

    let chain_id_u64: u64 = chain_id.try_into()?;

    Ok(chain_id_u64)
}
