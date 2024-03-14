use crate::error::CommandResult;
use rain_orderbook_app_settings::string_structs::ConfigString;

#[tauri::command]
pub fn parse_config(text: String) -> CommandResult<ConfigString> {
    Ok(text.try_into()?)
}
