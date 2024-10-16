use crate::error::CommandResult;
use alloy::primitives::{bytes::Bytes, Address};
use rain_orderbook_common::{dotrain_order::DotrainOrder, rainlang::parse_rainlang_on_fork};

#[tauri::command]
pub async fn parse_dotrain(
    rainlang: &str,
    rpc_url: &str,
    block_number: u64,
    deployer: Address,
) -> CommandResult<Bytes> {
    Ok(parse_rainlang_on_fork(rainlang, rpc_url, Some(block_number), deployer).await?)
}

#[tauri::command]
pub async fn dotrain_filter_by_deployment(
    dotrain: &str,
    deployments: Vec<String>,
    config: Option<String>,
    include_gui: Option<bool>,
) -> CommandResult<String> {
    Ok(DotrainOrder::new_with_frontmatter_filtered_by_deployment(
        dotrain.to_string(),
        deployments,
        config,
        include_gui,
    )
    .await?
    .dotrain()
    .to_string())
}
