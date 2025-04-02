use crate::remote::chains::chainid::{ChainId, ChainIdError};
use crate::yaml::context::Context;
use crate::yaml::{
    default_document, require_hash, require_string, FieldErrorKind, YamlError, YamlParsableHash,
};
use crate::NetworkCfg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum RemoteNetworks {
    ChainId(Vec<ChainId>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RemoteNetworksCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
    pub format: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RemoteNetworksCfg);

impl RemoteNetworksCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseRemoteNetworksError> {
        Url::parse(value).map_err(ParseRemoteNetworksError::UrlParseError)
    }

    pub async fn fetch_networks(
        remote_networks: HashMap<String, RemoteNetworksCfg>,
    ) -> Result<HashMap<String, NetworkCfg>, ParseRemoteNetworksError> {
        let mut networks = HashMap::new();

        for (_, remote_network) in remote_networks {
            match remote_network.format.as_str() {
                "chainid" => {
                    let chains = reqwest::get(remote_network.url.to_string())
                        .await?
                        .json::<Vec<ChainId>>()
                        .await?;

                    for chain in &chains {
                        let network: NetworkCfg = chain
                            .clone()
                            .try_into_network_cfg(remote_network.document.clone())?;

                        if networks.contains_key(&network.key) {
                            return Err(ParseRemoteNetworksError::ConflictingNetworks(
                                network.key.clone(),
                            ));
                        }
                        networks.insert(network.key.clone(), network);
                    }
                }
                _ => {
                    return Err(ParseRemoteNetworksError::UnknownFormat(
                        remote_network.format.clone(),
                    ))
                }
            };
        }

        Ok(networks)
    }
}

impl YamlParsableHash for RemoteNetworksCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut remote_networks = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) = require_hash(
                &document_read,
                Some("using-networks-from"),
                Some("root".to_string()),
            ) {
                for (key_yaml, network_yaml) in networks_hash {
                    let key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("using-networks-from '{}'", key);

                    let url_str =
                        require_string(network_yaml, Some("url"), Some(location.clone()))?;
                    let url = RemoteNetworksCfg::validate_url(&url_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "url".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let format =
                        require_string(network_yaml, Some("format"), Some(location.clone()))?;

                    let remote_network = RemoteNetworksCfg {
                        document: document.clone(),
                        key: key.clone(),
                        url,
                        format,
                    };

                    if remote_networks.contains_key(&key) {
                        return Err(YamlError::KeyShadowing(
                            key.clone(),
                            "using-networks-from".to_string(),
                        ));
                    }
                    remote_networks.insert(key, remote_network);
                }
            }
        }

        Ok(remote_networks)
    }
}

impl Default for RemoteNetworksCfg {
    fn default() -> Self {
        RemoteNetworksCfg {
            document: default_document(),
            key: "".to_string(),
            url: Url::parse("https://example.com").unwrap(),
            format: "".to_string(),
        }
    }
}
impl PartialEq for RemoteNetworksCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url && self.format == other.format
    }
}

#[derive(Error, Debug)]
pub enum ParseRemoteNetworksError {
    #[error(transparent)]
    UrlParseError(ParseError),
    #[error("Unknown format: {}", 0)]
    UnknownFormat(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Conflicting networks, a network with key '{}' already exists", 0)]
    ConflictingNetworks(String),
    #[error(transparent)]
    ChainIdError(#[from] ChainIdError),
}

#[cfg(test)]
mod tests {
    use httpmock::MockServer;

    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_parse_remote_networks_from_yaml() {
        let yaml = r#"
using-networks-from:
    test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("url".to_string()),
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url:
        - test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "url".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url:
        - test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "url".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "url".to_string(),
                    reason: "relative URL without a base".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("format".to_string()),
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format:
        - test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "format".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format:
        - test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "format".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_remote_networks_from_yaml_duplicate_key() {
        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format: chainid
"#;
        let error = RemoteNetworksCfg::parse_all_from_yaml(
            vec![get_document(yaml), get_document(yaml)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("test".to_string(), "using-networks-from".to_string())
        );
    }

    #[tokio::test]
    async fn test_fetch_remote_networks() {
        let server = MockServer::start_async().await;
        let yaml = format!(
            r#"
using-networks-from:
    test:
      url: {} # Use the actual mock server URL here
      format: chainid
"#,
            server.base_url()
        );
        let remote_networks =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(&yaml)], None).unwrap();

        let response = r#"
[
    {
        "name": "Remote",
        "chain": "remote-network",
        "chainId": 123,
        "rpc": ["http://localhost:8085/rpc-url"],
        "networkId": 123,
        "nativeCurrency": {
            "name": "Remote",
            "symbol": "RN",
            "decimals": 18
        },
        "infoURL": "http://localhost:8085/info-url",
        "shortName": "remote-network"
    },
    {
        "name": "Remote2",
        "chain": "remote2-network",
        "chainId": 234,
        "rpc": ["http://localhost:8085/rpc-url"],
        "networkId": 123,
        "nativeCurrency": {
            "name": "Remote2",
            "symbol": "RN",
            "decimals": 18
        },
        "infoURL": "http://localhost:8085/info-url",
        "shortName": "remote2-network"
    }
]
        "#;
        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response);
            })
            .await;

        let networks = RemoteNetworksCfg::fetch_networks(remote_networks)
            .await
            .unwrap();

        assert_eq!(networks.len(), 2 as usize);

        let network = networks.get("remote-network").unwrap();
        assert_eq!(network.key, "remote-network");
        assert_eq!(
            network.rpc,
            Url::parse("http://localhost:8085/rpc-url").unwrap()
        );
        assert_eq!(network.chain_id, 123);

        let network = networks.get("remote2-network").unwrap();
        assert_eq!(network.key, "remote2-network");
        assert_eq!(
            network.rpc,
            Url::parse("http://localhost:8085/rpc-url").unwrap()
        );
        assert_eq!(network.chain_id, 234);
    }
}
