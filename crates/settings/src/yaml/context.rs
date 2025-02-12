use crate::network_bindings::NetworkBinding;
use crate::{Order, OrderIO, Token};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct GuiContext {
    pub current_deployment: Option<String>,
    pub current_order: Option<String>,
    pub current_network: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub order: Option<Arc<Order>>,
    pub select_tokens: Option<Vec<String>>,
    pub select_networks: Option<Vec<String>>,
    pub gui_context: Option<GuiContext>,
    pub network_bindings: Option<HashMap<String, Arc<NetworkBinding>>>,
}

#[derive(Error, Debug, PartialEq)]
pub enum ContextError {
    #[error("No order in context")]
    NoOrder,
    #[error("No network in context")]
    NoNetwork,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Invalid index: {0}")]
    InvalidIndex(String),
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
}

pub trait OrderContext {
    fn order(&self) -> Option<&Arc<Order>>;

    fn resolve_order_path(&self, parts: &[&str]) -> Result<String, ContextError> {
        let order = self.order().ok_or(ContextError::NoOrder)?;
        match parts.first() {
            Some(&"inputs") => self.resolve_io_path(&order.inputs, &parts[1..]),
            Some(&"outputs") => self.resolve_io_path(&order.outputs, &parts[1..]),
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_io_path(&self, ios: &[OrderIO], parts: &[&str]) -> Result<String, ContextError>;
    fn resolve_token_path(&self, token: &Token, parts: &[&str]) -> Result<String, ContextError>;
}

pub trait NetworkBindingsContext: GuiContextTrait {
    fn resolve_network_binding_path(&self, parts: &[&str]) -> Result<String, ContextError> {
        match parts.first() {
            Some(&binding) => self.resolve_binding(binding),
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_binding(&self, binding: &str) -> Result<String, ContextError>;
}

pub trait GuiSelectionsContext {
    fn select_tokens(&self) -> Option<&Vec<String>>;
    fn select_networks(&self) -> Option<&Vec<String>>;

    fn is_select_token(&self, key: &str) -> bool {
        self.select_tokens()
            .map(|tokens| tokens.iter().any(|t| t == key))
            .unwrap_or(false)
    }

    fn is_select_network(&self, key: &str) -> bool {
        self.select_networks()
            .map(|networks| networks.iter().any(|n| n == key))
            .unwrap_or(false)
    }
}

impl GuiSelectionsContext for Context {
    fn select_tokens(&self) -> Option<&Vec<String>> {
        self.select_tokens.as_ref()
    }

    fn select_networks(&self) -> Option<&Vec<String>> {
        self.select_networks.as_ref()
    }
}

impl OrderContext for Context {
    fn order(&self) -> Option<&Arc<Order>> {
        self.order.as_ref()
    }

    fn resolve_io_path(&self, ios: &[OrderIO], parts: &[&str]) -> Result<String, ContextError> {
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

    fn resolve_token_path(&self, token: &Token, parts: &[&str]) -> Result<String, ContextError> {
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

impl NetworkBindingsContext for Context {
    fn resolve_binding(&self, binding: &str) -> Result<String, ContextError> {
        let network = self.get_current_network().ok_or(ContextError::NoNetwork)?;
        self.network_bindings
            .as_ref()
            .and_then(|bindings| bindings.get(network))
            .and_then(|network_binding| network_binding.bindings.get(binding))
            .ok_or_else(|| ContextError::PropertyNotFound(binding.to_string()))
            .map(|s| s.to_string())
    }
}

pub trait GuiContextTrait {
    fn get_current_deployment(&self) -> Option<&String>;

    fn get_current_order(&self) -> Option<&String>;

    fn get_current_network(&self) -> Option<&String>;
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

    fn get_current_network(&self) -> Option<&String> {
        self.gui_context
            .as_ref()
            .and_then(|gui_context| gui_context.current_network.as_ref())
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            order: None,
            select_tokens: None,
            select_networks: None,
            gui_context: None,
            network_bindings: None,
        }
    }

    pub fn from_context(context: Option<&Context>) -> Self {
        let mut new_context = Self::new();
        if let Some(context) = context {
            new_context.order.clone_from(&context.order);
            new_context.select_tokens.clone_from(&context.select_tokens);
            new_context
                .select_networks
                .clone_from(&context.select_networks);
            new_context.gui_context.clone_from(&context.gui_context);
            new_context
                .network_bindings
                .clone_from(&context.network_bindings);
        }
        new_context
    }

    pub fn add_order(&mut self, order: Arc<Order>) -> &mut Self {
        self.order = Some(order);
        self
    }

    pub fn add_select_tokens(&mut self, select_tokens: Vec<String>) -> &mut Self {
        self.select_tokens = Some(select_tokens);
        self
    }

    pub fn add_select_networks(&mut self, select_networks: Vec<String>) -> &mut Self {
        self.select_networks = Some(select_networks);
        self
    }

    pub fn add_current_deployment(&mut self, deployment: String) -> &mut Self {
        self.gui_context = Some(GuiContext {
            current_deployment: Some(deployment),
            current_order: None,
            current_network: None,
        });
        self
    }

    pub fn add_current_order(&mut self, order: String) -> &mut Self {
        self.gui_context = Some(GuiContext {
            current_deployment: None,
            current_order: Some(order),
            current_network: None,
        });
        self
    }

    pub fn add_current_network(&mut self, network: String) -> &mut Self {
        if let Some(ref mut gui_context) = self.gui_context {
            gui_context.current_network = Some(network);
        } else {
            self.gui_context = Some(GuiContext {
                current_deployment: None,
                current_order: None,
                current_network: Some(network),
            });
        }
        self
    }

    pub fn add_network_bindings(
        &mut self,
        bindings: HashMap<String, Arc<NetworkBinding>>,
    ) -> &mut Self {
        self.network_bindings = Some(bindings);
        self
    }

    fn resolve_path(&self, path: &str) -> Result<String, ContextError> {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.first() {
            Some(&"order") => self.resolve_order_path(&parts[1..]),
            Some(&"network-bindings") => self.resolve_network_binding_path(&parts[1..]),
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
    use crate::test::*;
    use crate::yaml::RwLock;
    use crate::Order;
    use alloy::primitives::{Address, U256};
    use strict_yaml_rust::StrictYaml;

    fn setup_test_order_with_vault_id() -> Arc<Order> {
        let token = Token {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test_token".to_string(),
            network: mock_network(),
            address: Address::repeat_byte(0x42),
            decimals: Some(18),
            label: Some("Test Token".to_string()),
            symbol: Some("TST".to_string()),
        };

        Arc::new(Order {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test_order".to_string(),
            inputs: vec![OrderIO {
                token: Some(Arc::new(token.clone())),
                vault_id: Some(U256::from(42)),
            }],
            outputs: vec![OrderIO {
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
        assert!(context.interpolate("${invalid}").is_err());
        assert!(context
            .interpolate("${order.inputs.999.token.address}")
            .is_err());
        assert!(context
            .interpolate("${order.inputs.0.token.invalid}")
            .is_err());

        // Test vault-id interpolation
        assert_eq!(
            context
                .interpolate("Vault ID: ${order.inputs.0.vault-id}")
                .unwrap(),
            "Vault ID: 42"
        );

        // Test that missing vault-id returns error
        assert!(matches!(
            context.interpolate("${order.outputs.0.vault-id}"),
            Err(ContextError::PropertyNotFound(_))
        ));
    }

    fn setup_test_network_bindings() -> HashMap<String, Arc<NetworkBinding>> {
        let mut bindings = HashMap::new();
        let network_binding = NetworkBinding {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test_network".to_string(),
            bindings: {
                let mut b = HashMap::new();
                b.insert("raindex-subparser".to_string(), "test-value".to_string());
                b
            },
        };
        bindings.insert("test_network".to_string(), Arc::new(network_binding));
        bindings
    }

    #[test]
    fn test_network_binding_interpolation() {
        let mut context = Context::new();
        context.add_network_bindings(setup_test_network_bindings());
        context.add_current_network("test_network".to_string());

        assert_eq!(
            context
                .interpolate("Value: ${network-bindings.raindex-subparser}")
                .unwrap(),
            "Value: test-value"
        );

        assert!(matches!(
            context.interpolate("${network-bindings.non-existent}"),
            Err(ContextError::PropertyNotFound(_))
        ));

        let mut context = Context::new();
        context.add_network_bindings(setup_test_network_bindings());
        assert!(matches!(
            context.interpolate("${network-bindings.raindex-subparser}"),
            Err(ContextError::NoNetwork)
        ));

        let mut context = Context::new();
        context.add_network_bindings(setup_test_network_bindings());
        context.add_current_network("non-existent-network".to_string());
        assert!(matches!(
            context.interpolate("${network-bindings.raindex-subparser}"),
            Err(ContextError::PropertyNotFound(_))
        ));
    }
}
