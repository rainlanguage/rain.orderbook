use crate::config_source::*;
use crate::yaml::{optional_string, require_hash, require_string, YamlError, YamlParsableHash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::{
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;
use url::{ParseError, Url};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Network {
    #[serde(skip)]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub rpc: Url,
    #[typeshare(typescript(type = "number"))]
    pub chain_id: u64,
    pub label: Option<String>,
    #[typeshare(typescript(type = "number"))]
    pub network_id: Option<u64>,
    pub currency: Option<String>,
}
impl Network {
    pub fn dummy() -> Self {
        Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            rpc: Url::parse("http://rpc.com").unwrap(),
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
        }
    }

    pub fn update_rpc(&mut self, rpc: &str) -> Result<(), YamlError> {
        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut networks)) =
                document_hash.get_mut(&StrictYaml::String("networks".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut network)) =
                    networks.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    network[&StrictYaml::String("rpc".to_string())] =
                        StrictYaml::String(rpc.to_string());
                    self.rpc =
                        Url::parse(rpc).map_err(ParseNetworkConfigSourceError::RpcParseError)?;
                } else {
                    return Err(YamlError::ParseError(format!(
                        "missing field: {} in networks",
                        self.key
                    )));
                }
            } else {
                return Err(YamlError::ParseError("missing field: networks".to_string()));
            }
        } else {
            return Err(YamlError::ParseError("document parse error".to_string()));
        }

        Ok(())
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Network);

impl YamlParsableHash for Network {
    type Item = Network;

    fn parse_all_from_yaml(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<HashMap<String, Network>, YamlError> {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let networks_hash = require_hash(
            &document_read,
            Some("networks"),
            Some("missing field: networks".to_string()),
        )?;

        networks_hash
            .into_iter()
            .map(|(key_yaml, network_yaml)| {
                let network_key = key_yaml.as_str().unwrap_or_default().to_string();

                let rpc_url = Url::from_str(&require_string(
                    network_yaml,
                    Some("rpc"),
                    Some(format!("rpc string missing in network: {network_key}")),
                )?)
                .map_err(|e| ParseNetworkConfigSourceError::RpcParseError(e))?;

                let chain_id = require_string(
                    network_yaml,
                    Some("chain-id"),
                    Some(format!(
                        "chain-id number as string missing in network: {network_key}"
                    )),
                )?
                .parse::<u64>()
                .map_err(ParseNetworkConfigSourceError::ChainIdParseError)?;

                let label = optional_string(network_yaml, "label");

                let network_id = optional_string(network_yaml, "network-id")
                    .map(|id| id.parse::<u64>())
                    .transpose()
                    .map_err(ParseNetworkConfigSourceError::NetworkIdParseError)?;

                let currency = optional_string(network_yaml, "currency");

                let network = Network {
                    document: document.clone(),
                    key: network_key.clone(),
                    rpc: rpc_url,
                    chain_id,
                    label,
                    network_id,
                    currency,
                };

                Ok((network_key, network))
            })
            .collect()
    }
}

impl Default for Network {
    fn default() -> Self {
        Network::dummy()
    }
}
impl PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.rpc == other.rpc
            && self.chain_id == other.chain_id
            && self.label == other.label
            && self.network_id == other.network_id
            && self.currency == other.currency
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkConfigSourceError {
    #[error("Failed to parse rpc: {}", 0)]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {}", 0)]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {}", 0)]
    NetworkIdParseError(ParseIntError),
}

impl NetworkConfigSource {
    pub fn try_into_network(self, key: String) -> Result<Network, ParseNetworkConfigSourceError> {
        Ok(Network {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key,
            rpc: self.rpc,
            chain_id: self.chain_id,
            label: self.label,
            network_id: self.network_id,
            currency: self.currency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strict_yaml_rust::StrictYamlLoader;
    use url::Url;

    #[test]
    fn test_try_from_network_string_success() {
        let network_string = NetworkConfigSource {
            rpc: Url::parse("http://127.0.0.1:8545").unwrap(),
            chain_id: 1,
            network_id: Some(1),
            label: Some("Local Testnet".into()),
            currency: Some("ETH".into()),
        };

        let result = network_string.try_into_network("local".into());
        assert!(result.is_ok());
        let network = result.unwrap();

        assert_eq!(network.rpc, Url::parse("http://127.0.0.1:8545").unwrap());
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.label, Some("Local Testnet".into()));
        assert_eq!(network.currency, Some("ETH".into()));
        assert_eq!(network.key, "local");
    }

    fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }

    #[test]
    fn test_parse_networks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Network::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = Network::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc string missing in network: mainnet".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = Network::parse_all_from_yaml(get_document(yaml)).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "chain-id number as string missing in network: mainnet".to_string()
            )
        );
    }
}
