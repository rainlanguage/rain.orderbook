use crate::config::Subgraph;
use crate::yaml::{require_hash, require_string, YamlError, YamlParsableHash};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use url::{ParseError, Url};

#[derive(Clone, Debug)]
pub struct YamlSubgraph(Subgraph);

impl YamlSubgraph {
    pub fn validate_url(value: &str) -> Result<Url, ParseError> {
        Url::parse(value)
    }
}

impl YamlParsableHash for YamlSubgraph {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, YamlSubgraph>, YamlError> {
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

                let url = YamlSubgraph::validate_url(&require_string(
                    subgraph_yaml,
                    None,
                    Some(format!(
                        "subgraph value must be a string for key: {subgraph_key}"
                    )),
                )?)?;

                Ok((subgraph_key, YamlSubgraph(url)))
            })
            .collect()
    }
}

impl From<Subgraph> for YamlSubgraph {
    fn from(value: Subgraph) -> Self {
        YamlSubgraph(value)
    }
}

impl From<YamlSubgraph> for Subgraph {
    fn from(value: YamlSubgraph) -> Self {
        value.0
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
        let error = YamlSubgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: subgraphs".to_string())
        );

        let yaml = r#"
subgraphs:
    TestSubgraph:
        test: https://subgraph.com
"#;
        let error = YamlSubgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
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
        let error = YamlSubgraph::parse_all_from_yaml(get_document(yaml)).unwrap_err();
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
        let result = YamlSubgraph::parse_all_from_yaml(get_document(yaml)).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("TestSubgraph"));
    }
}
