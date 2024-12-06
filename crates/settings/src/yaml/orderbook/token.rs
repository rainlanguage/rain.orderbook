use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TokenYaml {
    pub network: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl TokenYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut tokens = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("tokens"),
            Some("missing field: tokens".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let token = TokenYaml {
                network: require_string(
                    value,
                    Some("network"),
                    Some(format!("network string missing in token: {}", key)),
                )?,
                address: require_string(
                    value,
                    Some("address"),
                    Some(format!("address string missing in token: {}", key)),
                )?,
                decimals: optional_string(value, "decimals"),
                label: optional_string(value, "label"),
                symbol: optional_string(value, "symbol"),
            };
            tokens.insert(key.to_string(), token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_errors() {
        let yaml = r#"
test: test
"#;
        let error = TokenYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: tokens".to_string())
        );

        let yaml = r#"
tokens:
    weth:
        address: "0x5678"
"#;
        let error = TokenYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("network string missing in token: weth".to_string())
        );

        let yaml = r#"
tokens:
    weth:
        network: "mainnet"
"#;
        let error = TokenYaml::try_from_string(yaml).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address string missing in token: weth".to_string())
        );
    }
}
