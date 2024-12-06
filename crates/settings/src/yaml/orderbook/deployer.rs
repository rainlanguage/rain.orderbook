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
            Some("missing field: deployers".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let deployer = DeployerYaml {
                address: require_string(
                    value,
                    Some("address"),
                    Some(format!("address string missing in deployer: {}", key)),
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

    #[test]
    fn test_deployers_errors() {
        let yaml = r#"
test: test
"#;
        let error = DeployerYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: deployers".to_string())
        );

        let yaml = r#"
deployers:
    main:
        network: "mainnet"
"#;
        let error = DeployerYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in deployer: main".to_string())
        );
    }
}
