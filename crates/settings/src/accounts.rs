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
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
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

    fn to_yaml_value(&self) -> Result<StrictYaml, YamlError> {
        Ok(StrictYaml::String(alloy::hex::encode_prefixed(
            self.address,
        )))
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let accounts_key = StrictYaml::String("accounts".to_string());
            let Some(accounts_value) = root_hash.get(&accounts_key) else {
                continue;
            };
            let StrictYaml::Hash(ref accounts_hash) = accounts_value.clone() else {
                continue;
            };

            let mut sanitized_accounts: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in accounts_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                if value.as_str().is_none() {
                    continue;
                }

                sanitized_accounts.push((key_str.to_string(), value.clone()));
            }

            sanitized_accounts.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_accounts_hash = Hash::new();
            for (key, value) in sanitized_accounts {
                new_accounts_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(accounts_key, StrictYaml::Hash(new_accounts_hash));
        }

        Ok(())
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

    #[test]
    fn test_to_yaml_hash_serializes_multiple_accounts() {
        let addr_one = Address::from_str("0x00000000000000000000000000000000000000aa").unwrap();
        let addr_two = Address::from_str("0x00000000000000000000000000000000000000bb").unwrap();

        let mut accounts = HashMap::new();
        accounts.insert(
            "admin".to_string(),
            AccountCfg {
                document: default_document(),
                key: "admin".to_string(),
                address: addr_one,
            },
        );
        accounts.insert(
            "operator".to_string(),
            AccountCfg {
                document: default_document(),
                key: "operator".to_string(),
                address: addr_two,
            },
        );

        let yaml = AccountCfg::to_yaml_hash(&accounts).unwrap();
        let StrictYaml::Hash(hash) = yaml else {
            panic!("expected hash for accounts yaml");
        };
        assert_eq!(
            hash.get(&StrictYaml::String("admin".to_string())),
            Some(&StrictYaml::String(
                "0x00000000000000000000000000000000000000aa".to_string()
            ))
        );
        assert_eq!(
            hash.get(&StrictYaml::String("operator".to_string())),
            Some(&StrictYaml::String(
                "0x00000000000000000000000000000000000000bb".to_string()
            ))
        );
    }

    #[test]
    fn test_sanitize_documents_drops_non_string_values() {
        let yaml = r#"
accounts:
    valid-account: 0x0000000000000000000000000000000000000001
    invalid-account:
        nested: value
"#;
        let document = get_document(yaml);
        AccountCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let accounts = root
            .get(&StrictYaml::String("accounts".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref accounts_hash) = *accounts else {
            panic!("expected accounts hash");
        };

        assert!(accounts_hash.contains_key(&StrictYaml::String("valid-account".to_string())));
        assert!(!accounts_hash.contains_key(&StrictYaml::String("invalid-account".to_string())));
        assert_eq!(accounts_hash.len(), 1);
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
accounts:
    zebra: 0x0000000000000000000000000000000000000003
    alpha: 0x0000000000000000000000000000000000000001
    beta: 0x0000000000000000000000000000000000000002
"#;
        let document = get_document(yaml);
        AccountCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let accounts = root
            .get(&StrictYaml::String("accounts".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref accounts_hash) = *accounts else {
            panic!("expected accounts hash");
        };

        let keys: Vec<String> = accounts_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_accounts_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        AccountCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("accounts".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        AccountCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_accounts() {
        let yaml = r#"
accounts: not-a-hash
"#;
        let document = get_document(yaml);
        AccountCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let accounts = root
            .get(&StrictYaml::String("accounts".to_string()))
            .unwrap();
        assert_eq!(accounts.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
accounts:
    admin: 0x0000000000000000000000000000000000000001
"#;
        let yaml_two = r#"
accounts:
    operator: 0x0000000000000000000000000000000000000002
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        AccountCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let accounts = root
                .get(&StrictYaml::String("accounts".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref accounts_hash) = *accounts else {
                panic!("expected accounts hash");
            };

            let keys: Vec<String> = accounts_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["admin"]);
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let accounts = root
                .get(&StrictYaml::String("accounts".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref accounts_hash) = *accounts else {
                panic!("expected accounts hash");
            };

            let keys: Vec<String> = accounts_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["operator"]);
        }
    }
}
