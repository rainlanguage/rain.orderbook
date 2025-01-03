use crate::yaml::{require_hash, require_string, YamlError, YamlParsableHash};
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
#[serde(default)]
pub struct Subgraph {
    #[serde(skip)]
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
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Subgraph>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let subgraphs_hash = require_hash(
            &document_read,
            Some("subgraphs"),
            Some("missing field: subgraphs".to_string()),
        )?;

        subgraphs_hash
            .iter()
            .map(|(key_yaml, subgraph_yaml)| {
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

                Ok((subgraph_key, subgraph))
            })
            .collect()
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
        let error = Subgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: subgraphs".to_string())
        );

        let yaml = r#"
subgraphs:
    TestSubgraph:
        test: https://subgraph.com
"#;
        let error = Subgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
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
        let error = Subgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
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
        let result = Subgraph::parse_all_from_yaml(get_document(yaml)).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("TestSubgraph"));
    }
}
