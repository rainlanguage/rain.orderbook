use crate::error::CommandResult;
use rain_orderbook_app_settings::config::Config;
use rain_orderbook_common::frontmatter::get_merged_config;

#[tauri::command]
pub fn parse_config(text: String) -> CommandResult<Config> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn get_config(dotrain: String, setting_text: String) -> CommandResult<Config> {
    Ok(get_merged_config(
        dotrain.as_str(),
        Some(setting_text.as_str()),
    )?)
}
