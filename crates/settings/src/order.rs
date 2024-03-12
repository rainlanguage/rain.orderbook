use crate::*;
use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OrderIO {
    #[typeshare(typescript(type = "Token"))]
    pub token: Arc<Token>,
    #[typeshare(typescript(type = "string"))]
    pub vault_id: U256,
}

#[typeshare]
#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[typeshare(typescript(type = "OrderIO[]"))]
    pub inputs: Vec<OrderIO>,
    #[typeshare(typescript(type = "OrderIO[]"))]
    pub outputs: Vec<OrderIO>,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "Deployer"))]
    pub deployer: Option<Arc<Deployer>>,
    #[typeshare(typescript(type = "Orderbook"))]
    pub orderbook: Option<Arc<Orderbook>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderStringError {
    #[error("Failed to parse deployer")]
    DeployerParseError(ParseDeployerStringError),
    #[error("Failed to parse orderbook")]
    OrderbookParseError(ParseOrderbookStringError),
    #[error("Failed to parse token")]
    TokenParseError(ParseTokenStringError),
    #[error("Network not found: {0}")]
    NetworkNotFoundError(String),
    #[error("Network does not match")]
    NetworkNotMatch,
    #[error("Failed to parse vault {}", 0)]
    VaultParseError(#[from] alloy_primitives::ruint::ParseError),
}

impl OrderString {
    pub fn try_into_order(
        self,
        networks: &HashMap<String, Arc<Network>>,
        deployers: &HashMap<String, Arc<Deployer>>,
        orderbooks: &HashMap<String, Arc<Orderbook>>,
        tokens: &HashMap<String, Arc<Token>>,
    ) -> Result<Order, ParseOrderStringError> {
        let network = networks
            .get(&self.network)
            .ok_or(ParseOrderStringError::NetworkNotFoundError(
                self.network.clone(),
            ))
            .map(Arc::clone)?;

        let deployer = self
            .deployer
            .map(|deployer_name| {
                deployers
                    .get(&deployer_name)
                    .ok_or(ParseOrderStringError::DeployerParseError(
                        ParseDeployerStringError::NetworkNotFoundError(deployer_name.clone()),
                    ))
                    .map(|v| {
                        if v.network == network {
                            Ok(v.clone())
                        } else {
                            Err(ParseOrderStringError::NetworkNotMatch)
                        }
                    })?
            })
            .transpose()?;

        let orderbook = self
            .orderbook
            .map(|orderbook_name| {
                orderbooks
                    .get(&orderbook_name)
                    .ok_or(ParseOrderStringError::OrderbookParseError(
                        ParseOrderbookStringError::NetworkNotFoundError(orderbook_name.clone()),
                    ))
                    .map(|v| {
                        if v.network == network {
                            Ok(v.clone())
                        } else {
                            Err(ParseOrderStringError::NetworkNotMatch)
                        }
                    })?
            })
            .transpose()?;

        let inputs = self
            .inputs
            .into_iter()
            .map(|input| {
                tokens
                    .get(&input.token)
                    .ok_or(ParseOrderStringError::TokenParseError(
                        ParseTokenStringError::NetworkNotFoundError(input.token.clone()),
                    ))
                    .map(|v| {
                        if v.network == network {
                            Ok(OrderIO {
                                token: v.clone(),
                                vault_id: input.vault_id.parse::<U256>()?,
                            })
                        } else {
                            Err(ParseOrderStringError::NetworkNotMatch)
                        }
                    })?
            })
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = self
            .outputs
            .into_iter()
            .map(|output| {
                tokens
                    .get(&output.token)
                    .ok_or(ParseOrderStringError::TokenParseError(
                        ParseTokenStringError::NetworkNotFoundError(output.token.clone()),
                    ))
                    .map(|v| {
                        if v.network == network {
                            Ok(OrderIO {
                                token: v.clone(),
                                vault_id: output.vault_id.parse::<U256>()?,
                            })
                        } else {
                            Err(ParseOrderStringError::NetworkNotMatch)
                        }
                    })?
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Order {
            inputs,
            outputs,
            network,
            deployer,
            orderbook,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::*;

    #[test]
    fn test_try_into_order_success() {
        let mut networks = HashMap::new();
        let network = mock_network();
        networks.insert("Local Testnet".to_string(), network);

        let mut deployers = HashMap::new();
        let deployer = mock_deployer();
        deployers.insert("Deployer1".to_string(), deployer);

        let mut orderbooks = HashMap::new();
        let orderbook = mock_orderbook();
        orderbooks.insert("Orderbook1".to_string(), orderbook);

        let mut tokens = HashMap::new();
        let token_input = mock_token("Token1");
        let token_output = mock_token("Token2");
        tokens.insert("Token1".to_string(), token_input.clone());
        tokens.insert("Token2".to_string(), token_output.clone());

        let order_string = OrderString {
            network: "Local Testnet".to_string(),
            deployer: Some("Deployer1".to_string()),
            orderbook: Some("Orderbook1".to_string()),
            inputs: vec![IOString {
                token: "Token1".to_string(),
                vault_id: "1".to_string(),
            }],
            outputs: vec![IOString {
                token: "Token2".to_string(),
                vault_id: "2".to_string(),
            }],
        };

        let result = order_string.try_into_order(&networks, &deployers, &orderbooks, &tokens);
        assert!(result.is_ok());
        let order = result.unwrap();

        assert_eq!(order.network, networks["Local Testnet"]);
        assert_eq!(order.deployer, Some(deployers["Deployer1"].clone()));
        assert_eq!(order.orderbook, Some(orderbooks["Orderbook1"].clone()));
        assert_eq!(
            order
                .inputs
                .iter()
                .map(|v| v.token.clone())
                .collect::<Vec<_>>(),
            vec![token_input]
        );
        assert_eq!(
            order
                .outputs
                .iter()
                .map(|v| v.token.clone())
                .collect::<Vec<_>>(),
            vec![token_output]
        );
    }

    #[test]
    fn test_try_into_order_network_not_found_error() {
        let networks = HashMap::new(); // Empty network map

        let order_string = OrderString {
            network: "Nonexistent Network".to_string(),
            deployer: None,
            orderbook: None,
            inputs: vec![],
            outputs: vec![],
        };

        let result = order_string.try_into_order(
            &networks,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(matches!(
            result,
            Err(ParseOrderStringError::NetworkNotFoundError(_))
        ));
    }

    #[test]
    fn test_try_into_order_deployer_not_found_error() {
        let networks = HashMap::from([("Local Testnet".to_string(), mock_network())]);
        let deployers = HashMap::new(); // Empty deployer map

        let order_string = OrderString {
            network: "Local Testnet".to_string(),
            deployer: Some("Nonexistent Deployer".to_string()),
            orderbook: None,
            inputs: vec![],
            outputs: vec![],
        };

        let result =
            order_string.try_into_order(&networks, &deployers, &HashMap::new(), &HashMap::new());
        assert!(matches!(
            result,
            Err(ParseOrderStringError::DeployerParseError(_))
        ));
    }

    #[test]
    fn test_try_into_order_orderbook_not_found_error() {
        let networks = HashMap::from([("Local Testnet".to_string(), mock_network())]);
        let orderbooks = HashMap::new(); // Empty orderbook map

        let order_string = OrderString {
            network: "Local Testnet".to_string(),
            deployer: None,
            orderbook: Some("Nonexistent Orderbook".to_string()),
            inputs: vec![],
            outputs: vec![],
        };

        let result =
            order_string.try_into_order(&networks, &HashMap::new(), &orderbooks, &HashMap::new());
        assert!(matches!(
            result,
            Err(ParseOrderStringError::OrderbookParseError(_))
        ));
    }

    #[test]
    fn test_try_into_order_token_not_found_error() {
        let networks = HashMap::from([("Local Testnet".to_string(), mock_network())]);
        let tokens = HashMap::new(); // Empty token map

        let order_string = OrderString {
            network: "Local Testnet".to_string(),
            deployer: None,
            orderbook: None,
            inputs: vec![IOString {
                token: "Nonexistent Token".to_string(),
                vault_id: "1".to_string(),
            }],
            outputs: vec![],
        };

        let result =
            order_string.try_into_order(&networks, &HashMap::new(), &HashMap::new(), &tokens);
        assert!(matches!(
            result,
            Err(ParseOrderStringError::TokenParseError(_))
        ));
    }
}
