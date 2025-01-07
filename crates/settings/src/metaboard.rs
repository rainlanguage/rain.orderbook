use crate::yaml::{default_document, require_hash, require_string, YamlError, YamlParsableHash};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use typeshare::typeshare;
use url::{ParseError, Url};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Metaboard {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    pub url: Url,
}

impl Metaboard {
    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }
}

impl YamlParsableHash for Metaboard {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Metaboard>, YamlError> {
        let mut metaboards = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            let metaboards_hash = require_hash(
                &document_read,
                Some("metaboards"),
                Some("missing field: metaboards".to_string()),
            )?;

            for (key_yaml, metaboard_yaml) in metaboards_hash {
                let metaboard_key = key_yaml.as_str().unwrap_or_default().to_string();

                let url = Metaboard::validate_url(&require_string(
                    metaboard_yaml,
                    None,
                    Some(format!(
                        "metaboard value must be a string for key: {metaboard_key}"
                    )),
                )?)?;

                let metaboard = Metaboard {
                    document: document.clone(),
                    key: metaboard_key.clone(),
                    url,
                };

                if metaboards.contains_key(&metaboard_key) {
                    return Err(YamlError::KeyShadowing(metaboard_key));
                }
                metaboards.insert(metaboard_key, metaboard);
            }
        }

        Ok(metaboards)
    }
}

impl Default for Metaboard {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            url: Url::parse("https://metaboard.com").unwrap(),
        }
    }
}

impl PartialEq for Metaboard {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_metaboards_from_yaml_multiple_files() {
        let yaml_one = r#"
metaboards:
    MetaboardOne: https://metaboard-one.com
"#;
        let yaml_two = r#"
metaboards:
    MetaboardTwo: https://metaboard-two.com
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let metaboards = Metaboard::parse_all_from_yaml(documents).unwrap();

        assert_eq!(metaboards.len(), 2);
        assert!(metaboards.contains_key("MetaboardOne"));
        assert!(metaboards.contains_key("MetaboardTwo"));

        assert_eq!(
            metaboards.get("MetaboardOne").unwrap().url.as_str(),
            "https://metaboard-one.com"
        );
        assert_eq!(
            metaboards.get("MetaboardTwo").unwrap().url.as_str(),
            "https://metaboard-two.com"
        );
    }

    #[test]
    fn test_parse_metaboards_from_yaml_duplicate_key() {
        let yaml_one = r#"
metaboards:
    DuplicateMetaboard: https://metaboard-one.com
"#;
        let yaml_two = r#"
metaboards:
    DuplicateMetaboard: https://metaboard-two.com
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = Metaboard::parse_all_from_yaml(documents).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateMetaboard".to_string())
        );
    }
}
