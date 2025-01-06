use crate::config_source::*;
use crate::yaml::{
    default_document, optional_string, require_hash, require_string, YamlError,
    YamlParsableMergableHash,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct Network {
    #[serde(skip, default = "default_document")]
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

    pub fn validate_rpc(value: &str) -> Result<Url, ParseNetworkConfigSourceError> {
        Url::parse(value).map_err(ParseNetworkConfigSourceError::RpcParseError)
    }
    pub fn validate_chain_id(value: &str) -> Result<u64, ParseNetworkConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseNetworkConfigSourceError::ChainIdParseError)
    }
    pub fn validate_network_id(value: &str) -> Result<u64, ParseNetworkConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseNetworkConfigSourceError::NetworkIdParseError)
    }

    pub fn update_rpc(&mut self, rpc: &str) -> Result<Self, YamlError> {
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
                    self.rpc = Network::validate_rpc(rpc)?;
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

        Ok(self.clone())
    }
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Network);

impl YamlParsableMergableHash for Network {
    fn parse_and_merge_all_from_yamls(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut all_networks = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if let Ok(networks_hash) = require_hash(
                &document_read,
                Some("networks"),
                None, // Don't error if not found
            ) {
                for (key_yaml, network_yaml) in networks_hash {
                    let network_key = key_yaml.as_str().unwrap_or_default().to_string();

                    // Error on duplicates
                    if all_networks.contains_key(&network_key) {
                        return Err(YamlError::DuplicateKey(network_key));
                    }

                    let rpc_url = Network::validate_rpc(&require_string(
                        network_yaml,
                        Some("rpc"),
                        Some(format!("rpc string missing in network: {network_key}")),
                    )?)?;

                    let chain_id = Network::validate_chain_id(&require_string(
                        network_yaml,
                        Some("chain-id"),
                        Some(format!(
                            "chain-id number as string missing in network: {network_key}"
                        )),
                    )?)?;

                    let label = optional_string(network_yaml, "label");
                    let network_id = optional_string(network_yaml, "network-id")
                        .map(|id| Network::validate_network_id(&id))
                        .transpose()?;
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

                    all_networks.insert(network_key, network);
                }
            }
        }

        if all_networks.is_empty() {
            return Err(YamlError::ParseError("missing field: networks".to_string()));
        }

        Ok(all_networks)
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
    use crate::yaml::tests::{get_document, get_yaml_string};
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

    #[test]
    fn test_parse_networks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Network::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: networks".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = Network::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("rpc string missing in network: mainnet".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
"#;
        let error = Network::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "chain-id number as string missing in network: mainnet".to_string()
            )
        );
    }

    #[test]
    fn test_network_document_preservation() {
        // Main document with one network
        let main_yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
"#;
        let main_doc = get_document(main_yaml);

        // Orderbook yaml with another network
        let orderbook_yaml = r#"
networks:
    testnet:
        rpc: https://testnet.infura.io
        chain-id: 5
"#;
        let orderbook_doc = get_document(orderbook_yaml);

        // Parse both documents
        let networks =
            Network::parse_and_merge_all_from_yamls(vec![main_doc.clone(), orderbook_doc.clone()])
                .unwrap();

        // Verify networks came from correct documents
        let mainnet = networks.get("mainnet").unwrap();
        let testnet = networks.get("testnet").unwrap();

        // Check document preservation by comparing Arc pointers
        assert!(Arc::ptr_eq(&mainnet.document, &main_doc));
        assert!(Arc::ptr_eq(&testnet.document, &orderbook_doc));

        // Verify we can update each network in its original document
        let mut mainnet = mainnet.clone();
        let mut testnet = testnet.clone();

        mainnet.update_rpc("https://new-mainnet.infura.io").unwrap();
        testnet.update_rpc("https://new-testnet.infura.io").unwrap();

        // Verify the updates went to the correct documents
        let main_yaml = get_yaml_string(main_doc).unwrap();
        let orderbook_yaml = get_yaml_string(orderbook_doc).unwrap();

        assert!(main_yaml.contains("https://new-mainnet.infura.io"));
        assert!(orderbook_yaml.contains("https://new-testnet.infura.io"));
    }
}
