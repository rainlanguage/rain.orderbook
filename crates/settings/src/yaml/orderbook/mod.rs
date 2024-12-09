pub mod accounts;
pub mod deployer;
pub mod metaboards;
pub mod network;
pub mod orderbook_entry;
pub mod sentry;
pub mod subgraphs;
pub mod token;

use super::*;
use crate::Network;
use accounts::AccountsYaml;
use deployer::DeployerYaml;
use metaboards::MetaboardsYaml;
use network::NetworkYaml;
use orderbook_entry::OrderbookEntryYaml;
use sentry::SentryYaml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use subgraphs::SubgraphsYaml;
use token::TokenYaml;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OrderbookYamlFields {
    Networks,
    Subgraphs,
    Metaboards,
    Orderbooks,
    Tokens,
    Deployers,
    Accounts,
    Sentry,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OrderbookYamlData {
    pub networks: HashMap<String, NetworkYaml>,
    pub subgraphs: HashMap<String, String>,
    pub metaboards: HashMap<String, String>,
    pub orderbooks: HashMap<String, OrderbookEntryYaml>,
    pub tokens: HashMap<String, TokenYaml>,
    pub deployers: HashMap<String, DeployerYaml>,
    pub accounts: Option<HashMap<String, String>>,
    pub sentry: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderbookYaml {
    pub source: String,
}

impl OrderbookYaml {
    pub fn new(source: String, validate: bool) -> Result<Self, YamlError> {
        if validate {
            NetworkYaml::try_from_string(&source)?;
            SubgraphsYaml::try_from_string(&source)?;
            MetaboardsYaml::try_from_string(&source)?;
            OrderbookEntryYaml::try_from_string(&source)?;
            TokenYaml::try_from_string(&source)?;
            DeployerYaml::try_from_string(&source)?;
            AccountsYaml::try_from_string(&source)?;
            SentryYaml::try_from_string(&source)?;
        }
        Ok(OrderbookYaml { source })
    }

    pub fn get_source(&self) -> &str {
        &self.source
    }

    pub fn get_networks_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(NetworkYaml::get_network_keys(&self.source)?)
    }
    pub fn get_network(&self, key: &str) -> Result<Network, YamlError> {
        Ok(NetworkYaml::get_network(&self.source, key)?.try_into_network(key)?)
    }
    pub fn update_network_rpc(&mut self, key: &str, rpc: &str) -> Result<(), YamlError> {
        let networks = NetworkYaml::update_network_rpc(&self.source, key, rpc)?;
        self.apply_field_update(
            OrderbookYamlFields::Networks,
            &HashMap::from([("networks".to_string(), networks)]),
        )?;
        Ok(())
    }

    pub fn apply_field_update<T: Serialize>(
        &mut self,
        field: OrderbookYamlFields,
        data: &T,
    ) -> Result<(), YamlError> {
        let network_yaml = if field == OrderbookYamlFields::Networks {
            NetworkYaml::from(data)?
        } else {
            NetworkYaml::try_from_string(&self.source)?
        };

        // TODO: Rest of the fields will be implemented later
        let subgraphs_yaml = SubgraphsYaml::try_from_string(&self.source)?;
        let metaboards_yaml = MetaboardsYaml::try_from_string(&self.source)?;
        let orderbooks_yaml = OrderbookEntryYaml::try_from_string(&self.source)?;
        let tokens_yaml = TokenYaml::try_from_string(&self.source)?;
        let deployers_yaml = DeployerYaml::try_from_string(&self.source)?;
        let accounts_yaml = AccountsYaml::try_from_string(&self.source)?;
        let sentry_yaml = SentryYaml::try_from_string(&self.source)?;

        self.source = serde_yaml::to_string(&OrderbookYamlData {
            networks: network_yaml,
            subgraphs: subgraphs_yaml,
            metaboards: metaboards_yaml,
            orderbooks: orderbooks_yaml,
            tokens: tokens_yaml,
            deployers: deployers_yaml,
            accounts: accounts_yaml,
            sentry: sentry_yaml,
        })
        .map_err(|_| YamlError::ConvertError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    const FULL_YAML: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        secondary: https://api.thegraph.com/subgraphs/name/abc
    metaboards:
        board1: https://meta.example.com/board1
        board2: https://meta.example.com/board2
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
            network: mainnet
            subgraph: main
            label: Primary Orderbook
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    deployers:
        deployer1:
            address: 0x3456789012abcdef
            network: mainnet
            label: Main Deployer
    accounts:
        admin: 0x4567890123abcdef
        user: 0x5678901234abcdef
    sentry: true
    "#;

    const YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
    deployers:
        deployer1:
            address: 0x3456789012abcdef
    "#;

    #[test]
    fn test_full_yaml() {
        let ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();

        assert_eq!(ob_yaml.get_networks_keys().unwrap().len(), 1);
        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://mainnet.infura.io").unwrap()
        );
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.label, Some("Ethereum Mainnet".to_string()));
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.currency, Some("ETH".to_string()));

        assert!(OrderbookYaml::new(YAML_WITHOUT_OPTIONAL_FIELDS.to_string(), true).is_ok());
    }

    #[test]
    fn test_update_network_rpc() {
        let mut ob_yaml = OrderbookYaml::new(FULL_YAML.to_string(), false).unwrap();
        ob_yaml
            .update_network_rpc("mainnet", "https://some-random-rpc-address.com")
            .unwrap();
        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpc,
            Url::parse("https://some-random-rpc-address.com").unwrap()
        );
    }
}
