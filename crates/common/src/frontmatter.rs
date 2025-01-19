use dotrain::RainDocument;
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{config::ParseConfigSourceError, config_source::ConfigSource};

/// Parse dotrain frontmatter and merges it with top Config if given
pub async fn parse_frontmatter(dotrain: String) -> Result<ConfigSource, ParseConfigSourceError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Ok(ConfigSource::try_from_string(frontmatter.to_string(), None)
        .await?
        .0)
}
