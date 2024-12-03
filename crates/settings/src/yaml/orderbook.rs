use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strict_yaml_rust::{ScanError, StrictYamlLoader};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OrderbookYamlError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
    #[error("Required field missing: {0}")]
    MissingField(String),
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
    #[error("Empty yaml file")]
    EmptyFile,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderbookYaml {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub networks: HashMap<String, NetworkYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub subgraphs: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metaboards: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub orderbooks: HashMap<String, OrderbookEntryYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tokens: HashMap<String, TokenYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub deployers: HashMap<String, DeployerYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentry: Option<String>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DeployerYaml {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl OrderbookYaml {
    pub fn from_str(yaml: &str) -> Result<Self, OrderbookYamlError> {
        let docs = StrictYamlLoader::load_from_str(yaml)?;

        if docs.is_empty() {
            return Err(OrderbookYamlError::EmptyFile);
        }

        let doc = &docs[0];
        let mut yaml = Self::default();

        match doc["networks"].as_hash() {
            Some(networks) => {
                for (key, value) in networks {
                    let key_str = key.as_str().unwrap_or_default();
                    let rpc = value["rpc"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "rpc missing for network {:?}",
                            key_str
                        ))
                    })?;

                    let chain_id = value["chain-id"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "chain-id missing for network {:?}",
                            key_str
                        ))
                    })?;

                    let network = NetworkYaml {
                        rpc: rpc.to_string(),
                        chain_id: chain_id.to_string(),
                        label: value["label"].as_str().map(|s| s.to_string()),
                        network_id: value["network-id"].as_str().map(|s| s.to_string()),
                        currency: value["currency"].as_str().map(|s| s.to_string()),
                    };
                    yaml.networks.insert(key_str.to_string(), network);
                }
            }
            None => return Err(OrderbookYamlError::MissingField("networks".to_string())),
        }

        match doc["subgraphs"].as_hash() {
            Some(subgraphs) => {
                for (key, value) in subgraphs {
                    let key_str = key.as_str().unwrap_or_default();
                    let value_str = value.as_str().ok_or_else(|| {
                        OrderbookYamlError::InvalidFieldType(format!(
                            "subgraph value must be a string for key {:?}",
                            key_str
                        ))
                    })?;
                    yaml.subgraphs
                        .insert(key_str.to_string(), value_str.to_string());
                }
            }
            None => return Err(OrderbookYamlError::MissingField("subgraphs".to_string())),
        }

        match doc["metaboards"].as_hash() {
            Some(metaboards) => {
                for (key, value) in metaboards {
                    let key_str = key.as_str().unwrap_or_default();
                    let value_str = value.as_str().ok_or_else(|| {
                        OrderbookYamlError::InvalidFieldType(format!(
                            "metaboard value must be a string for key {:?}",
                            key_str
                        ))
                    })?;
                    yaml.metaboards
                        .insert(key_str.to_string(), value_str.to_string());
                }
            }
            None => return Err(OrderbookYamlError::MissingField("metaboards".to_string())),
        }

        match doc["orderbooks"].as_hash() {
            Some(orderbooks) => {
                for (key, value) in orderbooks {
                    let key_str = key.as_str().unwrap_or_default();
                    let address = value["address"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "address missing for orderbook {:?}",
                            key_str
                        ))
                    })?;

                    let orderbook = OrderbookEntryYaml {
                        address: address.to_string(),
                        network: value["network"].as_str().map(|s| s.to_string()),
                        subgraph: value["subgraph"].as_str().map(|s| s.to_string()),
                        label: value["label"].as_str().map(|s| s.to_string()),
                    };
                    yaml.orderbooks.insert(key_str.to_string(), orderbook);
                }
            }
            None => return Err(OrderbookYamlError::MissingField("orderbooks".to_string())),
        }

        match doc["tokens"].as_hash() {
            Some(tokens) => {
                for (key, value) in tokens {
                    let key_str = key.as_str().unwrap_or_default();
                    let network = value["network"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "network missing for token {:?}",
                            key_str
                        ))
                    })?;

                    let address = value["address"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "address missing for token {:?}",
                            key_str
                        ))
                    })?;

                    let token = TokenYaml {
                        network: network.to_string(),
                        address: address.to_string(),
                        decimals: value["decimals"].as_str().map(|s| s.to_string()),
                        label: value["label"].as_str().map(|s| s.to_string()),
                        symbol: value["symbol"].as_str().map(|s| s.to_string()),
                    };
                    yaml.tokens.insert(key_str.to_string(), token);
                }
            }
            None => return Err(OrderbookYamlError::MissingField("tokens".to_string())),
        }

        match doc["deployers"].as_hash() {
            Some(deployers) => {
                for (key, value) in deployers {
                    let key_str = key.as_str().unwrap_or_default();
                    let address = value["address"].as_str().ok_or_else(|| {
                        OrderbookYamlError::MissingField(format!(
                            "address missing for deployer {:?}",
                            key_str
                        ))
                    })?;

                    let deployer = DeployerYaml {
                        address: address.to_string(),
                        network: value["network"].as_str().map(|s| s.to_string()),
                        label: value["label"].as_str().map(|s| s.to_string()),
                    };
                    yaml.deployers.insert(key_str.to_string(), deployer);
                }
            }
            None => return Err(OrderbookYamlError::MissingField("deployers".to_string())),
        }

        if doc["accounts"].as_str().is_some() || doc["accounts"].as_vec().is_some() {
            return Err(OrderbookYamlError::InvalidFieldType(
                "accounts must be a map".to_string(),
            ));
        }
        if let Some(accounts) = doc["accounts"].as_hash() {
            let mut accounts_map = HashMap::new();
            for (key, value) in accounts {
                let key_str = key.as_str().unwrap_or_default();
                let value_str = value.as_str().ok_or_else(|| {
                    OrderbookYamlError::InvalidFieldType(format!(
                        "account value must be a string for key {:?}",
                        key_str
                    ))
                })?;
                accounts_map.insert(key_str.to_string(), value_str.to_string());
            }
            yaml.accounts = Some(accounts_map);
        }

        if doc["sentry"].as_hash().is_some() || doc["sentry"].as_vec().is_some() {
            return Err(OrderbookYamlError::InvalidFieldType(
                "sentry must be a string".to_string(),
            ));
        }
        if let Some(sentry) = doc["sentry"].as_str() {
            yaml.sentry = Some(sentry.to_string());
        }

        Ok(yaml)
    }

    pub fn set_network(&mut self, key: String, network: NetworkYaml) {
        self.networks.insert(key, network);
    }

    pub fn set_subgraph(&mut self, key: String, url: String) {
        self.subgraphs.insert(key, url);
    }

    pub fn set_metaboard(&mut self, key: String, url: String) {
        self.metaboards.insert(key, url);
    }

    pub fn set_orderbook(&mut self, key: String, orderbook: OrderbookEntryYaml) {
        self.orderbooks.insert(key, orderbook);
    }

    pub fn set_token(&mut self, key: String, token: TokenYaml) {
        self.tokens.insert(key, token);
    }

    pub fn set_deployer(&mut self, key: String, deployer: DeployerYaml) {
        self.deployers.insert(key, deployer);
    }

    pub fn set_accounts(&mut self, accounts: HashMap<String, String>) {
        self.accounts = Some(accounts);
    }

    pub fn set_sentry(&mut self, enabled: String) {
        self.sentry = Some(enabled);
    }
}
