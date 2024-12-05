use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DeployerYaml {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
impl DeployerYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut deployers = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("deployers"),
            Some("missing field deployers".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let deployer = DeployerYaml {
                address: require_string(
                    value,
                    Some("address"),
                    Some(format!("address missing for deployer: {:?}", key)),
                )?,
                network: optional_string(value, "network"),
                label: optional_string(value, "label"),
            };
            deployers.insert(key.to_string(), deployer);
        }
        Ok(deployers)
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
metaboards:
    board1: https://meta.example.com/board1
orderbooks:
    book1:
        address: "0x1234"
tokens:
    weth:
        network: "mainnet"
        address: "0x5678"
"#;

    #[test]
    fn test_deployers_errors() {
        let error = DeployerYaml::try_from_string(VALID_YAML).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field deployers".to_string())
        );

        let yaml = r#"
deployers:
    main:
        network: "mainnet"
"#;
        let error = DeployerYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address missing for deployer: \"main\"".to_string())
        );
    }
}
