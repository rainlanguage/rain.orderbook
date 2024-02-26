use crate::*;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug)]
pub struct Order {
    pub inputs: Vec<Arc<Token>>,
    pub outputs: Vec<Arc<Token>>,
    pub network: Arc<Network>,
    pub deployer: Option<Arc<Deployer>>,
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
}

impl OrderString {
    pub fn try_into_order(
        self,
        networks: &HashMap<String, Arc<Network>>,
        deployers: &HashMap<String, Arc<Deployer>>,
        orderbooks: &HashMap<String, Arc<Orderbook>>,
        tokens: &HashMap<String, Arc<Token>>,
    ) -> Result<Arc<Order>, ParseOrderStringError> {
        let network_ref = networks
            .get(&self.network)
            .ok_or(ParseOrderStringError::NetworkNotFoundError(
                self.network.clone(),
            ))
            .map(Arc::clone)?;

        let deployer_ref = self
            .deployer
            .map(|deployer_name| {
                deployers
                    .get(&deployer_name)
                    .ok_or(ParseOrderStringError::DeployerParseError(
                        ParseDeployerStringError::NetworkNotFoundError(deployer_name.clone()),
                    ))
                    .map(Arc::clone)
            })
            .transpose()?;

        let orderbook_ref = self
            .orderbook
            .map(|orderbook_name| {
                orderbooks
                    .get(&orderbook_name)
                    .ok_or(ParseOrderStringError::OrderbookParseError(
                        ParseOrderbookStringError::NetworkNotFoundError(orderbook_name.clone()),
                    ))
                    .map(Arc::clone)
            })
            .transpose()?;

        let inputs = self
            .inputs
            .into_iter()
            .map(|input| {
                tokens
                    .get(&input)
                    .ok_or(ParseOrderStringError::TokenParseError(
                        ParseTokenStringError::NetworkNotFoundError(input.clone()),
                    ))
                    .map(Arc::clone)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = self
            .outputs
            .into_iter()
            .map(|output| {
                tokens
                    .get(&output)
                    .ok_or(ParseOrderStringError::TokenParseError(
                        ParseTokenStringError::NetworkNotFoundError(output.clone()),
                    ))
                    .map(Arc::clone)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Arc::new(Order {
            inputs,
            outputs,
            network: network_ref,
            deployer: deployer_ref,
            orderbook: orderbook_ref,
        }))
    }
}
