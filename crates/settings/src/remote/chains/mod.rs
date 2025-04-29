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

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_ok_case() {
        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: "https://chainid.network/chains.json".to_string(),
        };

        let result =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await
                .unwrap();

        let RemoteNetworks::ChainId(chain_ids) = result;

        assert_ne!(chain_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_invalid_response_err_case() {
        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: "https://example.com/".to_string(),
        };

        let result =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await;

        if let Err(RemoteNetworkError::ReqwestError(err)) = &result {
            if err.is_decode() {
                return;
            }
        }

        panic!("Expected RemoteNetworkError::ReqwestError decoding error");
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_invalid_url_err_case() {
        let remote_networks_config_source = RemoteNetworksConfigSource {
            format: "chainid".to_string(),
            url: "https://invalid.url/unreal".to_string(),
        };

        let result =
            RemoteNetworks::try_from_remote_network_config_source(remote_networks_config_source)
                .await;

        if let Err(RemoteNetworkError::ReqwestError(err)) = &result {
            if err.is_connect() {
                return;
            }
        }

        panic!("Expected RemoteNetworkError::ReqwestError connection error");
    }

    #[tokio::test]
    async fn test_try_from_remote_network_config_source_format_parsing_err_case() {
        let remote_networks_config_sources = vec![
            RemoteNetworksConfigSource {
                format: "".to_string(),
                url: "https://example.com".to_string(),
            },
            RemoteNetworksConfigSource {
                format: "unknown".to_string(),
                url: "https://polygon-rpc.com/".to_string(),
            },
        ];

        for remote_networks_config_source in remote_networks_config_sources {
            let unexpected_format = remote_networks_config_source.format.clone();

            let result = RemoteNetworks::try_from_remote_network_config_source(
                remote_networks_config_source,
            )
            .await;

            if let Err(RemoteNetworkError::UnknownFormat(format)) = result {
                assert_eq!(format, unexpected_format);
            } else {
                panic!("Expected RemoteNetworkError::UnknownFormat error");
            }
        }
    }
}
