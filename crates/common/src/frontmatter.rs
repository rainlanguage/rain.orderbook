use dotrain::RainDocument;
use rain_orderbook_app_settings::{config::ParseConfigSourceError, config_source::ConfigSource};

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn parse_frontmatter(dotrain: String) -> Result<ConfigSource, ParseConfigSourceError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Ok(frontmatter.to_string().try_into()?)
}
