use crate::config::Subgraph;
use crate::yaml::{require_hash, require_string, YamlError, YamlParsableHash};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use url::Url;

impl YamlParsableHash for Subgraph {
    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Url>, YamlError> {
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

                let url = Url::parse(&require_string(
                    subgraph_yaml,
                    None,
                    Some(format!(
                        "subgraph value must be a string for key: {subgraph_key}"
                    )),
                )?)?;

                Ok((subgraph_key, url))
            })
            .collect()
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
    }
}
