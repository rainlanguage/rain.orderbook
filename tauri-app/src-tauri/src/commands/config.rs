use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::{config::Config, config_source::ConfigSource};

#[tauri::command]
pub async fn parse_configstring(text: String) -> CommandResult<ConfigSource> {
    Ok(ConfigSource::try_from_string(text, None).await?.0)
}

#[tauri::command]
pub async fn merge_configstrings(
    dotrain: String,
    config_text: String,
) -> CommandResult<ConfigSource> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str())
        .unwrap_or("")
        .to_string();
    let (mut dotrain_config, config) =
        ConfigSource::try_from_string(frontmatter, Some(config_text)).await?;
    dotrain_config.merge(config)?;
    Ok(dotrain_config)
}

#[tauri::command]
pub fn convert_configstring_to_config(config_string: ConfigSource) -> CommandResult<Config> {
    Ok(config_string.try_into()?)
}
