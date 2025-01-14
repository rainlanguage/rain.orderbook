use crate::yaml::{
    context::Context, default_document, require_hash, require_string, YamlError, YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
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

    pub fn add_record_to_yaml(
        document: Arc<RwLock<StrictYaml>>,
        key: &str,
        value: &str,
    ) -> Result<(), YamlError> {
        Metaboard::validate_url(value)?;

        let mut document = document.write().map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if !document_hash.contains_key(&StrictYaml::String("metaboards".to_string())) {
                document_hash.insert(
                    StrictYaml::String("metaboards".to_string()),
                    StrictYaml::Hash(Hash::new()),
                );
            }

            if let Some(StrictYaml::Hash(ref mut metaboards)) =
                document_hash.get_mut(&StrictYaml::String("metaboards".to_string()))
            {
                if metaboards.contains_key(&StrictYaml::String(key.to_string())) {
                    return Err(YamlError::KeyShadowing(key.to_string()));
                }

                metaboards.insert(
                    StrictYaml::String(key.to_string()),
                    StrictYaml::String(value.to_string()),
                );
            } else {
                return Err(YamlError::ParseError(
                    "missing field: metaboards".to_string(),
                ));
            }
        } else {
            return Err(YamlError::ParseError("document parse error".to_string()));
        }

        Ok(())
    }
}

impl YamlParsableHash for Metaboard {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Metaboard>, YamlError> {
        let mut metaboards = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(metaboards_hash) = require_hash(&document_read, Some("metaboards"), None) {
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
        }

        if metaboards.is_empty() {
            return Err(YamlError::ParseError(
                "missing field: metaboards".to_string(),
            ));
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
        let metaboards = Metaboard::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(metaboards.len(), 2);
        assert!(metaboards.contains_key("MetaboardOne"));
        assert!(metaboards.contains_key("MetaboardTwo"));

        assert_eq!(
            metaboards.get("MetaboardOne").unwrap().url,
            Url::parse("https://metaboard-one.com").unwrap()
        );
        assert_eq!(
            metaboards.get("MetaboardTwo").unwrap().url,
            Url::parse("https://metaboard-two.com").unwrap()
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
        let error = Metaboard::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateMetaboard".to_string())
        );
    }
}
