use crate::commands::config::merge_configstrings;
use crate::error::CommandResult;
use rain_orderbook_app_settings::config::*;
use rain_orderbook_common::dotrain_order::{DotrainOrder, ScenariosAuthoringMeta};
use rain_metadata::types::authoring::v2::AuthoringMetaV2;

#[tauri::command]
pub async fn get_authoring_metas(dotrain: String, settings: String) -> CommandResult<ScenariosAuthoringMeta> {
    let order = DotrainOrder::new(dotrain, settings).await?;
    order.get_authoring_metas_for_all_scenarios().await
}