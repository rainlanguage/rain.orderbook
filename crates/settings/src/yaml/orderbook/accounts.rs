use super::*;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccountsYaml {}
impl AccountsYaml {
    pub fn try_from_string(source: &str) -> Result<Option<HashMap<String, String>>, YamlError> {
        let doc = &load_yaml(source)?;

        if let Some(accounts_map) = optional_hash(doc, "accounts") {
            let mut accounts = HashMap::new();
            for (key, value) in accounts_map {
                let key = key.as_str().unwrap_or_default();
                accounts.insert(
                    key.to_string(),
                    require_string(
                        value,
                        None,
                        Some(format!("account value must be a string for key: {}", key)),
                    )?,
                );
            }
            Ok(Some(accounts))
        } else {
            Ok(None)
        }
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
deployers:
    main:
        address: "0x9012"
"#;

    #[test]
    fn test_accounts_errors() {
        let accounts = AccountsYaml::try_from_string(VALID_YAML).unwrap();
        assert_eq!(accounts, None);

        let yaml = r#"
accounts:
    admin:
        - one
"#;
        let error = AccountsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("account value must be a string for key: admin".to_string())
        );

        let yaml = r#"
accounts:
    admin:
        - one: one
"#;
        let error = AccountsYaml::try_from_string(&format!("{}{}", VALID_YAML, yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("account value must be a string for key: admin".to_string())
        );
    }
}
