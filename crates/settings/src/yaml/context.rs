use crate::{NetworkCfg, OrderCfg, OrderIOCfg, TokenCfg};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct GuiContext {
    pub current_deployment: Option<String>,
    pub current_order: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct YamlCache {
    pub remote_networks: HashMap<String, NetworkCfg>,
    pub remote_tokens: HashMap<String, TokenCfg>,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub order: Option<Arc<OrderCfg>>,
    pub select_tokens: Option<Vec<String>>,
    pub gui_context: Option<GuiContext>,
    pub yaml_cache: Option<YamlCache>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ContextError {
    #[error("No order in context")]
    NoOrder,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Invalid index: {0}")]
    InvalidIndex(String),
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
}

impl ContextError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ContextError::NoOrder =>
                "No order is available in the current context. Please ensure an order is specified in your YAML configuration.".to_string(),
            ContextError::InvalidPath(path) =>
                format!("The path '{}' in your YAML configuration is invalid. Please check the syntax and ensure all path segments are correct.", path),
            ContextError::InvalidIndex(index) =>
                format!("The index '{}' in your YAML configuration is invalid. Please ensure the index is a valid number and within the bounds of the array.", index),
            ContextError::PropertyNotFound(property) =>
                format!("The property '{}' was not found in your YAML configuration. Please check that this property is defined correctly.", property),
        }
    }
}

pub trait OrderContext {
    fn order(&self) -> Option<&Arc<OrderCfg>>;

