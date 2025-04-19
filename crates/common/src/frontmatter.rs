use dotrain::RainDocument;
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{Config, ParseConfigError};

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn parse_frontmatter(dotrain: String, validate: bool) -> Result<Config, ParseConfigError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Config::try_from_settings(vec![frontmatter.to_string()], validate)
}
