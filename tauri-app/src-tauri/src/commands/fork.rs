use alloy_primitives::bytes::Bytes;
use rain_orderbook_common::{add_order::AddOrderArgs, fork::fork_call};
use alloy_ethers_typecast::transaction::ReadableClientHttp;

#[tauri::command]
pub async fn fork_parse(dotrain_string: String) -> Result<Bytes, String> {
    let rain_document = RainDocument::create(dotrrain_string);
    // let (deployer, _, _) = x.
    // let chain_id = ReadableClientHttp::new_from_url(rpc_url)?
    //     .get_chainid()
    //     .await?;

    // let chain_id_u64: u64 = chain_id.try_into()?;

    // Ok(chain_id_u64)
}