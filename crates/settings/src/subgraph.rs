use crate::yaml::{
    default_document, optional_string, require_hash, require_string, YamlError,
    YamlParsableMergableHash,
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
pub struct Subgraph {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    pub url: Url,
}

impl Subgraph {
    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }
}

impl YamlParsableMergableHash for Subgraph {
    fn parse_and_merge_all_from_yamls(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut all_subgraphs = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if let Ok(subgraphs_hash) = require_hash(
                &document_read,
                Some("subgraphs"),
                None, // Don't error if not found
            ) {
                for (key_yaml, subgraph_yaml) in subgraphs_hash {
                    let subgraph_key = key_yaml.as_str().unwrap_or_default().to_string();

                    // Error on duplicates
                    if all_subgraphs.contains_key(&subgraph_key) {
                        return Err(YamlError::DuplicateKey(subgraph_key));
                    }

                    let url = Subgraph::validate_url(&require_string(
                        subgraph_yaml,
                        None,
                        Some(format!(
                            "subgraph value must be a string for key: {subgraph_key}"
                        )),
                    )?)?;

                    let subgraph = Subgraph {
                        document: document.clone(),
                        key: subgraph_key.clone(),
                        url,
                    };

                    all_subgraphs.insert(subgraph_key, subgraph);
                }
            }
        }

        if all_subgraphs.is_empty() {
            return Err(YamlError::ParseError(
                "missing field: subgraphs".to_string(),
            ));
        }

        Ok(all_subgraphs)
    }
}

impl Default for Subgraph {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            url: Url::parse("https://subgraph.com").unwrap(),
        }
    }
}

impl PartialEq for Subgraph {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_subgraphs_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Subgraph::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: subgraphs".to_string())
        );

        let yaml = r#"
subgraphs:
    TestSubgraph:
        test: https://subgraph.com
"#;
        let error = Subgraph::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "subgraph value must be a string for key: TestSubgraph".to_string()
            )
        );

        let yaml = r#"
subgraphs:
    TestSubgraph: https://subgraph.com
"#;
        let result = Subgraph::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("TestSubgraph"));
    }

    #[test]
    fn test_subgraph_document_preservation() {
        // Main document with one subgraph
        let main_yaml = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/main
"#;
        let main_doc = get_document(main_yaml);

        // Orderbook yaml with another subgraph
        let orderbook_yaml = r#"
subgraphs:
    testnet: https://api.thegraph.com/subgraphs/name/test
"#;
        let orderbook_doc = get_document(orderbook_yaml);

        // Parse both documents
        let subgraphs =
            Subgraph::parse_and_merge_all_from_yamls(vec![main_doc.clone(), orderbook_doc.clone()])
                .unwrap();

        // Verify subgraphs came from correct documents
        let mainnet = subgraphs.get("mainnet").unwrap();
        let testnet = subgraphs.get("testnet").unwrap();

        // Check document preservation by comparing Arc pointers
        assert!(Arc::ptr_eq(&mainnet.document, &main_doc));
        assert!(Arc::ptr_eq(&testnet.document, &orderbook_doc));
    }

    #[test]
    fn test_subgraph_duplicate_error() {
        let yaml1 = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/main
"#;
        let yaml2 = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/other
"#;

        let error = Subgraph::parse_and_merge_all_from_yamls(vec![
            get_document(yaml1),
            get_document(yaml2),
        ])
        .unwrap_err();

        assert_eq!(error, YamlError::DuplicateKey("mainnet".to_string()));
    }
}
