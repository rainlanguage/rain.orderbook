use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderbookEntryYaml {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgraph: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
impl OrderbookEntryYaml {
    pub fn try_from_string(source: &str) -> Result<HashMap<String, Self>, YamlError> {
        let doc = &load_yaml(source)?;

        let mut orderbooks = HashMap::new();
        for (key, value) in require_hash(
            doc,
            Some("orderbooks"),
            Some("missing field orderbooks".to_string()),
        )? {
            let key = key.as_str().unwrap_or_default();
            let orderbook = OrderbookEntryYaml {
                address: require_string(
                    value,
                    Some("address"),
                    Some(format!("address missing for orderbook: {:?}", key)),
                )?,
                network: optional_string(value, "network"),
                subgraph: optional_string(value, "subgraph"),
                label: optional_string(value, "label"),
            };
            orderbooks.insert(key.to_string(), orderbook);
        }
        Ok(orderbooks)
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
"#;

    #[test]
    fn test_orderbooks_errors() {
        let error = OrderbookEntryYaml::try_from_string(VALID_YAML).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field orderbooks".to_string())
        );

        let yaml = r#"
orderbooks:
    book1:
        network: "mainnet"
"#;
        let error =
            OrderbookEntryYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("address missing for orderbook: \"book1\"".to_string())
        );
    }
}
