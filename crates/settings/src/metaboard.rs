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
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Metaboard>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let metaboards_hash = require_hash(
            &document_read,
            Some("metaboards"),
            Some("missing field: metaboards".to_string()),
        )?;

        metaboards_hash
            .iter()
            .map(|(key_yaml, metaboard_yaml)| {
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

                Ok((metaboard_key, metaboard))
            })
            .collect()
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
    fn test_parse_metaboards_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Metaboard::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: metaboards".to_string())
        );

        let yaml = r#"
metaboards:
    TestMetaboard:
        test: https://metaboard.com
"#;
        let error = Metaboard::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key: TestMetaboard".to_string()
            )
        );

        let yaml = r#"
metaboards:
    TestMetaboard:
        - https://metaboard.com
"#;
        let error = Metaboard::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key: TestMetaboard".to_string()
            )
        );

        let yaml = r#"
metaboards:
    TestMetaboard: invalid-url
"#;
        let res = Metaboard::parse_all_from_yaml(get_document(yaml));
        assert!(res.is_err());
    }
}
