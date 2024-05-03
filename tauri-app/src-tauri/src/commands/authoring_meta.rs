use crate::error::CommandResult;
use rain_orderbook_common::dotrain_order::{DotrainOrder, ScenariosAuthoringMeta};

#[tauri::command]
pub async fn get_authoring_metas(dotrain: String, settings: String) -> CommandResult<ScenariosAuthoringMeta> {
    let order = DotrainOrder::new(dotrain, Some(settings)).await?;
    let authoring_metas = order.get_authoring_metas_for_all_scenarios().await?;
    Ok(authoring_metas)
}