use crate::*;
use alloy_primitives::hex::FromHexError;
use alloy_primitives::Address;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Orderbook {
    pub address: Address,
    pub network: Arc<Network>,
    pub subgraph: Arc<Subgraph>,
    pub label: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderbookStringError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Network not found: {0}")]
    NetworkNotFoundError(String),
    #[error("Subgraph not found: {0}")]
    SubgraphNotFoundError(String),
}

impl OrderbookString {
    pub fn try_into_orderbook(
        self,
        name: String,
        networks: &HashMap<String, Arc<Network>>,
        subgraphs: &HashMap<String, Arc<Subgraph>>,
    ) -> Result<Orderbook, ParseOrderbookStringError> {
        let network_ref = match self.network {
            Some(network_name) => networks
                .get(&network_name)
                .ok_or(ParseOrderbookStringError::NetworkNotFoundError(
                    network_name.clone(),
                ))
                .map(Arc::clone)?,
            None => networks
                .get(&name)
                .ok_or(ParseOrderbookStringError::NetworkNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        let subgraph_ref = match self.subgraph {
            Some(subgraph_name) => subgraphs
                .get(&subgraph_name)
                .ok_or(ParseOrderbookStringError::SubgraphNotFoundError(
                    subgraph_name.clone(),
                ))
                .map(Arc::clone)?,
            None => subgraphs
                .get(&name)
                .ok_or(ParseOrderbookStringError::SubgraphNotFoundError(
                    name.clone(),
                ))
                .map(Arc::clone)?,
        };

        Ok(Orderbook {
            address: self
                .address
                .parse()
                .map_err(ParseOrderbookStringError::AddressParseError)?,
            network: network_ref,
            subgraph: subgraph_ref,
            label: self.label,
        })
    }
}
