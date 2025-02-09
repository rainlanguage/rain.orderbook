use crate::{yaml::FieldErrorKind, *};
use alloy::primitives::{private::rand, U256};
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
    context::{Context, GuiContextTrait, SelectTokensContext},
    default_document, optional_string, require_hash, require_string, require_vec, YamlError,
    YamlParsableHash,
};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::{impl_all_wasm_traits, wasm_traits::prelude::*};

#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct OrderIO {
    #[typeshare(typescript(type = "Token | undefined"))]
    pub token: Option<Arc<Token>>,
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
    #[cfg_attr(target_family = "wasm", tsify(type = "OrderIO[]"))]
    pub inputs: Vec<OrderIO>,
    #[typeshare(typescript(type = "OrderIO[]"))]
    #[cfg_attr(target_family = "wasm", tsify(type = "OrderIO[]"))]
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

    pub fn update_vault_id(
        &mut self,
        is_input: bool,
        index: u8,
        vault_id: String,
    ) -> Result<Self, YamlError> {
        let new_vault_id = Order::validate_vault_id(&vault_id)?;

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut orders)) =
                document_hash.get_mut(&StrictYaml::String("orders".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut order)) =
                    orders.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    let vec_key = if is_input { "inputs" } else { "outputs" };
                    if let Some(StrictYaml::Array(ref mut vec)) =
                        order.get_mut(&StrictYaml::String(vec_key.to_string()))
                    {
                        if let Some(item) = vec.get_mut(index as usize) {
                            if let StrictYaml::Hash(ref mut item_hash) = item {
                                item_hash.insert(
                                    StrictYaml::String("vault-id".to_string()),
                                    StrictYaml::String(vault_id.to_string()),
                                );
                                if is_input {
                                    self.inputs[index as usize].vault_id = Some(new_vault_id);
                                } else {
                                    self.outputs[index as usize].vault_id = Some(new_vault_id);
                                }
                            } else {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: vec_key.to_string(),
                                        expected: "a hash".to_string(),
                                    },
                                    location: format!("order '{0}'", self.key),
                                });
                            }
                        } else {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: vec_key.to_string(),
                                    reason: format!("index out of bounds: {index}"),
                                },
                                location: format!("order '{0}'", self.key),
                            });
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing(vec_key.to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "orders".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("orders".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn populate_vault_ids(&mut self) -> Result<Self, YamlError> {
        let vault_id: U256 = rand::random();

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut orders)) =
                document_hash.get_mut(&StrictYaml::String("orders".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut order)) =
                    orders.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    if let Some(StrictYaml::Array(ref mut inputs)) =
                        order.get_mut(&StrictYaml::String("inputs".to_string()))
                    {
                        for (index, input) in inputs.iter_mut().enumerate() {
                            if let StrictYaml::Hash(ref mut input_hash) = input {
                                if !input_hash
                                    .contains_key(&StrictYaml::String("vault-id".to_string()))
                                {
                                    input_hash.insert(
                                        StrictYaml::String("vault-id".to_string()),
                                        StrictYaml::String(vault_id.to_string()),
                                    );
                                }
                            } else {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: format!("input index: {index}"),
                                        expected: "a hash".to_string(),
                                    },
                                    location: format!("order '{0}'", self.key),
                                });
                            }
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing("inputs".to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }
                    if let Some(StrictYaml::Array(ref mut outputs)) =
                        order.get_mut(&StrictYaml::String("outputs".to_string()))
                    {
                        for (index, output) in outputs.iter_mut().enumerate() {
                            if let StrictYaml::Hash(ref mut output_hash) = output {
                                if !output_hash
                                    .contains_key(&StrictYaml::String("vault-id".to_string()))
                                {
                                    output_hash.insert(
                                        StrictYaml::String("vault-id".to_string()),
                                        StrictYaml::String(vault_id.to_string()),
                                    );
                                }
                            } else {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: format!("output index: {index}"),
                                        expected: "a hash".to_string(),
                                    },
                                    location: format!("order '{0}'", self.key),
                                });
                            }
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing("outputs".to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }

                    self.inputs.iter_mut().for_each(|input| {
                        input.vault_id = Some(input.vault_id.unwrap_or(vault_id));
                    });
                    self.outputs.iter_mut().for_each(|output| {
                        output.vault_id = Some(output.vault_id.unwrap_or(vault_id));
                    });
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "orders".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("orders".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
    ) -> Result<String, YamlError> {
        let mut network_key: Option<String> = None;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                if let Some(order_yaml) =
                    orders_hash.get(&StrictYaml::String(order_key.to_string()))
                {
                    let location = format!("order '{}'", order_key);

                    if let Some(deployer_key) = optional_string(order_yaml, "deployer") {
                        let key = Deployer::parse_network_key(documents.clone(), &deployer_key)?;

                        if let Some(ref existing_key) = network_key {
                            if *existing_key != key {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::DeployerNetworkDoesNotMatch {
                                        expected: existing_key.clone(),
                                        found: key.clone(),
                                    },
                                ));
                            }
                        } else {
                            network_key = Some(key);
                        }
                    }

                    if let Some(orderbook_key) = optional_string(order_yaml, "orderbook") {
                        let key = Orderbook::parse_network_key(documents.clone(), &orderbook_key)?;

                        if let Some(ref existing_key) = network_key {
                            if *existing_key != key {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::OrderbookNetworkDoesNotMatch {
                                        expected: existing_key.clone(),
                                        found: key.clone(),
                                    },
                                ));
                            }
                        } else {
                            network_key = Some(key);
                        }
                    }

                    for (index, input) in require_vec(order_yaml, "inputs", Some(location.clone()))?
                        .iter()
                        .enumerate()
                    {
                        let location = format!("input index '{index}' in order '{order_key}'");

                        let token_key =
                            require_string(input, Some("token"), Some(location.clone()))?;
                        let res = Token::parse_network_key(documents.clone(), &token_key);
                        if let Ok(key) = res {
                            if let Some(ref existing_key) = network_key {
                                if *existing_key != key {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::InputTokenNetworkDoesNotMatch {
                                            key: token_key,
                                            expected: existing_key.clone(),
                                            found: key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network_key = Some(key);
                            }
                        }
                    }

                    for (index, output) in
                        require_vec(order_yaml, "outputs", Some(location.clone()))?
                            .iter()
                            .enumerate()
                    {
                        let location = format!("output index '{index}' in order '{order_key}'");

                        let token_key =
                            require_string(output, Some("token"), Some(location.clone()))?;
                        let res = Token::parse_network_key(documents.clone(), &token_key);
                        if let Ok(key) = res {
                            if let Some(ref existing_key) = network_key {
                                if *existing_key != key {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::OutputTokenNetworkDoesNotMatch {
                                            key: token_key,
                                            expected: existing_key.clone(),
                                            found: key.clone(),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "orders".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }

        Ok(
            network_key.ok_or(ParseOrderConfigSourceError::NetworkNotFoundError(
                String::new(),
            ))?,
        )
    }

    pub fn parse_vault_ids(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
        is_input: bool,
    ) -> Result<Vec<Option<String>>, YamlError> {
        let mut vault_ids = Vec::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                if let Some(order_yaml) =
                    orders_hash.get(&StrictYaml::String(order_key.to_string()))
                {
                    let location = format!("order '{}'", order_key);

                    let items = if is_input {
                        require_vec(order_yaml, "inputs", Some(location.clone()))?
                    } else {
                        require_vec(order_yaml, "outputs", Some(location.clone()))?
                    };

                    for item in items {
                        let vault_id = optional_string(item, "vault-id");
                        vault_ids.push(vault_id);
                    }
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "orders".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }

        Ok(vault_ids)
    }
}

impl YamlParsableHash for Order {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut orders = HashMap::new();

        let deployers = Deployer::parse_all_from_yaml(documents.clone(), None);
        let orderbooks = Orderbook::parse_all_from_yaml(documents.clone(), None);
        let tokens = Token::parse_all_from_yaml(documents.clone(), None);

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                for (key_yaml, order_yaml) in orders_hash {
                    let order_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("order '{}'", order_key);

                    if let Some(context) = context {
                        if let Some(current_order) = context.get_current_order() {
                            if current_order != &order_key {
                                continue;
                            }
                        }
                    }

                    let mut network: Option<Arc<Network>> = None;

                    let deployer = match optional_string(order_yaml, "deployer") {
                        Some(deployer_name) => {
                            let deployers = deployers.as_ref().map_err(|e| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "deployers".to_string(),
                                    reason: e.to_string(),
                                },
                                location: "root".to_string(),
                            })?;
                            let deployer = Arc::new(
                                deployers
                                    .get(&deployer_name)
                                    .ok_or_else(|| {
                                        YamlError::KeyNotFound(deployer_name.to_string())
                                    })?
                                    .clone(),
                            );
                            if let Some(n) = &network {
                                if deployer.network != *n {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::DeployerNetworkDoesNotMatch {
                                            expected: n.key.clone(),
                                            found: deployer.network.key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network = Some(deployer.network.clone());
                            }
                            Some(deployer)
                        }
                        None => None,
                    };

                    let orderbook = match optional_string(order_yaml, "orderbook") {
                        Some(orderbook_name) => {
                            let orderbooks = orderbooks.as_ref().map_err(|e| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "orderbooks".to_string(),
                                    reason: e.to_string(),
                                },
                                location: "root".to_string(),
                            })?;
                            let orderbook = Arc::new(
                                orderbooks
                                    .get(&orderbook_name)
                                    .ok_or_else(|| {
                                        YamlError::KeyNotFound(orderbook_name.to_string())
                                    })?
                                    .clone(),
                            );
                            if let Some(n) = &network {
                                if orderbook.network != *n {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::OrderbookNetworkDoesNotMatch {
                                            expected: n.key.clone(),
                                            found: orderbook.network.key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network = Some(orderbook.network.clone());
                            }
                            Some(orderbook)
                        }
                        None => None,
                    };

                    let inputs = require_vec(
                        order_yaml,
                        "inputs",
                        Some(location.clone()),
                    )?
                    .iter()
                    .enumerate()
                    .map(|(i, input)| {
                        let location = format!("input index '{i}' in order '{order_key}'");

                        let token_name = require_string(
                            input,
                            Some("token"),
                            Some(location.clone()),
                        )?;

                        let mut order_token = None;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&token_name);

                            if let Some(token) = token {
                                if let Some(n) = &network {
                                    if token.network != *n {
                                        return Err(YamlError::ParseOrderConfigSourceError(
                                            ParseOrderConfigSourceError::InputTokenNetworkDoesNotMatch {
                                                key: token_name,
                                                expected: n.key.clone(),
                                                found: token.network.key.clone(),
                                            },
                                        ));
                                    }
                                } else {
                                    network = Some(token.network.clone());
                                }

                                order_token = Some(token.clone());
                            } else if let Some(context) = context {
                                if !context.is_select_token(&token_name) {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::InvalidValue {
                                            field: "token".to_string(),
                                            reason: format!(
                                                "missing yaml data for token '{token_name}'"
                                            ),
                                        },
                                        location: location.clone(),
                                    });
                                }
                            }
                        } else if let Some(context) = context {
                            if !context.is_select_token(&token_name) {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "token".to_string(),
                                        reason: format!(
                                            "missing yaml data for token '{token_name}'"
                                        ),
                                    },
                                    location: location.clone(),
                                });
                            }
                        }

                        let vault_id = match optional_string(input, "vault-id") {
                            Some(id) => Some(Order::validate_vault_id(&id).map_err(|e| {
                                YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "vault-id".to_string(),
                                        reason: e.to_string(),
                                    },
                                    location: location.clone(),
                                }
                            })?),
                            None => None,
                        };

                        Ok(OrderIO {
                            token: order_token.map(Arc::new),
                            vault_id,
                        })
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let outputs = require_vec(
                        order_yaml,
                        "outputs",
                        Some(location.clone()),
                    )?
                    .iter()
                    .enumerate()
                    .map(|(i, output)| {
                        let location = format!("output index '{i}' in order '{order_key}'");

                        let token_name = require_string(
                            output,
                            Some("token"),
                            Some(location.clone()),
                        )?;

                        let mut order_token = None;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&token_name);

                            if let Some(token) = token {
                                if let Some(n) = &network {
                                    if token.network != *n {
                                        return Err(YamlError::ParseOrderConfigSourceError(
                                            ParseOrderConfigSourceError::OutputTokenNetworkDoesNotMatch {
                                                key: token_name,
                                                expected: n.key.clone(),
                                                found: token.network.key.clone(),
                                            },
                                        ));
                                    }
                                } else {
                                    network = Some(token.network.clone());
                                }

                                order_token = Some(token.clone());
                            } else if let Some(context) = context {
                                if !context.is_select_token(&token_name) {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::InvalidValue {
                                            field: "token".to_string(),
                                            reason: format!(
                                                "missing yaml data for token '{token_name}'"
                                            ),
                                        },
                                        location: location.clone(),
                                    });
                                }
                            }
                        } else if let Some(context) = context {
                            if !context.is_select_token(&token_name) {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "token".to_string(),
                                        reason: format!(
                                            "missing yaml data for token '{token_name}'"
                                        ),
                                    },
                                    location: location.clone(),
                                });
                            }
                        }

                        let vault_id = match optional_string(output, "vault-id") {
                            Some(id) => Some(Order::validate_vault_id(&id).map_err(|e| {
                                YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "vault-id".to_string(),
                                        reason: e.to_string(),
                                    },
                                    location: location.clone(),
                                }
                            })?),
                            None => None,
                        };

                        Ok(OrderIO {
                            token: order_token.map(Arc::new),
                            vault_id,
                        })
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let order = Order {
                        document: document.clone(),
                        key: order_key.clone(),
                        inputs,
                        outputs,
                        network: network.ok_or(
                            ParseOrderConfigSourceError::NetworkNotFoundError(String::new()),
                        )?,
                        deployer,
                        orderbook,
                    };

                    if orders.contains_key(&order_key) {
                        return Err(YamlError::KeyShadowing(order_key));
                    }
                    orders.insert(order_key, order);
                }
            }
        }

        if orders.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("orders".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(orders)
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
    #[error("Deployer network does not match: expected {expected}, found {found}")]
    DeployerNetworkDoesNotMatch { expected: String, found: String },
    #[error("Orderbook network does not match: expected {expected}, found {found}")]
    OrderbookNetworkDoesNotMatch { expected: String, found: String },
    #[error(
        "Input token network with key: {key} does not match: expected {expected}, found {found}"
    )]
    InputTokenNetworkDoesNotMatch {
        key: String,
        expected: String,
        found: String,
    },
    #[error(
        "Output token network with key: {key} does not match: expected {expected}, found {found}"
    )]
    OutputTokenNetworkDoesNotMatch {
        key: String,
        expected: String,
        found: String,
    },
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
                                    token: Some(v.clone()),
                                    vault_id: input.vault_id,
                                })
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(OrderIO {
                                token: Some(v.clone()),
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
                                    token: Some(v.clone()),
                                    vault_id: output.vault_id,
                                })
                            } else {
                                Err(ParseOrderConfigSourceError::NetworkNotMatch)
                            }
                        } else {
                            network = Some(v.network.clone());
                            Ok(OrderIO {
                                token: Some(v.clone()),
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
                .map(|v| v.token.clone().unwrap())
                .collect::<Vec<_>>(),
            vec![token_input]
        );
        assert_eq!(
            order
                .outputs
                .iter()
                .map(|v| v.token.clone().unwrap())
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
        let error = Order::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("orders".to_string()),
                location: "root".to_string()
            }
        );

        let yaml = r#"
orders:
    order1:
"#;
        let error = Order::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("inputs".to_string()),
                location: "order 'order1'".to_string()
            }
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - test: test
"#;
        let error = Order::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "input index '0' in order 'order1'".to_string()
            }
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
        let error = Order::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("outputs".to_string()),
                location: "order 'order1'".to_string()
            }
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
        let error = Order::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "output index '0' in order 'order1'".to_string()
            }
        );
    }

    #[test]
    fn test_parse_orders_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
