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
