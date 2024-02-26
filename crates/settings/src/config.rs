use crate::*;
use alloy_primitives::{Address, U256};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub networks: Option<HashMap<String, Arc<Network>>>,
    pub subgraphs: Option<HashMap<String, Arc<Subgraph>>>,
    pub orderbooks: Option<HashMap<String, Arc<Orderbook>>>,
    pub vaults: Option<HashMap<String, Arc<Vault>>>,
    pub tokens: Option<HashMap<String, Arc<Token>>>,
    pub deployers: Option<HashMap<String, Arc<Deployer>>>,
    pub orders: Option<HashMap<String, Arc<Order>>>,
    pub scenarios: Option<HashMap<String, Arc<Scenario>>>,
    pub charts: Option<HashMap<String, Arc<Chart>>>,
}

pub type Subgraph = Url;
pub type Vault = U256;

#[derive(Error, Debug, PartialEq)]
pub enum ParseConfigStringError {
    #[error("Failed to parse network {}", 0)]
    NetworkParseError(ParseNetworkStringError),
}
