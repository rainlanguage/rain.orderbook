use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkYaml {
    pub rpc: String,
    pub chain_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}
impl NetworkYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, NetworkYaml>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut networks = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("networks"),
            Some("missing field networks".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let network = NetworkYaml {
                rpc: require_string(
                    value,
                    Some("rpc"),
                    Some(format!("rpc missing for network: {:?}", key)),
                )?,
                chain_id: require_string(
                    value,
                    Some("chain-id"),
                    Some(format!("chain-id missing for network: {:?}", key)),
                )?,
                label: optional_string(value, "label"),
                network_id: optional_string(value, "network-id"),
                currency: optional_string(value, "currency"),
            };
            networks.insert(key.to_string(), network);
        }
        Ok(networks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_errors() {
        let yaml = r#"
test: test
"#;
        let error = NetworkYaml::try_from_string(&yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = NetworkYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc missing for network: \"mainnet\"".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = NetworkYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("chain-id missing for network: \"mainnet\"".to_string())
        );
    }
}
