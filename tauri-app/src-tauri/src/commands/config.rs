use crate::error::CommandResult;
use dotrain::RainDocument;
use rain_orderbook_app_settings::config::Config;

#[tauri::command]
pub fn parse_yaml(text: Vec<String>, validate: bool) -> CommandResult<Config> {
    Ok(Config::try_from_yaml(text, validate)?)
}

#[tauri::command]
pub fn parse_dotrain_and_yaml(
    dotrain: String,
    settings: String,
    validate: bool,
) -> CommandResult<Config> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str())
        .unwrap_or("")
        .to_string();
    Ok(Config::try_from_yaml(
        vec![frontmatter, settings],
        validate,
    )?)
}
