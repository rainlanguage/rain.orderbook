use crate::*;
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use typeshare::typeshare;
use yaml::{
    default_document, optional_string, require_hash, require_string, require_vec, YamlError,
    YamlParsableMergableHash,
};

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
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct Order {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
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

impl Order {
    pub fn validate_vault_id(value: &str) -> Result<U256, ParseOrderConfigSourceError> {
        U256::from_str(value).map_err(ParseOrderConfigSourceError::VaultParseError)
    }
}

impl YamlParsableMergableHash for Order {
    fn parse_and_merge_all_from_yamls(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut all_orders = HashMap::new();

        // Get all tokens, deployers, and orderbooks from all documents
        let all_tokens = Token::parse_and_merge_all_from_yamls(documents.clone())?;
        let all_deployers = Deployer::parse_and_merge_all_from_yamls(documents.clone())?;
        let all_orderbooks = Orderbook::parse_and_merge_all_from_yamls(documents.clone())?;

        // Only look for orders in the main document (first one)
        let document = &documents[0];
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        let orders_hash = require_hash(
            &document_read,
            Some("orders"),
            Some("missing field: orders".to_string()),
        )?;

        for (key_yaml, order_yaml) in orders_hash {
            let order_key = key_yaml.as_str().unwrap_or_default().to_string();
            let mut network: Option<Arc<Network>> = None;

            let deployer = match optional_string(order_yaml, "deployer") {
                Some(deployer_name) => {
                    let deployer = all_deployers
                        .get(&deployer_name)
                        .or_else(|| all_deployers.get(&order_key))
                        .ok_or_else(|| {
                            YamlError::ParseOrderConfigSourceError(
                                ParseOrderConfigSourceError::DeployerParseError(
                                    ParseDeployerConfigSourceError::NetworkNotFoundError(
                                        deployer_name.clone(),
                                    ),
                                ),
                            )
                        })?
                        .clone();

                    if let Some(n) = &network {
                        if deployer.network != *n {
                            return Err(YamlError::ParseOrderConfigSourceError(
                                ParseOrderConfigSourceError::NetworkNotMatch,
                            ));
                        }
                    } else {
                        network = Some(deployer.network.clone());
                    }
                    Some(Arc::new(deployer))
                }
                None => all_deployers
                    .get(&order_key)
                    .map(|d| {
                        if let Some(n) = &network {
                            if d.network != *n {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::NetworkNotMatch,
                                ));
                            }
                        } else {
                            network = Some(d.network.clone());
                        }
                        Ok(Arc::new(d.clone()))
                    })
                    .transpose()?,
            };

            let orderbook = match optional_string(order_yaml, "orderbook") {
                Some(orderbook_name) => {
                    let orderbook = all_orderbooks
                        .get(&orderbook_name)
                        .or_else(|| all_orderbooks.get(&order_key))
                        .ok_or_else(|| {
                            YamlError::ParseOrderConfigSourceError(
                                ParseOrderConfigSourceError::OrderbookParseError(
                                    ParseOrderbookConfigSourceError::NetworkNotFoundError(
                                        orderbook_name.clone(),
                                    ),
                                ),
                            )
                        })?
                        .clone();

                    if let Some(n) = &network {
                        if orderbook.network != *n {
                            return Err(YamlError::ParseOrderConfigSourceError(
                                ParseOrderConfigSourceError::NetworkNotMatch,
                            ));
                        }
                    } else {
                        network = Some(orderbook.network.clone());
                    }
                    Some(Arc::new(orderbook))
                }
                None => all_orderbooks
                    .get(&order_key)
                    .map(|o| {
                        if let Some(n) = &network {
                            if o.network != *n {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::NetworkNotMatch,
                                ));
                            }
                        } else {
                            network = Some(o.network.clone());
                        }
                        Ok(Arc::new(o.clone()))
                    })
                    .transpose()?,
            };

            let inputs = require_vec(
                order_yaml,
                "inputs",
                Some(format!("inputs list missing in order: {order_key}")),
            )?
            .iter()
            .enumerate()
            .map(|(i, input)| {
                let token_name = require_string(
                    input,
                    Some("token"),
                    Some(format!(
                        "token string missing in input index: {i} in order: {order_key}"
                    )),
                )?;
                let token = all_tokens
                    .get(&token_name)
                    .ok_or_else(|| {
                        YamlError::ParseOrderConfigSourceError(
                            ParseOrderConfigSourceError::TokenParseError(
                                ParseTokenConfigSourceError::NetworkNotFoundError(
                                    token_name.clone(),
                                ),
                            ),
                        )
                    })?
                    .clone();

                if let Some(n) = &network {
                    if token.network != *n {
                        return Err(YamlError::ParseOrderConfigSourceError(
                            ParseOrderConfigSourceError::NetworkNotMatch,
                        ));
                    }
                } else {
                    network = Some(token.network.clone());
                }

                let vault_id = match optional_string(input, "vault-id") {
                    Some(id) => Some(Order::validate_vault_id(&id)?),
                    None => None,
                };

                Ok(OrderIO {
                    token: Arc::new(token),
                    vault_id,
                })
            })
            .collect::<Result<Vec<_>, YamlError>>()?;

            let outputs = require_vec(
                order_yaml,
                "outputs",
                Some(format!("outputs list missing in order: {order_key}")),
            )?
            .iter()
            .enumerate()
            .map(|(i, output)| {
                let token_name = require_string(
                    output,
                    Some("token"),
                    Some(format!(
                        "token string missing in output index: {i} in order: {order_key}"
                    )),
                )?;
                let token = all_tokens
                    .get(&token_name)
                    .ok_or_else(|| {
                        YamlError::ParseOrderConfigSourceError(
                            ParseOrderConfigSourceError::TokenParseError(
                                ParseTokenConfigSourceError::NetworkNotFoundError(
                                    token_name.clone(),
                                ),
                            ),
                        )
                    })?
                    .clone();

                if let Some(n) = &network {
                    if token.network != *n {
                        return Err(YamlError::ParseOrderConfigSourceError(
                            ParseOrderConfigSourceError::NetworkNotMatch,
                        ));
                    }
                } else {
                    network = Some(token.network.clone());
                }

                let vault_id = match optional_string(output, "vault-id") {
                    Some(id) => Some(Order::validate_vault_id(&id)?),
                    None => None,
                };

                Ok(OrderIO {
                    token: Arc::new(token),
                    vault_id,
                })
            })
            .collect::<Result<Vec<_>, YamlError>>()?;

            let order = Order {
                document: document.clone(),
                key: order_key.clone(),
                inputs,
                outputs,
                network: network.ok_or(ParseOrderConfigSourceError::NetworkNotFoundError(
                    String::new(),
                ))?,
                deployer,
                orderbook,
            };

            all_orders.insert(order_key, order);
        }

        Ok(all_orders)
    }
}

