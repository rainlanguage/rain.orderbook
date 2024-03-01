use crate::error::CommandResult;
use rain_orderbook_app_settings::{config::Config, AppSettings};

#[tauri::command]
pub fn parse_settings(text: String) -> CommandResult<AppSettings> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn parse_config(text: String) -> CommandResult<Config> {
    Ok(text.try_into()?)
}
