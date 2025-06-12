#![allow(clippy::map_entry)]
use crate::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MergeError {
    #[error("There is already a network called {0}")]
    NetworkCollision(String),

    #[error("There is already a subgraph called {0}")]
    SubgraphCollision(String),

    #[error("There is already a metaboard called {0}")]
    MetaboardCollision(String),

    #[error("There is already an orderbook called {0}")]
    OrderbookCollision(String),

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

    #[error("There is already a deployment called {0}")]
    DeploymentCollision(String),

    #[error("There is already a remote networks definition called {0}")]
    RemoteNetworksCollision(String),

    #[error("There is already a accounts called {0}")]
    AccountsCollision(String),
}

impl ConfigSource {
    pub fn merge(&mut self, other: ConfigSource) -> Result<(), MergeError> {
        // Networks
        let networks = &mut self.networks;
        for (key, value) in other.networks {
            if networks.contains_key(&key) {
                return Err(MergeError::NetworkCollision(key));
            }
            networks.insert(key, value);
        }

        // Remote Networks
        let remote_networks = &mut self.using_networks_from;
        for (key, value) in other.using_networks_from {
            if remote_networks.contains_key(&key) {
                return Err(MergeError::NetworkCollision(key));
            }
            remote_networks.insert(key, value);
        }

        // Subgraphs
        let subgraphs = &mut self.subgraphs;
        for (key, value) in other.subgraphs {
            if subgraphs.contains_key(&key) {
                return Err(MergeError::SubgraphCollision(key));
            }
            subgraphs.insert(key, value);
        }

        // Metaboards
        let metaboards = &mut self.metaboards;
        for (key, value) in other.metaboards {
            if metaboards.contains_key(&key) {
                return Err(MergeError::MetaboardCollision(key));
            }
            metaboards.insert(key, value);
        }

        // Orderbooks
        let orderbooks = &mut self.orderbooks;
        for (key, value) in other.orderbooks {
            if orderbooks.contains_key(&key) {
                return Err(MergeError::OrderbookCollision(key));
            }
            orderbooks.insert(key, value);
        }

        // Tokens
        let tokens = &mut self.tokens;
        for (key, value) in other.tokens {
            if tokens.contains_key(&key) {
                return Err(MergeError::TokenCollision(key));
            }
            tokens.insert(key, value);
        }

        // Deployers
        let deployers = &mut self.deployers;
        for (key, value) in other.deployers {
            if deployers.contains_key(&key) {
                return Err(MergeError::DeployerCollision(key));
            }
            deployers.insert(key, value);
        }

        // Orders
        let orders = &mut self.orders;
        for (key, value) in other.orders {
            if orders.contains_key(&key) {
                return Err(MergeError::OrderCollision(key));
            }
            orders.insert(key, value);
        }

        // Scenarios
        let scenarios = &mut self.scenarios;
        for (key, value) in other.scenarios {
            if scenarios.contains_key(&key) {
                return Err(MergeError::ScenarioCollision(key));
            }
            scenarios.insert(key, value);
        }

        // Charts
        let charts = &mut self.charts;
        for (key, value) in other.charts {
            if charts.contains_key(&key) {
                return Err(MergeError::ChartCollision(key));
            }
            charts.insert(key, value);
        }

        // Deployments
        let deployments = &mut self.deployments;
        for (key, value) in other.deployments {
            if deployments.contains_key(&key) {
                return Err(MergeError::DeploymentCollision(key));
            }
            deployments.insert(key, value);
        }

        // Sentry
        self.sentry = match (self.sentry, other.sentry) {
            (Some(a), None) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(MergeError::DeploymentCollision("sentry".into())),
        }?;

        // Accounts
        match (&mut self.accounts, other.accounts) {
            (Some(accounts), Some(other_accounts)) => {
                for (key, value) in other_accounts {
                    if accounts.contains_key(&key) {
                        return Err(MergeError::AccountsCollision(key));
                    }
                    accounts.insert(key, value);
                }
            }
            (None, Some(other_accounts)) => {
                self.accounts = Some(other_accounts);
            }
            _ => {}
        }

        Ok(())
    }
}