impl Default for Order {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: Arc::new(Network::default()),
            deployer: None,
            orderbook: None,
        }
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.inputs == other.inputs
            && self.outputs == other.outputs
            && self.network == other.network
            && self.deployer == other.deployer
            && self.orderbook == other.orderbook
    }
}

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
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
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
    use yaml::tests::get_document;

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

    #[test]
    fn test_parse_orders_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: orders".to_string())
        );

        let yaml = r#"
orders:
    order1:
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("inputs list missing in order: order1".to_string())
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - test: test
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token string missing in input index: 0 in order: order1".to_string()
            )
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("missing field: tokens".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    eth:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
orders:
    order1:
        inputs:
            - token: eth
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError("outputs list missing in order: order1".to_string())
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    eth:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - test: test
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseError(
                "token string missing in output index: 0 in order: order1".to_string()
            )
        );
    }

    #[test]
    fn test_order_with_dependencies_from_multiple_documents() {
        // Main document with order and token
        let main_yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
tokens:
    token1:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2  # token2 is in another document
"#;
        let main_doc = get_document(main_yaml);

        // Second document with another token and orderbook
        let second_yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
tokens:
    token2:
        address: 0x0987654321098765432109876543210987654321
        network: mainnet
orderbooks:
    order1:  # Same name as order for fallback
        address: 0x5555555555555555555555555555555555555555
        network: mainnet
        subgraph: subgraph1
subgraphs:
    subgraph1: https://api.thegraph.com/subgraphs/name/test
"#;
        let second_doc = get_document(second_yaml);

        let orders =
            Order::parse_and_merge_all_from_yamls(vec![main_doc.clone(), second_doc.clone()])
                .unwrap();

        let order = orders.get("order1").unwrap();

        // Check that tokens were found from both documents
        assert_eq!(order.inputs[0].token.key, "token1");
        assert_eq!(order.outputs[0].token.key, "token2");

        // Check that orderbook was found by fallback name matching
        assert!(order.orderbook.is_some());
        assert_eq!(order.orderbook.as_ref().unwrap().key, "order1");
    }

    #[test]
    fn test_order_network_mismatch() {
        let main_yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
    testnet:
        rpc: https://testnet.infura.io
        chain-id: 5
tokens:
    token1:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
    token2:
        address: 0x0987654321098765432109876543210987654321
        network: testnet
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
"#;
        let error =
            Order::parse_and_merge_all_from_yamls(vec![get_document(main_yaml)]).unwrap_err();
        assert!(matches!(
            error,
            YamlError::ParseOrderConfigSourceError(ParseOrderConfigSourceError::NetworkNotMatch)
        ));
    }

    #[test]
    fn test_order_dependency_not_found() {
        let yaml = r#"
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
tokens:
    token1:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
orders:
    order1:
        inputs:
            - token: nonexistent_token
        outputs:
            - token: token1
"#;
        let error = Order::parse_and_merge_all_from_yamls(vec![get_document(yaml)]).unwrap_err();
        assert!(matches!(
            error,
            YamlError::ParseOrderConfigSourceError(ParseOrderConfigSourceError::TokenParseError(_))
        ));
    }
}
