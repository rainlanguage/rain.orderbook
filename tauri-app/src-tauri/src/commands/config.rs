use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::config::Config;

#[tauri::command]
pub fn parse_configstring(text: String, validate: bool) -> CommandResult<Config> {
    Ok(Config::try_from_yaml(vec![text], validate)?)
}

#[tauri::command]
pub fn merge_configstrings(
    dotrain: String,
    config_text: String,
    validate: bool,
) -> CommandResult<Config> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Config::try_from_yaml(
        vec![frontmatter, config_text],
        validate,
    )?)
}
