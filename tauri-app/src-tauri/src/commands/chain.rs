use alloy_ethers_typecast::transaction::ReadableClientHttp;

#[tauri::command]
pub async fn get_chainid(rpc_url: String) -> Result<u64, String> {
    let chain_id = ReadableClientHttp::new_from_url(rpc_url)
        .map_err(|_| String::from("Failed to connect to RPC URL"))?
        .get_chainid()
        .await
        .map_err(|_| String::from("Failed to get Chain ID"))?;

    let chain_id_u64: u64 = chain_id
        .try_into()
        .map_err(|_| String::from("Failed to convert Chain ID to u64"))?;

    Ok(chain_id_u64)
}
