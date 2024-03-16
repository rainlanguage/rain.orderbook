use dotrain::RainDocument;
use rain_orderbook_app_settings::{config::ParseConfigStringError, string_structs::ConfigString};

/// Parse dotrain frontmatter and merges it with top Config if given
pub fn parse_frontmatter(dotrain: String) -> Result<ConfigString, ParseConfigStringError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Ok(frontmatter.to_string().try_into()?)
}
