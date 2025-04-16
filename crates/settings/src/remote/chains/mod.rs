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
