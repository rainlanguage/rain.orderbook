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

/// Parse dotrain frontmatter to extract Config
pub fn try_parse_frontmatter(frontmatter: &str) -> Result<Config, FrontmatterError> {
    if frontmatter.is_empty() {
        return Ok(Config::default());
    }
    Ok(frontmatter.try_into()?)
}

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn get_merged_config(
    dotrain: &str,
    top_config: Option<&str>,
) -> Result<Config, FrontmatterError> {
    let frontmatter = RainDocument::get_front_matter(dotrain).unwrap_or("");
    let mut frontmatter_str_config: ConfigString = frontmatter
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
