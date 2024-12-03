use super::{optional_hash, optional_string, require_hash, require_string, YamlError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strict_yaml_rust::StrictYamlLoader;

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
    pub fn from_str(yaml: &str) -> Result<Self, YamlError> {
        let docs = StrictYamlLoader::load_from_str(yaml)?;

        if docs.is_empty() {
            return Err(YamlError::EmptyFile);
        }

        let doc = &docs[0];
        let mut yaml = Self::default();

        for (key, value) in require_hash(doc, "networks", Some(format!("missing field networks")))?
        {
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
            yaml.networks.insert(key.to_string(), network);
        }

        for (key, value) in
            require_hash(doc, "subgraphs", Some(format!("missing field subgraphs")))?
        {
            let key = key.as_str().unwrap_or_default();
            yaml.subgraphs.insert(
                key.to_string(),
                require_string(
                    value,
                    None,
                    Some(format!("subgraph value must be a string for key {:?}", key)),
                )?,
            );
        }

        for (key, value) in
            require_hash(doc, "metaboards", Some(format!("missing field metaboards")))?
        {
            let key = key.as_str().unwrap_or_default();
            yaml.metaboards.insert(
                key.to_string(),
                require_string(
                    value,
                    None,
                    Some(format!(
                        "metaboard value must be a string for key {:?}",
                        key
                    )),
                )?,
            );
        }

        for (key, value) in
            require_hash(doc, "orderbooks", Some(format!("missing field orderbooks")))?
        {
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
            yaml.orderbooks.insert(key.to_string(), orderbook);
        }

        for (key, value) in require_hash(doc, "tokens", Some(format!("missing field tokens")))? {
            let key = key.as_str().unwrap_or_default();
            let token = TokenYaml {
                network: require_string(
                    value,
                    Some("network"),
                    Some(format!("network missing for token: {:?}", key)),
                )?,
                address: require_string(
                    value,
                    Some("address"),
                    Some(format!("address missing for token: {:?}", key)),
                )?,
                decimals: optional_string(value, "decimals"),
                label: optional_string(value, "label"),
                symbol: optional_string(value, "symbol"),
            };
            yaml.tokens.insert(key.to_string(), token);
        }

        for (key, value) in
            require_hash(doc, "deployers", Some(format!("missing field deployers")))?
        {
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
            yaml.deployers.insert(key.to_string(), deployer);
        }

        if let Some(accounts) = optional_hash(doc, "accounts") {
            let mut accounts_map = HashMap::new();
            for (key, value) in accounts {
                let key = key.as_str().unwrap_or_default();
                accounts_map.insert(
                    key.to_string(),
                    require_string(
                        value,
                        None,
                        Some(format!("account value must be a string for key {:?}", key)),
                    )?,
                );
            }
            yaml.accounts = Some(accounts_map);
        }

        if let Some(sentry) = optional_string(doc, "sentry") {
            yaml.sentry = Some(sentry);
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
