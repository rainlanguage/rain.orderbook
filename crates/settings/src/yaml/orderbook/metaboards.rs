use super::*;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MetaboardsYaml {}
impl MetaboardsYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, String>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut metaboards = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("metaboards"),
            Some("missing field metaboards".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            metaboards.insert(
                key.to_string(),
                require_string(
                    value,
                    None,
                    Some(format!(
                        "metaboard value must be a string for key {:?}",
                        key
                    )),
                )?,
            );
        }
        Ok(metaboards)
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
subgraphs:
    main: https://api.thegraph.com/subgraphs/name/xyz
"#;

    #[test]
    fn test_metaboards_errors() {
        let error = MetaboardsYaml::try_from_string(VALID_YAML).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field metaboards".to_string())
        );

        let yaml = r#"
metaboards:
    board1:
        - one
"#;
        let error =
            MetaboardsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key \"board1\"".to_string()
            )
        );

        let yaml = r#"
metaboards:
    board1:
        - one: one
"#;
        let error =
            MetaboardsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "metaboard value must be a string for key \"board1\"".to_string()
            )
        );
    }
}
