use crate::*;
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use typeshare::typeshare;

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct OrderIO {
    #[typeshare(typescript(type = "Token"))]
    pub token: Arc<Token>,
    #[typeshare(typescript(type = "string"))]
    #[cfg_attr(
        target_family = "wasm",
        tsify(type = "string"),
        serde(rename = "vaultId")
    )]
    pub vault_id: Option<U256>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(OrderIO);

#[typeshare]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Order {
    #[typeshare(typescript(type = "OrderIO[]"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "Vault[]"))]
    pub inputs: Vec<OrderIO>,
    #[typeshare(typescript(type = "OrderIO[]"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "Vault[]"))]
    pub outputs: Vec<OrderIO>,
    #[typeshare(typescript(type = "Network"))]
    pub network: Arc<Network>,
    #[typeshare(typescript(type = "Deployer"))]
    pub deployer: Option<Arc<Deployer>>,
    #[typeshare(typescript(type = "Orderbook"))]
    pub orderbook: Option<Arc<Orderbook>>,
}
#[cfg(target_family = "wasm")]
impl_all_wasm_traits!(Order);

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderConfigSourceError {
    #[error("Failed to parse deployer")]
    DeployerParseError(ParseDeployerConfigSourceError),
    #[error("Failed to parse orderbook")]
    OrderbookParseError(ParseOrderbookConfigSourceError),
    #[error("Failed to parse token")]
    TokenParseError(ParseTokenConfigSourceError),
    #[error("Network not found for Order: {0}")]
    NetworkNotFoundError(String),
    #[error("Network does not match")]
    NetworkNotMatch,
    #[error("Failed to parse vault {}", 0)]
    VaultParseError(#[from] alloy::primitives::ruint::ParseError),
}

impl OrderConfigSource {
    pub fn try_into_order(
        self,
        deployers: &HashMap<String, Arc<Deployer>>,
        orderbooks: &HashMap<String, Arc<Orderbook>>,
        tokens: &HashMap<String, Arc<Token>>,
    ) -> Result<Order, ParseOrderConfigSourceError> {
        let mut network = None;

        let deployer = self
            .deployer
            .map(|deployer_name| {
                deployers
                    .get(&deployer_name)
                    .ok_or(ParseOrderConfigSourceError::DeployerParseError(
                        ParseDeployerConfigSourceError::NetworkNotFoundError(deployer_name.clone()),
                    ))
                    .map(|v| {
                        if let Some(n) = &network {
                            if v.network == *n {
                                Ok(v.clone())
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(v.clone())
                        }
                    })?
            })
            .transpose()?;

        let orderbook = self
            .orderbook
            .map(|orderbook_name| {
                orderbooks
                    .get(&orderbook_name)
                    .ok_or(ParseOrderConfigSourceError::OrderbookParseError(
                        ParseOrderbookConfigSourceError::NetworkNotFoundError(
                            orderbook_name.clone(),
                        ),
                    ))
                    .map(|v| {
                        if let Some(n) = &network {
                            if v.network == *n {
                                Ok(v.clone())
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(v.clone())
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
                    .ok_or(ParseOrderConfigSourceError::TokenParseError(
                        ParseTokenConfigSourceError::NetworkNotFoundError(input.token.clone()),
                    ))
                    .map(|v| {
                        if let Some(n) = &network {
                            if v.network == *n {
                                Ok(OrderIO {
                                    token: v.clone(),
                                    vault_id: input.vault_id,
                                })
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(OrderIO {
                                token: v.clone(),
                                vault_id: input.vault_id,
                            })
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
                    .ok_or(ParseOrderConfigSourceError::TokenParseError(
                        ParseTokenConfigSourceError::NetworkNotFoundError(output.token.clone()),
                    ))
                    .map(|v| {
                        if let Some(n) = &network {
                            if v.network == *n {
                                Ok(OrderIO {
                                    token: v.clone(),
                                    vault_id: output.vault_id,
                                })
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(OrderIO {
                                token: v.clone(),
                                vault_id: output.vault_id,
                            })
                        }
                    })?
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Order {
            inputs,
            outputs,
            network: network.ok_or(ParseOrderConfigSourceError::NetworkNotFoundError(
                String::new(),
            ))?,
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

        let order_string = OrderConfigSource {
            deployer: Some("Deployer1".to_string()),
            orderbook: Some("Orderbook1".to_string()),
            inputs: vec![IOString {
                token: "Token1".to_string(),
                vault_id: Some(U256::from(1)),
            }],
            outputs: vec![IOString {
                token: "Token2".to_string(),
                vault_id: Some(U256::from(2)),
            }],
        };

        let result = order_string.try_into_order(&deployers, &orderbooks, &tokens);
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
        let order_string = OrderConfigSource {
            deployer: None,
            orderbook: None,
            inputs: vec![],
            outputs: vec![],
        };

        let result = order_string.try_into_order(&HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(matches!(
            result,
            Err(ParseOrderConfigSourceError::NetworkNotFoundError(_))
        ));
    }

    #[test]
    fn test_try_into_order_deployer_not_found_error() {
        let deployers = HashMap::new(); // Empty deployer map

        let order_string = OrderConfigSource {
            deployer: Some("Nonexistent Deployer".to_string()),
            orderbook: None,
            inputs: vec![],
            outputs: vec![],
        };

        let result = order_string.try_into_order(&deployers, &HashMap::new(), &HashMap::new());
        assert!(matches!(
            result,
            Err(ParseOrderConfigSourceError::DeployerParseError(_))
        ));
    }

    #[test]
    fn test_try_into_order_orderbook_not_found_error() {
        let orderbooks = HashMap::new(); // Empty orderbook map

        let order_string = OrderConfigSource {
            deployer: None,
            orderbook: Some("Nonexistent Orderbook".to_string()),
            inputs: vec![],
            outputs: vec![],
        };

        let result = order_string.try_into_order(&HashMap::new(), &orderbooks, &HashMap::new());
        assert!(matches!(
            result,
            Err(ParseOrderConfigSourceError::OrderbookParseError(_))
        ));
    }

    #[test]
    fn test_try_into_order_token_not_found_error() {
        let tokens = HashMap::new(); // Empty token map

        let order_string = OrderConfigSource {
            deployer: None,
            orderbook: None,
            inputs: vec![IOString {
                token: "Nonexistent Token".to_string(),
                vault_id: Some(U256::from(1)),
            }],
            outputs: vec![],
        };

        let result = order_string.try_into_order(&HashMap::new(), &HashMap::new(), &tokens);
        assert!(matches!(
            result,
            Err(ParseOrderConfigSourceError::TokenParseError(_))
        ));
    }
}
