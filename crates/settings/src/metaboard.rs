use crate::yaml::{
    default_document, require_hash, require_string, YamlError, YamlParsableMergableHash,
};
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

impl YamlParsableMergableHash for Metaboard {
    fn parse_and_merge_all_from_yamls(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut all_metaboards = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if let Ok(metaboards_hash) = require_hash(
                &document_read,
                Some("metaboards"),
                None, // Don't error if not found
            ) {
                for (key_yaml, metaboard_yaml) in metaboards_hash {
                    let metaboard_key = key_yaml.as_str().unwrap_or_default().to_string();

                    // Error on duplicates
                    if all_metaboards.contains_key(&metaboard_key) {
                        return Err(YamlError::DuplicateKey(metaboard_key));
                    }

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

                    all_metaboards.insert(metaboard_key, metaboard);
                }
            }
        }

        if all_metaboards.is_empty() {
            return Err(YamlError::ParseError(
                "missing field: metaboards".to_string(),
            ));
        }

        Ok(all_metaboards)
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
        let error =
            Metaboard::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: metaboards".to_string())
        );

        let yaml = r#"
metaboards:
    TestMetaboard:
        test: https://metaboard.com
"#;
        let error =
            Metaboard::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key: TestMetaboard".to_string()
            )
        );

        let yaml = r#"
metaboards:
    TestMetaboard: https://metaboard.com
"#;
        let result = Metaboard::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("TestMetaboard"));
    }

    #[test]
    fn test_metaboard_document_preservation() {
        // Main document with one metaboard
        let main_yaml = r#"
metaboards:
    mainnet: https://api.metaboard.com/main
"#;
        let main_doc = get_document(main_yaml);

        // Orderbook yaml with another metaboard
        let orderbook_yaml = r#"
metaboards:
    testnet: https://api.metaboard.com/test
"#;
        let orderbook_doc = get_document(orderbook_yaml);

        // Parse both documents
        let metaboards = Metaboard::parse_and_merge_all_from_yamls(vec![
            main_doc.clone(),
            orderbook_doc.clone(),
        ])
        .unwrap();

        // Verify metaboards came from correct documents
        let mainnet = metaboards.get("mainnet").unwrap();
        let testnet = metaboards.get("testnet").unwrap();

        // Check document preservation by comparing Arc pointers
        assert!(Arc::ptr_eq(&mainnet.document, &main_doc));
        assert!(Arc::ptr_eq(&testnet.document, &orderbook_doc));
    }

    #[test]
    fn test_metaboard_duplicate_error() {
        let yaml1 = r#"
metaboards:
    mainnet: https://api.metaboard.com/main
"#;
        let yaml2 = r#"
metaboards:
    mainnet: https://api.metaboard.com/other
"#;

        let error = Metaboard::parse_and_merge_all_from_yamls(vec![
            get_document(yaml1),
            get_document(yaml2),
        ])
        .unwrap_err();

        assert_eq!(error, YamlError::DuplicateKey("mainnet".to_string()));
    }
}
