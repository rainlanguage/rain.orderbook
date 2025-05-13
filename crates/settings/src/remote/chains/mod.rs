use crate::RemoteNetworksConfigSource;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod chainid;

#[derive(Error, Debug)]
pub enum RemoteNetworkError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Unknown format: {}", 0)]
    UnknownFormat(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum RemoteNetworks {
    ChainId(Vec<chainid::ChainId>),
}

impl RemoteNetworks {
    pub async fn try_from_remote_network_config_source(
        value: RemoteNetworksConfigSource,
    ) -> Result<RemoteNetworks, RemoteNetworkError> {
        match value.format.as_str() {
            "chainid" => Ok(Self::ChainId(
                reqwest::get(value.url)
                    .await?
                    .json::<Vec<chainid::ChainId>>()
                    .await?,
            )),
            _ => Err(RemoteNetworkError::UnknownFormat(value.format.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloy::primitives::Address;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_ok_case() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/chains");

            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"[
                        {
                            "name": "Ethereum Mainnet",
                            "chain": "ETH",
                            "icon": "ethereum",
                            "rpc": [
                                "https://mainnet.infura.io/v3/${INFURA_API_KEY}",
                                "wss://mainnet.infura.io/ws/v3/${INFURA_API_KEY}",
                                "https://rpc.blocknative.com/boost",
                                "https://rpc.flashbots.net",
                                "https://rpc.mevblocker.io",
                                "https://eth.drpc.org",
                                "wss://eth.drpc.org",
                                "https://api.securerpc.com/v1"
                            ],
                            "features": [
                                {
                                "name": "EIP155"
                                },
                                {
                                "name": "EIP1559"
                                }
                            ],
                            "faucets": [],
                            "nativeCurrency": {
                                "name": "Ether",
                                "symbol": "ETH",
                                "decimals": 18
                            },
                            "infoURL": "https://ethereum.org",
                            "shortName": "eth",
                            "chainId": 1,
                            "networkId": 1,
                            "slip44": 60,
                            "ens": {
                                "registry": "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e"
                            },
                            "explorers": [
                                {
                                "name": "etherscan",
                                "url": "https://etherscan.io",
                                "standard": "EIP3091"
                                },
                                {
                                "name": "blockscout",
                                "url": "https://eth.blockscout.com",
                                "icon": "blockscout",
                                "standard": "EIP3091"
                                }
                            ]
                        },
                        {
                            "name": "Expanse Network",
                            "chain": "EXP",
                            "rpc": [
                                "https://node.expanse.tech"
                            ],
                            "faucets": [],
                            "nativeCurrency": {
                                "name": "Expanse Network Ether",
                                "symbol": "EXP",
                                "decimals": 18
                            },
                            "infoURL": "https://expanse.tech",
                            "shortName": "exp",
                            "chainId": 2,
                            "networkId": 1,
                            "slip44": 40
                        }
                    ]"#,
                );
        });

        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: server.url("/chains"),
        };

        let result =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap();

        let RemoteNetworks::ChainId(actual) = result;

        let expected = vec![
            chainid::ChainId {
                name: "Ethereum Mainnet".to_string(),
                chain: "ETH".to_string(),
                icon: Some("ethereum".to_string()),
                rpc: vec![
                    "https://mainnet.infura.io/v3/${INFURA_API_KEY}"
                        .parse()
                        .unwrap(),
                    "wss://mainnet.infura.io/ws/v3/${INFURA_API_KEY}"
                        .parse()
                        .unwrap(),
                    "https://rpc.blocknative.com/boost".parse().unwrap(),
                    "https://rpc.flashbots.net".parse().unwrap(),
                    "https://rpc.mevblocker.io".parse().unwrap(),
                    "https://eth.drpc.org".parse().unwrap(),
                    "wss://eth.drpc.org".parse().unwrap(),
                    "https://api.securerpc.com/v1".parse().unwrap(),
                ],
                features: Some(vec![
                    chainid::Features {
                        name: "EIP155".to_string(),
                    },
                    chainid::Features {
                        name: "EIP1559".to_string(),
                    },
                ]),
                faucets: Some(vec![]),
                native_currency: chainid::NativeCurrency {
                    name: "Ether".to_string(),
                    symbol: "ETH".to_string(),
                    decimals: 18,
                },
                info_url: "https://ethereum.org".to_string(),
                short_name: "eth".to_string(),
                chain_id: 1,
                network_id: 1,
                slip44: Some(60),
                ens: Some(chainid::ENS {
                    registry: Address::parse_checksummed(
                        "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e",
                        None,
                    )
                    .unwrap(),
                }),
                explorers: Some(vec![
                    chainid::Explorer {
                        name: "etherscan".to_string(),
                        url: "https://etherscan.io".parse().unwrap(),
                        icon: None,
                        standard: "EIP3091".to_string(),
                    },
                    chainid::Explorer {
                        name: "blockscout".to_string(),
                        url: "https://eth.blockscout.com".parse().unwrap(),
                        icon: Some("blockscout".to_string()),
                        standard: "EIP3091".to_string(),
                    },
                ]),
                red_flags: None,
            },
            chainid::ChainId {
                name: "Expanse Network".to_string(),
                chain: "EXP".to_string(),
                icon: None,
                rpc: vec!["https://node.expanse.tech".parse().unwrap()],
                features: None,
                faucets: Some(vec![]),
                native_currency: chainid::NativeCurrency {
                    name: "Expanse Network Ether".to_string(),
                    symbol: "EXP".to_string(),
                    decimals: 18,
                },
                info_url: "https://expanse.tech".to_string(),
                short_name: "exp".to_string(),
                chain_id: 2,
                network_id: 1,
                slip44: Some(40),
                ens: None,
                explorers: None,
                red_flags: None,
            },
        ];

        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert_eq!(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_invalid_response_err_case() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/chains");

            then.status(200)
                .header("content-type", "application/json")
                .body("invalid json");
        });

        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: server.url("/chains"),
        };

        let err =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap_err();

        assert!(matches!(
            err,
            RemoteNetworkError::ReqwestError(err) if err.is_decode()
        ));
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_invalid_url_err_case() {
        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: "http://localhost:1".to_string(),
        };

        let err =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap_err();

        assert!(matches!(
            err,
            RemoteNetworkError::ReqwestError(err) if err.is_connect()
        ));
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_format_parsing_err_case() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/chains");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[]"#);
        });

        let remote_networks_config_sources = vec![
            RemoteNetworksConfigSource {
                format: "".to_string(),
                url: server.url("/chains"),
            },
            RemoteNetworksConfigSource {
                format: "unknown".to_string(),
                url: server.url("/chains"),
            },
        ];

        for remote_networks_config_source in remote_networks_config_sources {
            let unexpected_format = remote_networks_config_source.format.clone();

            let err = RemoteNetworks::try_from_remote_network_config_source(
                remote_networks_config_source,
            )
            .await
            .unwrap_err();

            assert!(matches!(
                err,
                RemoteNetworkError::UnknownFormat(format) if format == unexpected_format
            ));
        }
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_empty_chain_list() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/chains");
            then.status(200)
                .header("content-type", "application/json")
                .body("[]");
        });

        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: server.url("/chains"),
        };

        let result =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap();

        let RemoteNetworks::ChainId(chains) = result;
        assert!(chains.is_empty());
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_missing_required_fields() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/chains");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{ "name": "Invalid Chain", "chain": "INV" }]"#);
        });

        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: server.url("/chains"),
        };

        let err =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap_err();

        assert!(matches!(err, RemoteNetworkError::ReqwestError(err) if err.is_decode()));
    }
}
