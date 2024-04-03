use crate::error::CommandResult;
use rain_orderbook_app_settings::{config::Config, config_source::ConfigSource};
use rain_orderbook_common::frontmatter::parse_frontmatter;

#[tauri::command]
pub async fn parse_configstring(text: String) -> CommandResult<ConfigSource> {
    Ok(ConfigSource::try_from_string(text).await?)
}

#[tauri::command]
pub async fn merge_configstrings(dotrain: String, config_text: String) -> CommandResult<ConfigSource> {
    let mut dotrain_config = parse_frontmatter(dotrain).await?;
    let config = ConfigSource::try_from_string(config_text).await?;
    dotrain_config.merge(config)?;
    Ok(dotrain_config)
}

#[tauri::command]
pub fn convert_configstring_to_config(config_string: ConfigSource) -> CommandResult<Config> {
    Ok(config_string.try_into()?)
}
