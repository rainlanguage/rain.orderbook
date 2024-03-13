use crate::error::CommandResult;
use rain_orderbook_app_settings::config::Config;
use rain_orderbook_common::frontmatter::merge_parse_configs as merge_parse_configs_inner;

#[tauri::command]
pub fn merge_parse_configs(dotrain: String, setting_text: String) -> CommandResult<Config> {
    Ok(merge_parse_configs_inner(
        dotrain.as_str(),
        Some(setting_text.as_str()),
    )?)
}
