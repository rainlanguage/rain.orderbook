use crate::error::CommandResult;
use rain_orderbook_app_settings::{
    config::{Config, ParseConfigStringError},
    AppSettings,
};

#[tauri::command]
pub fn parse_settings(text: String) -> CommandResult<AppSettings> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn parse_settings2(text: String) -> Result<Config, String> {
    Ok(text
        .try_into()
        .map_err(|e: ParseConfigStringError| e.to_string())?)
}
