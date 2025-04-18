use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::config::Config;

#[tauri::command]
pub fn parse_configstring(text: String) -> CommandResult<Config> {
    Ok(Config::try_from_settings(vec![text])?)
}

#[tauri::command]
pub fn merge_configstrings(dotrain: String, config_text: String) -> CommandResult<Config> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Config::try_from_settings(vec![frontmatter, config_text])?)
}
