use crate::error::CommandResult;
use rain_orderbook_app_settings::{
    config::{Config, ParseConfigStringError},
    string_structs::ConfigString,
};
use rain_orderbook_common::frontmatter::get_merged_config;

#[tauri::command]
pub fn parse_config(text: String) -> CommandResult<Config> {
    Ok(text.try_into()?)
}

#[tauri::command]
pub fn merge_config(dotrain: String, top_config: Config) -> CommandResult<Config> {
    Ok(get_merged_config(dotrain.as_str(), Some(&top_config))?)
}

#[tauri::command]
pub fn get_config(dotrain: String, setting_text: String) -> CommandResult<Config> {
    let mut top_str_config: ConfigString = setting_text
        .try_into()
        .map_err(|v| ParseConfigStringError::YamlDeserializerError(v))?;

    let frontmatter = if let Some(splitter) = dotrain.find("---") {
        &dotrain[..splitter]
    } else {
        ""
    };
    let dotrain_str_config: ConfigString = frontmatter
        .try_into()
        .map_err(|v| ParseConfigStringError::YamlDeserializerError(v))?;

    top_str_config.merge(dotrain_str_config)?;
    Ok(top_str_config.try_into()?)
}
