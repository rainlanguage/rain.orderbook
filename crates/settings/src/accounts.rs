use crate::yaml::{
    context::Context, default_document, optional_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use alloy::{hex::FromHexError, primitives::Address};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct AccountCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(AccountCfg);

impl AccountCfg {
    pub fn validate_address(value: &str) -> Result<Address, ParseAccountCfgError> {
        Address::from_str(value).map_err(ParseAccountCfgError::AddressParseError)
    }
}

impl YamlParsableHash for AccountCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, AccountCfg>, YamlError> {
        let mut accounts = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(accounts_hash) = optional_hash(&document_read, "accounts") {
                for (key_yaml, account_yaml) in accounts_hash {
                    let account_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = "accounts".to_string();

                    let address_str = require_string(account_yaml, None, Some(location.clone()))?;
                    let address = AccountCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: account_key.to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let account = AccountCfg {
                        document: document.clone(),
                        key: account_key.clone(),
                        address,
                    };
                    if accounts.contains_key(&account_key) {
                        return Err(YamlError::KeyShadowing(account_key, "accounts".to_string()));
                    }
                    accounts.insert(account_key, account);
                }
            }
        }

        Ok(accounts)
    }
}

impl Default for AccountCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
        }
    }
}

impl PartialEq for AccountCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.address == other.address
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseAccountCfgError {
    #[error("Failed to parse account address")]
    AddressParseError(FromHexError),
}

impl ParseAccountCfgError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseAccountCfgError::AddressParseError(err) =>
                format!("The account address in your YAML configuration is invalid. Please provide a valid EVM address: '{}'", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_accounts_invalid_address() {
        let error = AccountCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
accounts:
    name-one: invalid-address
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "name-one".to_string(),
                    reason: "Failed to parse account address".to_string(),
                },
                location: "accounts".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Invalid value for field 'name-one' in accounts: Failed to parse account address"
        );
    }

    #[test]
    fn test_parse_accounts_from_yaml_multiple_files() {
        let yaml_one = r#"
accounts:
    name-one: 0x0000000000000000000000000000000000000001
"#;
        let yaml_two = r#"
accounts:
    name-two: 0x0000000000000000000000000000000000000002
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let accounts = AccountCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains_key("name-one"));
        assert!(accounts.contains_key("name-two"));

        assert_eq!(
            accounts.get("name-one").unwrap().address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        assert_eq!(
            accounts.get("name-two").unwrap().address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
    }

    #[test]
    fn test_parse_accounts_from_yaml_duplicate_key() {
        let yaml_one = r#"
accounts:
    account: 0x0000000000000000000000000000000000000001
"#;
        let yaml_two = r#"
accounts:
    account: 0x0000000000000000000000000000000000000002
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = AccountCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("account".to_string(), "accounts".to_string())
        );
    }
}
