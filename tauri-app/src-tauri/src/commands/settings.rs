use crate::error::CommandResult;
use rain_orderbook_app_settings::AppSettings;

#[tauri::command]
pub fn parse_settings(text: String) -> CommandResult<AppSettings> {
    Ok(text.try_into()?)
}