deployers:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
tokens:
    token-one:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
    token-two:
        network: mainnet
        address: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
orders:
    OrderOne:
        deployer: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;
        let yaml_two = r#"
orders:
    OrderTwo:
        deployer: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let orders = Order::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(orders.len(), 2);
        assert!(orders.contains_key("OrderOne"));
        assert!(orders.contains_key("OrderTwo"));

        assert_eq!(orders.get("OrderOne").unwrap().key, "OrderOne");
        assert_eq!(orders.get("OrderTwo").unwrap().key, "OrderTwo");
    }

    #[test]
    fn test_parse_orders_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpc: "https://mainnet.infura.io"
        chain-id: "1"
deployers:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
tokens:
    token-one:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
    token-two:
        network: mainnet
        address: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
orders:
    DuplicateOrder:
        deployer: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;
        let yaml_two = r#"
orders:
    DuplicateOrder:
        deployer: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = Order::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(error, YamlError::KeyShadowing("DuplicateOrder".to_string()));
    }

    #[test]
    fn parse_network_key() {
        let yaml = r#"
orders: test
"#;
        let error = Order::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orders".to_string(),
                    expected: "a map".to_string()
                },
                location: "root".to_string()
            }
        );

        let yaml = r#"
orders:
  - test
"#;
        let error = Order::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orders".to_string(),
                    expected: "a map".to_string()
                },
                location: "root".to_string()
            }
        );

        let yaml = r#"
orders:
  - test: test
"#;
        let error = Order::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orders".to_string(),
                    expected: "a map".to_string()
                },
                location: "root".to_string()
            }
        );
    }
}
