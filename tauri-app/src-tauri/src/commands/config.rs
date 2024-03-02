use crate::error::CommandResult;
use rain_orderbook_app_settings::{config::Config, AppSettings};
use rain_orderbook_common::frontmatter::get_merged_config;

#[tauri::command]
pub fn parse_settings(text: String) -> CommandResult<AppSettings> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn parse_config(text: String) -> CommandResult<Config> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn merge_config(dotrain: String, top_config: Config) -> CommandResult<Config> {
    Ok(get_merged_config(dotrain.as_str(), Some(&top_config))?)
}
