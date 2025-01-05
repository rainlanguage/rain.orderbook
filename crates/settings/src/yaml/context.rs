use crate::{Order, OrderIO, Token};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Context {
    pub order: Option<Arc<Order>>,
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

impl Context {
    pub fn new() -> Self {
        Self { order: None }
    }

    pub fn with_order(order: Arc<Order>) -> Self {
        Self { order: Some(order) }
    }

    fn resolve_path(&self, path: &str) -> Result<String, ContextError> {
        let parts: Vec<&str> = path.split('.').collect();

        match parts.get(0) {
            Some(&"order") => self.resolve_order_path(&parts[1..]),
            _ => Err(ContextError::InvalidPath(path.to_string())),
        }
    }

    fn resolve_order_path(&self, parts: &[&str]) -> Result<String, ContextError> {
        let order = self.order.as_ref().ok_or(ContextError::NoOrder)?;
        match parts.get(0) {
            Some(&"inputs") => self.resolve_io_path(&order.inputs, &parts[1..]),
            Some(&"outputs") => self.resolve_io_path(&order.outputs, &parts[1..]),
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_io_path(&self, ios: &[OrderIO], parts: &[&str]) -> Result<String, ContextError> {
        let index = parts
            .get(0)
            .ok_or_else(|| ContextError::InvalidPath(parts.join(".")))?
            .parse::<usize>()
            .map_err(|_| ContextError::InvalidIndex(parts[0].to_string()))?;

        let io = ios
            .get(index)
            .ok_or_else(|| ContextError::InvalidIndex(index.to_string()))?;

        match parts.get(1) {
            Some(&"token") => self.resolve_token_path(&io.token, &parts[2..]),
            Some(&"vault-id") => match &io.vault_id {
                Some(vault_id) => Ok(vault_id.to_string()),
                None => Err(ContextError::PropertyNotFound("vault-id".to_string())),
            },
            _ => Err(ContextError::InvalidPath(parts.join("."))),
        }
    }

    fn resolve_token_path(&self, token: &Token, parts: &[&str]) -> Result<String, ContextError> {
        match parts.get(0) {
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
                token: Arc::new(token.clone()),
                vault_id: Some(U256::from(42)),
            }],
            outputs: vec![OrderIO {
                token: Arc::new(token),
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
        let context = Context::with_order(order.clone());

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
}
