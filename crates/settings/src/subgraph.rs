use crate::yaml::{
    context::Context, default_document, require_hash, require_string, YamlError, YamlParsableHash,
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

impl YamlParsableHash for Subgraph {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<Context>,
    ) -> Result<HashMap<String, Subgraph>, YamlError> {
        let mut subgraphs = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(subgraphs_hash) = require_hash(&document_read, Some("subgraphs"), None) {
                for (key_yaml, subgraph_yaml) in subgraphs_hash {
                    let subgraph_key = key_yaml.as_str().unwrap_or_default().to_string();

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

                    if subgraphs.contains_key(&subgraph_key) {
                        return Err(YamlError::KeyShadowing(subgraph_key));
                    }
                    subgraphs.insert(subgraph_key, subgraph);
                }
            }
        }

        if subgraphs.is_empty() {
            return Err(YamlError::ParseError(
                "missing field: subgraphs".to_string(),
            ));
        }

        Ok(subgraphs)
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
        let error = Subgraph::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: subgraphs".to_string())
        );

        let yaml = r#"
subgraphs:
    TestSubgraph:
        test: https://subgraph.com
"#;
        let error = Subgraph::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "subgraph value must be a string for key: TestSubgraph".to_string()
            )
        );

        let yaml = r#"
subgraphs:
    TestSubgraph:
        - https://subgraph.com
"#;
        let error = Subgraph::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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
        let result = Subgraph::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("TestSubgraph"));
    }

    #[test]
    fn test_parse_subgraphs_from_yaml_multiple_files() {
        let yaml_one = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/mainnet
    testnet: https://api.thegraph.com/subgraphs/name/testnet
"#;
        let yaml_two = r#"
subgraphs:
    subgraph-one: https://api.thegraph.com/subgraphs/name/one
    subgraph-two: https://api.thegraph.com/subgraphs/name/two
"#;
        let subgraphs = Subgraph::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(subgraphs.len(), 4);
        assert_eq!(
            subgraphs.get("mainnet").unwrap().url,
            Url::parse("https://api.thegraph.com/subgraphs/name/mainnet").unwrap()
        );
        assert_eq!(
            subgraphs.get("testnet").unwrap().url,
            Url::parse("https://api.thegraph.com/subgraphs/name/testnet").unwrap()
        );
        assert_eq!(
            subgraphs.get("subgraph-one").unwrap().url,
            Url::parse("https://api.thegraph.com/subgraphs/name/one").unwrap()
        );
        assert_eq!(
            subgraphs.get("subgraph-two").unwrap().url,
            Url::parse("https://api.thegraph.com/subgraphs/name/two").unwrap()
        );
    }

    #[test]
    fn test_parse_subgraphs_from_yaml_duplicate_key() {
        let yaml_one = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/mainnet
    testnet: https://api.thegraph.com/subgraphs/name/testnet
"#;
        let yaml_two = r#"
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/mainnet
"#;
        let error = Subgraph::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(error, YamlError::KeyShadowing("mainnet".to_string()));
    }
}