    fn resolve_order_path(&self, parts: &[&str]) -> Result<String, ContextError> {
        let order = self.order().ok_or(ContextError::NoOrder)?;
        match parts.first() {
            Some(&"inputs") => self.resolve_io_path(&order.inputs, &parts[1..]),
            Some(&"outputs") => self.resolve_io_path(&order.outputs, &parts[1..]),
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_io_path(&self, ios: &[OrderIOCfg], parts: &[&str]) -> Result<String, ContextError>;
    fn resolve_token_path(&self, token: &TokenCfg, parts: &[&str]) -> Result<String, ContextError>;
}

pub trait SelectTokensContext {
    fn select_tokens(&self) -> Option<&Vec<String>>;

    fn is_select_token(&self, key: &str) -> bool {
        self.select_tokens()
            .map(|tokens| tokens.iter().any(|t| t == key))
            .unwrap_or(false)
    }
}

impl SelectTokensContext for Context {
    fn select_tokens(&self) -> Option<&Vec<String>> {
        self.select_tokens.as_ref()
    }
}

impl OrderContext for Context {
    fn order(&self) -> Option<&Arc<OrderCfg>> {
        self.order.as_ref()
    }

    fn resolve_io_path(&self, ios: &[OrderIOCfg], parts: &[&str]) -> Result<String, ContextError> {
        let index = parts
            .first()
            .ok_or_else(|| ContextError::InvalidPath(parts.join(".")))?
            .parse::<usize>()
            .map_err(|_| ContextError::InvalidIndex(parts[0].to_string()))?;

        let io = ios
            .get(index)
            .ok_or_else(|| ContextError::InvalidIndex(index.to_string()))?;

        match parts.get(1) {
            Some(&"token") => match &io.token {
                Some(token) => self.resolve_token_path(token, &parts[2..]),
                None => Err(ContextError::PropertyNotFound("token".to_string())),
            },
            Some(&"vault-id") => match &io.vault_id {
                Some(vault_id) => Ok(vault_id.to_string()),
                None => Err(ContextError::PropertyNotFound("vault-id".to_string())),
            },
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_token_path(&self, token: &TokenCfg, parts: &[&str]) -> Result<String, ContextError> {
        match parts.first() {
            Some(&"address") => Ok(format!("{:?}", token.address)),
            Some(&"symbol") => Ok(token
                .symbol
                .clone()
                .ok_or_else(|| ContextError::PropertyNotFound("symbol".to_string()))?),
            Some(&"label") => Ok(token
                .label
                .clone()
                .ok_or_else(|| ContextError::PropertyNotFound("label".to_string()))?),
            Some(&"decimals") => Ok(token
                .decimals
                .ok_or_else(|| ContextError::PropertyNotFound("decimals".to_string()))?
                .to_string()),
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }
}

pub trait GuiContextTrait {
    fn get_current_deployment(&self) -> Option<&String>;

    fn get_current_order(&self) -> Option<&String>;
}

impl GuiContextTrait for Context {
    fn get_current_deployment(&self) -> Option<&String> {
        self.gui_context
            .as_ref()
            .and_then(|gui_context| gui_context.current_deployment.as_ref())
    }

    fn get_current_order(&self) -> Option<&String> {
        self.gui_context
            .as_ref()
            .and_then(|gui_context| gui_context.current_order.as_ref())
    }
}

pub trait YamlCacheTrait {
    fn get_yaml_cache(&self) -> Option<&YamlCache>;

    fn get_remote_networks(&self) -> Option<&HashMap<String, NetworkCfg>>;

    fn get_remote_network(&self, key: &str) -> Option<&NetworkCfg>;

    fn get_remote_tokens(&self) -> Option<&HashMap<String, TokenCfg>>;

    fn get_remote_token(&self, key: &str) -> Option<&TokenCfg>;
}

impl YamlCacheTrait for Context {
    fn get_yaml_cache(&self) -> Option<&YamlCache> {
        self.yaml_cache.as_ref()
    }

    fn get_remote_networks(&self) -> Option<&HashMap<String, NetworkCfg>> {
        self.yaml_cache.as_ref().map(|cache| &cache.remote_networks)
    }

    fn get_remote_network(&self, key: &str) -> Option<&NetworkCfg> {
        self.yaml_cache
            .as_ref()
            .and_then(|cache| cache.remote_networks.get(key))
    }

    fn get_remote_tokens(&self) -> Option<&HashMap<String, TokenCfg>> {
        self.yaml_cache.as_ref().map(|cache| &cache.remote_tokens)
    }

    fn get_remote_token(&self, key: &str) -> Option<&TokenCfg> {
        self.yaml_cache
            .as_ref()
            .and_then(|cache| cache.remote_tokens.get(key))
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            order: None,
            select_tokens: None,
            gui_context: None,
            yaml_cache: None,
        }
    }

    pub fn from_context(context: Option<&Context>) -> Self {
        let mut new_context = Self::new();
        if let Some(context) = context {
            new_context.order.clone_from(&context.order);
            new_context.select_tokens.clone_from(&context.select_tokens);
            new_context.gui_context.clone_from(&context.gui_context);
            new_context.yaml_cache.clone_from(&context.yaml_cache);
        }
        new_context
    }

    pub fn add_order(&mut self, order: Arc<OrderCfg>) -> &mut Self {
        self.order = Some(order);
        self
    }

    pub fn add_select_tokens(&mut self, select_tokens: Vec<String>) -> &mut Self {
        self.select_tokens = Some(select_tokens);
        self
    }

    pub fn add_current_deployment(&mut self, deployment: String) -> &mut Self {
        if let Some(gui_context) = self.gui_context.as_mut() {
            gui_context.current_deployment = Some(deployment);
        } else {
            self.gui_context = Some(GuiContext {
                current_deployment: Some(deployment),
                current_order: None,
            });
        }
        self
    }

    pub fn add_current_order(&mut self, order: String) -> &mut Self {
        if let Some(gui_context) = self.gui_context.as_mut() {
            gui_context.current_order = Some(order);
        } else {
            self.gui_context = Some(GuiContext {
                current_deployment: None,
                current_order: Some(order),
            });
        }
        self
    }

    pub fn set_remote_networks(
        &mut self,
        remote_networks: HashMap<String, NetworkCfg>,
    ) -> &mut Self {
        if let Some(yaml_cache) = self.yaml_cache.as_mut() {
            yaml_cache.remote_networks = remote_networks;
        } else {
            self.yaml_cache = Some(YamlCache {
                remote_networks,
                remote_tokens: HashMap::new(),
            });
        }
        self
    }

    pub fn set_remote_tokens(&mut self, remote_tokens: HashMap<String, TokenCfg>) -> &mut Self {
        if let Some(yaml_cache) = self.yaml_cache.as_mut() {
            yaml_cache.remote_tokens = remote_tokens;
        } else {
            self.yaml_cache = Some(YamlCache {
                remote_tokens,
                remote_networks: HashMap::new(),
            });
        }
        self
    }

    fn resolve_path(&self, path: &str) -> Result<String, ContextError> {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.first() {
            Some(&"order") => self.resolve_order_path(&parts[1..]),
            _ => Err(ContextError::InvalidPath(path.to_string())),
        }
    }

    pub fn interpolate(&self, input: &str) -> Result<String, ContextError> {
        let mut result = input.to_string();
        let mut start = 0;

        while let Some(var_start) = result[start..].find("${") {
            let var_start = start + var_start;
            if let Some(var_end) = result[var_start..].find('}') {
                let var_end = var_start + var_end + 1;
                let var = &result[var_start + 2..var_end - 1];
                let replacement = self.resolve_path(var)?;
                result.replace_range(var_start..var_end, &replacement);
                start = var_start + replacement.len();
            } else {
                break;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::RwLock;
    use crate::OrderCfg;
    use crate::{test::*, yaml::default_document};
    use alloy::primitives::{Address, U256};
    use strict_yaml_rust::StrictYaml;

    fn setup_test_order_with_vault_id() -> Arc<OrderCfg> {
        let token = TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test_token".to_string(),
            network: mock_network(),
            address: Address::repeat_byte(0x42),
            decimals: Some(18),
            label: Some("Test Token".to_string()),
            symbol: Some("TST".to_string()),
        };

        Arc::new(OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test_order".to_string(),
            inputs: vec![OrderIOCfg {
                token: Some(Arc::new(token.clone())),
                vault_id: Some(U256::from(42)),
            }],
            outputs: vec![OrderIOCfg {
                token: Some(Arc::new(token.clone())),
                vault_id: None,
            }],
            network: mock_network(),
            deployer: None,
            orderbook: None,
        })
    }

    #[test]
    fn test_context_interpolation() {
        let order = setup_test_order_with_vault_id();
        let mut context = Context::new();
        context.add_order(order.clone());

        // Test basic interpolation
        assert_eq!(
            context
                .interpolate("Address: ${order.inputs.0.token.address}")
                .unwrap(),
            "Address: 0x4242424242424242424242424242424242424242"
        );

        // Test multiple interpolations
        assert_eq!(
            context
                .interpolate(
                    "Symbol: ${order.inputs.0.token.symbol}, \
                     Label: ${order.inputs.0.token.label}"
                )
                .unwrap(),
            "Symbol: TST, Label: Test Token"
        );

        // Test error cases
        let invalid_path_error = context.interpolate("${invalid}").unwrap_err();
        assert_eq!(
            invalid_path_error.to_readable_msg(),
            "The path 'invalid' in your YAML configuration is invalid. Please check the syntax and ensure all path segments are correct."
        );

        let invalid_index_error = context
            .interpolate("${order.inputs.999.token.address}")
            .unwrap_err();
        assert_eq!(
            invalid_index_error.to_readable_msg(),
            "The index '999' in your YAML configuration is invalid. Please ensure the index is a valid number and within the bounds of the array."
        );

        let property_not_found_error = context
            .interpolate("${order.inputs.0.token.invalid}")
            .unwrap_err();
        assert_eq!(
            property_not_found_error.to_readable_msg(),
            "The path 'invalid' in your YAML configuration is invalid. Please check the syntax and ensure all path segments are correct."
        );

        // Test vault-id interpolation
        assert_eq!(
            context
                .interpolate("Vault ID: ${order.inputs.0.vault-id}")
                .unwrap(),
            "Vault ID: 42"
        );

        // Test that missing vault-id returns error
        let missing_vault_error = context
            .interpolate("${order.outputs.0.vault-id}")
            .unwrap_err();
        assert_eq!(
            missing_vault_error.to_readable_msg(),
            "The property 'vault-id' was not found in your YAML configuration. Please check that this property is defined correctly."
        );
    }

    #[test]
    fn test_context_no_order() {
        let context = Context::new();
        let error = context
            .interpolate("${order.inputs.0.token.address}")
            .unwrap_err();
        assert_eq!(error, ContextError::NoOrder);
        assert_eq!(
            error.to_readable_msg(),
            "No order is available in the current context. Please ensure an order is specified in your YAML configuration."
        );
    }

    #[test]
    fn test_order_context() {
        let mut context = Context::new();

        let order = OrderCfg {
            document: default_document(),
            key: "test_order".to_string(),
            inputs: vec![OrderIOCfg {
                token: Some(mock_token("token1")),
                vault_id: Some(U256::from(10)),
            }],
            outputs: vec![OrderIOCfg {
                token: Some(mock_token("token2")),
                vault_id: None,
            }],
            network: mock_network(),
            deployer: Some(mock_deployer()),
            orderbook: Some(mock_orderbook()),
        };
        context.add_order(Arc::new(order));

        assert!(context.order.is_some());
        let context_order = context.order.unwrap();
        assert_eq!(context_order.key, "test_order");
        assert_eq!(context_order.inputs.len(), 1);
        assert_eq!(context_order.inputs[0].token, Some(mock_token("token1")));
        assert_eq!(context_order.inputs[0].vault_id, Some(U256::from(10)));
        assert_eq!(context_order.outputs.len(), 1);
        assert_eq!(context_order.outputs[0].token, Some(mock_token("token2")));
        assert_eq!(context_order.outputs[0].vault_id, None);
        assert_eq!(context_order.network, mock_network());
        assert_eq!(context_order.deployer, Some(mock_deployer()));
        assert_eq!(context_order.orderbook, Some(mock_orderbook()));
    }

    #[test]
    fn test_select_tokens_context() {
        let mut context = Context::new();
        context.add_select_tokens(vec!["token1".to_string(), "token2".to_string()]);

        assert_eq!(
            context.select_tokens,
            Some(vec!["token1".to_string(), "token2".to_string()])
        );
        assert_eq!(
            context.select_tokens(),
            Some(&vec!["token1".to_string(), "token2".to_string()])
        );
        assert!(context.is_select_token("token1"));
        assert!(context.is_select_token("token2"));
        assert!(!context.is_select_token("token3"));
    }

    #[test]
    fn test_current_deployment_context() {
        let mut context = Context::new();
        context.add_current_deployment("deployment1".to_string());
        assert_eq!(
            context.gui_context.unwrap().current_deployment,
            Some("deployment1".to_string())
        );
    }

    #[test]
    fn test_current_order_context() {
        let mut context = Context::new();
        context.add_current_order("order1".to_string());
        assert_eq!(
            context.gui_context.unwrap().current_order,
            Some("order1".to_string())
        );
    }

    #[test]
    fn test_set_remote_networks() {
        let mut context = Context::new();

        let networks = HashMap::from([("network1".to_string(), mock_network().as_ref().clone())]);
        context.set_remote_networks(networks.clone());

        assert!(context.yaml_cache.is_some());
        let yaml_cache = context.yaml_cache.unwrap();
        assert_eq!(yaml_cache.remote_networks, networks);
    }

    #[test]
    fn test_set_remote_tokens() {
        let mut context = Context::new();

        let tokens = HashMap::from([("token1".to_string(), mock_token("token1").as_ref().clone())]);
        context.set_remote_tokens(tokens.clone());

        assert!(context.yaml_cache.is_some());
        let yaml_cache = context.yaml_cache.unwrap();
        assert_eq!(yaml_cache.remote_tokens, tokens);
    }

    #[test]
    fn test_from_context() {
        let mut context = Context::new();
        context.add_order(Arc::new(OrderCfg::default()));
        context.add_select_tokens(vec!["token1".to_string(), "token2".to_string()]);
        context.add_current_deployment("deployment1".to_string());
        context.add_current_order("order1".to_string());
        context.set_remote_networks(HashMap::from([(
            "network1".to_string(),
            mock_network().as_ref().clone(),
        )]));
        context.set_remote_tokens(HashMap::from([(
            "token1".to_string(),
            mock_token("token1").as_ref().clone(),
        )]));

        let new_context = Context::from_context(Some(&context));

        assert_eq!(new_context.order, context.order);
        assert_eq!(new_context.select_tokens, context.select_tokens);
        assert!(new_context.gui_context.is_some());
        assert!(new_context.yaml_cache.is_some());

        let gui_context = new_context.gui_context.unwrap();
        assert_eq!(
            gui_context.current_deployment,
            Some("deployment1".to_string())
        );
        assert_eq!(gui_context.current_order, Some("order1".to_string()));

        let yaml_cache = new_context.yaml_cache.unwrap();
        assert_eq!(
            yaml_cache.remote_networks,
            context.yaml_cache.as_ref().unwrap().remote_networks
        );
        assert_eq!(
            yaml_cache.remote_tokens,
            context.yaml_cache.unwrap().remote_tokens
        );
    }
}