impl Config {
    pub fn merge(&mut self, other: Config) -> Result<(), MergeError> {
        // Networks
        let networks = &mut self.networks;
        for (key, value) in other.networks {
            if networks.contains_key(&key) {
                return Err(MergeError::NetworkCollision(key));
            }
            networks.insert(key, value.clone());
        }

        // Subgraphs
        let subgraphs = &mut self.subgraphs;
        for (key, value) in other.subgraphs {
            if subgraphs.contains_key(&key) {
                return Err(MergeError::SubgraphCollision(key));
            }
            subgraphs.insert(key, value.clone());
        }

        // Metaboards
        let metaboards = &mut self.metaboards;
        for (key, value) in other.metaboards {
            if metaboards.contains_key(&key) {
                return Err(MergeError::MetaboardCollision(key));
            }
            metaboards.insert(key, value.clone());
        }

        // Orderbooks
        let orderbooks = &mut self.orderbooks;
        for (key, value) in other.orderbooks {
            if orderbooks.contains_key(&key) {
                return Err(MergeError::OrderbookCollision(key));
            }
            orderbooks.insert(key, value.clone());
        }

        // Tokens
        let tokens = &mut self.tokens;
        for (key, value) in other.tokens {
            if tokens.contains_key(&key) {
                return Err(MergeError::TokenCollision(key));
            }
            tokens.insert(key, value.clone());
        }

        // Deployers
        let deployers = &mut self.deployers;
        for (key, value) in other.deployers {
            if deployers.contains_key(&key) {
                return Err(MergeError::DeployerCollision(key));
            }
            deployers.insert(key, value.clone());
        }

        // Orders
        let orders = &mut self.orders;
        for (key, value) in other.orders {
            if orders.contains_key(&key) {
                return Err(MergeError::OrderCollision(key));
            }
            orders.insert(key, value.clone());
        }

        // Scenarios
        let scenarios = &mut self.scenarios;
        for (key, value) in other.scenarios {
            if scenarios.contains_key(&key) {
                return Err(MergeError::ScenarioCollision(key));
            }
            scenarios.insert(key, value.clone());
        }

        // Charts
        let charts = &mut self.charts;
        for (key, value) in other.charts {
            if charts.contains_key(&key) {
                return Err(MergeError::ChartCollision(key));
            }
            charts.insert(key, value.clone());
        }

        // Deployments
        let deployments = &mut self.deployments;
        for (key, value) in other.deployments {
            if deployments.contains_key(&key) {
                return Err(MergeError::DeploymentCollision(key));
            }
            deployments.insert(key, value);
        }

        // Sentry
        self.sentry = match (self.sentry, other.sentry) {
            (Some(a), None) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => Err(MergeError::DeploymentCollision("sentry".into())),
        }?;

        // Accounts
        match (&mut self.accounts, other.accounts) {
            (Some(accounts), Some(other_accounts)) => {
                for (key, value) in other_accounts {
                    if accounts.contains_key(&key) {
                        return Err(MergeError::AccountsCollision(key));
                    }
                    accounts.insert(key, value);
                }
            }
            (None, Some(other_accounts)) => {
                self.accounts = Some(other_accounts);
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::spec_version::SpecVersion;

    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_successful_merge() {
        let mut config = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        let other = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        assert_eq!(config.merge(other), Ok(()));
    }

    #[test]
    fn test_unsuccessful_merge() {
        let mut config = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        let mut other = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        // Add a collision to cause an unsuccessful merge
        config.subgraphs.insert(
            "subgraph1".to_string(),
            Url::parse("https://myurl").unwrap(),
        );

        other.subgraphs.insert(
            "subgraph1".to_string(),
            Url::parse("https://myurl").unwrap(),
        );

        assert_eq!(
            config.merge(other),
            Err(MergeError::SubgraphCollision("subgraph1".to_string()))
        );
    }

    #[test]
    fn test_successful_merge_metaboard() {
        let mut config = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        let mut other = ConfigSource {
            version: SpecVersion::current().to_string(),
            using_networks_from: HashMap::new(),
            subgraphs: HashMap::new(),
            metaboards: HashMap::new(),
            orderbooks: HashMap::new(),
            tokens: HashMap::new(),
            deployers: HashMap::new(),
            orders: HashMap::new(),
            scenarios: HashMap::new(),
            charts: HashMap::new(),
            networks: HashMap::new(),
            deployments: HashMap::new(),
            sentry: None,
            accounts: None,
            gui: None,
        };

        other.metaboards.insert(
            "metaboard1".to_string(),
            Url::parse("https://myurl").unwrap(),
        );

        assert!(config.metaboards.is_empty());

        assert_eq!(config.merge(other), Ok(()));

        assert_eq!(
            config.metaboards.get("metaboard1"),
            Some(&Url::parse("https://myurl").unwrap())
        );
    }
}
