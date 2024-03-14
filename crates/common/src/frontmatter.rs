use dotrain::RainDocument;
use rain_orderbook_app_settings::{
    config::{Config, ParseConfigStringError},
    merge::MergeError,
    string_structs::ConfigString,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrontmatterError {
    #[error(transparent)]
    ParseConfigError(#[from] ParseConfigStringError),
    #[error(transparent)]
    MergeError(#[from] MergeError),
}

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn merge_parse_configs(
    dotrain: String,
    top_config: Option<String>,
) -> Result<Config, FrontmatterError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    let mut frontmatter_str_config: ConfigString = frontmatter
        .to_string()
        .try_into()
        .map_err(ParseConfigStringError::YamlDeserializerError)?;
    if let Some(v) = top_config {
        let top_str_config: ConfigString = v
            .try_into()
            .map_err(ParseConfigStringError::YamlDeserializerError)?;
        frontmatter_str_config.merge(top_str_config)?;
        Ok(frontmatter_str_config.try_into()?)
    } else {
        Ok(frontmatter_str_config.try_into()?)
    }
}
