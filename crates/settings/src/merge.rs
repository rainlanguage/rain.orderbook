use crate::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MergeError {
    #[error("There is already a network called {0}")]
    NetworkCollision(String),

    #[error("There is already a subgraph called {0}")]
    SubgraphCollision(String),

    #[error("There is already an orderbook called {0}")]
    OrderbookCollision(String),

    #[error("There is already a vault called {0}")]
    VaultCollision(String),

    #[error("There is already a token called {0}")]
    TokenCollision(String),

    #[error("There is already a deployer called {0}")]
    DeployerCollision(String),

    #[error("There is already an order called {0}")]
    OrderCollision(String),

    #[error("There is already a scenario called {0}")]
    ScenarioCollision(String),

    #[error("There is already a chart called {0}")]
    ChartCollision(String),
}

impl ConfigString {
    pub fn merge(&mut self, other: ConfigString) -> Result<(), MergeError> {
        // Networks
        let networks = &mut self.networks;
        for (key, value) in other.networks {
            if networks.contains_key(&key) {
                return Err(MergeError::NetworkCollision(key));
            } else {
                networks.insert(key, value);
            }
        }

        // Subgraphs
        let subgraphs = &mut self.subgraphs;
        for (key, value) in other.subgraphs {
            if subgraphs.contains_key(&key) {
                return Err(MergeError::SubgraphCollision(key));
            } else {
                subgraphs.insert(key, value);
            }
        }

        // Orderbooks
        let orderbooks = &mut self.orderbooks;
        for (key, value) in other.orderbooks {
            if orderbooks.contains_key(&key) {
                return Err(MergeError::OrderbookCollision(key));
            } else {
                orderbooks.insert(key, value);
            }
        }

        // Vaults
        let vaults = &mut self.vaults;
        for (key, value) in other.vaults {
            if vaults.contains_key(&key) {
                return Err(MergeError::VaultCollision(key));
            } else {
                vaults.insert(key, value);
            }
        }

        // Tokens
        let tokens = &mut self.tokens;
        for (key, value) in other.tokens {
            if tokens.contains_key(&key) {
                return Err(MergeError::TokenCollision(key));
            } else {
                tokens.insert(key, value);
            }
        }

        // Deployers
        let deployers = &mut self.deployers;
        for (key, value) in other.deployers {
            if deployers.contains_key(&key) {
                return Err(MergeError::DeployerCollision(key));
            } else {
                deployers.insert(key, value);
            }
        }

        // Orders
        let orders = &mut self.orders;
        for (key, value) in other.orders {
            if orders.contains_key(&key) {
                return Err(MergeError::OrderCollision(key));
            } else {
                orders.insert(key, value);
            }
        }

        // Scenarios
        let scenarios = &mut self.scenarios;
        for (key, value) in other.scenarios {
            if scenarios.contains_key(&key) {
                return Err(MergeError::ScenarioCollision(key));
            } else {
                scenarios.insert(key, value);
            }
        }

        // Charts
        let charts = &mut self.charts;
        for (key, value) in other.charts {
            if charts.contains_key(&key) {
                return Err(MergeError::ChartCollision(key));
            } else {
                charts.insert(key, value);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_successful_merge() {
        let mut config = ConfigString {
            subgraphs: HashMap::new(),
            orderbooks: HashMap::new(),
            vaults: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
        };

        let other = ConfigString {
            subgraphs: HashMap::new(),
            orderbooks: HashMap::new(),
            vaults: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
        };

        assert_eq!(config.merge(other), Ok(()));
    }

    #[test]
    fn test_unsuccessful_merge() {
        let mut config = ConfigString {
            subgraphs: HashMap::new(),
            orderbooks: HashMap::new(),
            vaults: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
        };

        let mut other = ConfigString {
            subgraphs: HashMap::new(),
            orderbooks: HashMap::new(),
            vaults: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
        };

        // Add a collision to cause an unsuccessful merge
        config
            .subgraphs
            .insert("subgraph1".to_string(), "value1".to_string());

        other
            .subgraphs
            .insert("subgraph1".to_string(), "value1".to_string());

        assert_eq!(
            config.merge(other),
            Err(MergeError::SubgraphCollision("subgraph1".to_string()))
        );
    }
}
