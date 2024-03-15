use crate::*;
use alloy_primitives::hex::FromHexError;
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Orderbook {
    #[typeshare(typescript(type = "string"))]
    pub address: Address,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "string"))]
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
            address: self.address,
            network: network_ref,
            subgraph: subgraph_ref,
            label: self.label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;
    use alloy_primitives::Address;

    fn setup() -> (
        HashMap<String, Arc<Network>>,
        HashMap<String, Arc<Subgraph>>,
    ) {
        let network = mock_network();
        let subgraph = mock_subgraph();

        let mut networks = HashMap::new();
        networks.insert("TestNetwork".to_string(), network);

        let mut subgraphs = HashMap::new();
        subgraphs.insert("TestSubgraph".to_string(), subgraph);

        (networks, subgraphs)
    }

    #[test]
    fn test_orderbook_creation_success() {
        let (networks, subgraphs) = setup();
        let address = "0x1234567890123456789012345678901234567890"
            .parse::<Address>()
            .unwrap();
        let orderbook_string = OrderbookString {
            address,
            network: Some("TestNetwork".to_string()),
            subgraph: Some("TestSubgraph".to_string()),
            label: Some("TestLabel".to_string()),
        };

        let orderbook =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(orderbook.is_ok());
        let orderbook = orderbook.unwrap();

        assert_eq!(orderbook.address, address);
        assert_eq!(
            Arc::as_ptr(&orderbook.network),
            Arc::as_ptr(networks.get("TestNetwork").unwrap())
        );
        assert_eq!(
            Arc::as_ptr(&orderbook.subgraph),
            Arc::as_ptr(subgraphs.get("TestSubgraph").unwrap())
        );
        assert_eq!(orderbook.label, Some("TestLabel".to_string()));
    }

    #[test]
    fn test_orderbook_creation_with_missing_network() {
        let (networks, subgraphs) = setup();
        let orderbook_string = OrderbookString {
            address: Address::random(),
            network: Some("NonExistingNetwork".to_string()),
            subgraph: Some("TestSubgraph".to_string()),
            label: None,
        };

        let result =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseOrderbookStringError::NetworkNotFoundError("NonExistingNetwork".to_string())
        );
    }

    #[test]
    fn test_orderbook_creation_with_missing_subgraph() {
        let (networks, subgraphs) = setup();
        let orderbook_string = OrderbookString {
            address: Address::random(),
            network: Some("TestNetwork".to_string()),
            subgraph: Some("NonExistingSubgraph".to_string()),
            label: None,
        };

        let result =
            orderbook_string.try_into_orderbook("TestName".to_string(), &networks, &subgraphs);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseOrderbookStringError::SubgraphNotFoundError("NonExistingSubgraph".to_string())
        );
    }
}
