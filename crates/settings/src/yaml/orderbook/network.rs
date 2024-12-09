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
            Some("missing field: networks".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let network = NetworkYaml {
                rpc: require_string(
                    value,
                    Some("rpc"),
                    Some(format!("rpc string missing in network: {}", key)),
                )?,
                chain_id: require_string(
                    value,
                    Some("chain-id"),
                    Some(format!("chain-id string missing in network: {}", key)),
                )?,
                label: optional_string(value, "label"),
                network_id: optional_string(value, "network-id"),
                currency: optional_string(value, "currency"),
            };
            networks.insert(key.to_string(), network);
        }
        Ok(networks)
    }

    pub fn get_network_keys(source: &str) -> Result<Vec<String>, YamlError> {
        let networks = NetworkYaml::try_from_string(source)?;
        Ok(networks.keys().cloned().collect())
    }

    pub fn get_network(source: &str, key: &str) -> Result<NetworkYaml, YamlError> {
        let networks = NetworkYaml::try_from_string(source)?;
        let network = networks
            .get(key)
            .ok_or(YamlError::KeyNotFound(key.to_string()))?
            .clone();
        Ok(network)
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
            YamlError::ParseError("missing field: networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = NetworkYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc string missing in network: mainnet".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = NetworkYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("chain-id string missing in network: mainnet".to_string())
        );
    }
}
