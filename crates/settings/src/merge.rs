use crate::*;
use std::collections::HashMap;
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
        if let Some(other_networks) = other.networks {
            let networks = self.networks.get_or_insert_with(HashMap::new);
            for (key, value) in other_networks {
                if networks.contains_key(&key) {
                    return Err(MergeError::NetworkCollision(key));
                } else {
                    networks.insert(key, value);
                }
            }
        }

        // Subgraphs
        if let Some(other_subgraphs) = other.subgraphs {
            let subgraphs = self.subgraphs.get_or_insert_with(HashMap::new);
            for (key, value) in other_subgraphs {
                if subgraphs.contains_key(&key) {
                    return Err(MergeError::SubgraphCollision(key));
                } else {
                    subgraphs.insert(key, value);
                }
            }
        }

        // Orderbooks
        if let Some(other_orderbooks) = other.orderbooks {
            let orderbooks = self.orderbooks.get_or_insert_with(HashMap::new);
            for (key, value) in other_orderbooks {
                if orderbooks.contains_key(&key) {
                    return Err(MergeError::OrderbookCollision(key));
                } else {
                    orderbooks.insert(key, value);
                }
            }
        }

        // Vaults
        if let Some(other_vaults) = other.vaults {
            let vaults = self.vaults.get_or_insert_with(HashMap::new);
            for (key, value) in other_vaults {
                if vaults.contains_key(&key) {
                    return Err(MergeError::VaultCollision(key));
                } else {
                    vaults.insert(key, value);
                }
            }
        }

        // Tokens
        if let Some(other_tokens) = other.tokens {
            let tokens = self.tokens.get_or_insert_with(HashMap::new);
            for (key, value) in other_tokens {
                if tokens.contains_key(&key) {
                    return Err(MergeError::TokenCollision(key));
                } else {
                    tokens.insert(key, value);
                }
            }
        }

        // Deployers
        if let Some(other_deployers) = other.deployers {
            let deployers = self.deployers.get_or_insert_with(HashMap::new);
            for (key, value) in other_deployers {
                if deployers.contains_key(&key) {
                    return Err(MergeError::DeployerCollision(key));
                } else {
                    deployers.insert(key, value);
                }
            }
        }

        // Orders
        if let Some(other_orders) = other.orders {
            let orders = self.orders.get_or_insert_with(HashMap::new);
            for (key, value) in other_orders {
                if orders.contains_key(&key) {
                    return Err(MergeError::OrderCollision(key));
                } else {
                    orders.insert(key, value);
                }
            }
        }

        // Scenarios
        if let Some(other_scenarios) = other.scenarios {
            let scenarios = self.scenarios.get_or_insert_with(HashMap::new);
            for (key, value) in other_scenarios {
                if scenarios.contains_key(&key) {
                    return Err(MergeError::ScenarioCollision(key));
                } else {
                    scenarios.insert(key, value);
                }
            }
        }

        // Charts
        if let Some(other_charts) = other.charts {
            let charts = self.charts.get_or_insert_with(HashMap::new);
            for (key, value) in other_charts {
                if charts.contains_key(&key) {
                    return Err(MergeError::ChartCollision(key));
                } else {
                    charts.insert(key, value);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_successful_merge() {
        let mut config = ConfigString {
            subgraphs: Some(HashMap::new()),
            orderbooks: Some(HashMap::new()),
            vaults: Some(HashMap::new()),
            tokens: Some(HashMap::new()),
            deployers: Some(HashMap::new()),
            orders: Some(HashMap::new()),
            scenarios: Some(HashMap::new()),
            charts: Some(HashMap::new()),
            networks: Some(HashMap::new()),
        };

        let other = ConfigString {
            subgraphs: Some(HashMap::new()),
            orderbooks: Some(HashMap::new()),
            vaults: Some(HashMap::new()),
            tokens: Some(HashMap::new()),
            deployers: Some(HashMap::new()),
            orders: Some(HashMap::new()),
            scenarios: Some(HashMap::new()),
            charts: Some(HashMap::new()),
            networks: Some(HashMap::new()),
        };

        assert_eq!(config.merge(other), Ok(()));
    }

    #[test]
    fn test_unsuccessful_merge() {
        let mut config = ConfigString {
            subgraphs: Some(HashMap::new()),
            orderbooks: Some(HashMap::new()),
            vaults: Some(HashMap::new()),
            tokens: Some(HashMap::new()),
            deployers: Some(HashMap::new()),
            orders: Some(HashMap::new()),
            scenarios: Some(HashMap::new()),
            charts: Some(HashMap::new()),
            networks: Some(HashMap::new()),
        };

        let mut other = ConfigString {
            subgraphs: Some(HashMap::new()),
            orderbooks: Some(HashMap::new()),
            vaults: Some(HashMap::new()),
            tokens: Some(HashMap::new()),
            deployers: Some(HashMap::new()),
            orders: Some(HashMap::new()),
            scenarios: Some(HashMap::new()),
            charts: Some(HashMap::new()),
            networks: Some(HashMap::new()),
        };

        // Add a collision to cause an unsuccessful merge
        config
            .subgraphs
            .as_mut()
            .unwrap()
            .insert("subgraph1".to_string(), "value1".to_string());

        other
            .subgraphs
            .as_mut()
            .unwrap()
            .insert("subgraph1".to_string(), "value1".to_string());

        assert_eq!(
            config.merge(other),
            Err(MergeError::SubgraphCollision("subgraph1".to_string()))
        );
    }
}
