use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SentryYaml {}
impl SentryYaml {
    pub fn try_from_string(source: &str) -> Result<Option<String>, YamlError> {
        let doc = &load_yaml(source)?;

        if let Some(sentry) = optional_string(doc, "sentry") {
            Ok(Some(sentry))
        } else {
            Ok(None)
        }
    }
}
