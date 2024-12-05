use super::*;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SubgraphsYaml {}
impl SubgraphsYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, String>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut subgraphs = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("subgraphs"),
            Some("missing field subgraphs".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            subgraphs.insert(
                key.to_string(),
                require_string(
                    value,
                    None,
                    Some(format!("subgraph value must be a string for key {:?}", key)),
                )?,
            );
        }
        Ok(subgraphs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_YAML: &str = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: "1"
"#;

    #[test]
    fn test_subgraphs_errors() {
        let error = SubgraphsYaml::try_from_string(VALID_YAML).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field subgraphs".to_string())
        );

        let yaml = r#"
subgraphs:
    main:
        - one
"#;
        let error = SubgraphsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("subgraph value must be a string for key \"main\"".to_string())
        );
        let yaml = r#"
subgraphs:
    main:
        - one: one
"#;
        let error = SubgraphsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("subgraph value must be a string for key \"main\"".to_string())
        );
    }
}
