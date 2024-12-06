pub mod accounts;
pub mod deployer;
pub mod metaboards;
pub mod network;
pub mod orderbook_entry;
pub mod sentry;
pub mod subgraphs;
pub mod token;

use super::*;
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OrderbookYaml {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub networks: HashMap<String, NetworkYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub subgraphs: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metaboards: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub orderbooks: HashMap<String, OrderbookEntryYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tokens: HashMap<String, TokenYaml>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub deployers: HashMap<String, DeployerYaml>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentry: Option<String>,
}

impl OrderbookYaml {
    pub fn try_from_string(source: &str) -> Result<Self, YamlError> {
        Ok(OrderbookYaml {
            networks: NetworkYaml::try_from_string(source)?,
            subgraphs: SubgraphsYaml::try_from_string(source)?,
            metaboards: MetaboardsYaml::try_from_string(source)?,
            orderbooks: OrderbookEntryYaml::try_from_string(source)?,
            tokens: TokenYaml::try_from_string(source)?,
            deployers: DeployerYaml::try_from_string(source)?,
            accounts: AccountsYaml::try_from_string(source)?,
            sentry: SentryYaml::try_from_string(source)?,
        })
    }

    pub fn set_network(&mut self, key: String, network: NetworkYaml) {
        self.networks.insert(key, network);
    }

    pub fn set_subgraph(&mut self, key: String, url: String) {
        self.subgraphs.insert(key, url);
    }

    pub fn set_metaboard(&mut self, key: String, url: String) {
        self.metaboards.insert(key, url);
    }

    pub fn set_orderbook(&mut self, key: String, orderbook: OrderbookEntryYaml) {
        self.orderbooks.insert(key, orderbook);
    }

    pub fn set_token(&mut self, key: String, token: TokenYaml) {
        self.tokens.insert(key, token);
    }

    pub fn set_deployer(&mut self, key: String, deployer: DeployerYaml) {
        self.deployers.insert(key, deployer);
    }

    pub fn set_accounts(&mut self, accounts: HashMap<String, String>) {
        self.accounts = Some(accounts);
    }

    pub fn set_sentry(&mut self, enabled: String) {
        self.sentry = Some(enabled);
    }
}

#[cfg(test)]
mod tests {
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
        let config = OrderbookYaml::try_from_string(FULL_YAML).unwrap();

        assert_eq!(config.networks.len(), 1);
        let network = config.networks.get("mainnet").unwrap();
        assert_eq!(network.rpc, "https://mainnet.infura.io".to_string());
        assert_eq!(network.chain_id, "1".to_string());
        assert_eq!(network.label, Some("Ethereum Mainnet".to_string()));
        assert_eq!(network.network_id, Some("1".to_string()));
        assert_eq!(network.currency, Some("ETH".to_string()));

        assert_eq!(config.subgraphs.len(), 2);
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&"https://api.thegraph.com/subgraphs/name/xyz".to_string())
        );
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&"https://api.thegraph.com/subgraphs/name/xyz".to_string())
        );

        assert_eq!(config.orderbooks.len(), 1);
        let orderbook = config.orderbooks.get("orderbook1").unwrap();
        assert_eq!(orderbook.address, "0x1234567890abcdef");
        assert_eq!(orderbook.network, Some("mainnet".to_string()));
        assert_eq!(orderbook.subgraph, Some("main".to_string()));
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));

        assert_eq!(config.tokens.len(), 1);
        let token = config.tokens.get("token1").unwrap();
        assert_eq!(token.network, "mainnet");
        assert_eq!(token.address, "0x2345678901abcdef");
        assert_eq!(token.decimals, Some("18".to_string()));
        assert_eq!(token.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token.symbol, Some("WETH".to_string()));

        assert_eq!(config.deployers.len(), 1);
        let deployer = config.deployers.get("deployer1").unwrap();
        assert_eq!(deployer.address, "0x3456789012abcdef");
        assert_eq!(deployer.network, Some("mainnet".to_string()));
        assert_eq!(deployer.label, Some("Main Deployer".to_string()));

        assert_eq!(config.accounts.is_some(), true);
        let accounts = config.accounts.unwrap();
        assert_eq!(accounts.len(), 2);
        assert_eq!(
            accounts.get("admin"),
            Some(&"0x4567890123abcdef".to_string())
        );
        assert_eq!(
            accounts.get("user"),
            Some(&"0x5678901234abcdef".to_string())
        );

        assert_eq!(config.sentry, Some("true".to_string()));
    }

    #[test]
    fn test_yaml_without_optional_fields() {
        let config = OrderbookYaml::try_from_string(YAML_WITHOUT_OPTIONAL_FIELDS).unwrap();

        let networks = config.networks.get("mainnet").unwrap();
        assert_eq!(networks.rpc, "https://mainnet.infura.io".to_string());
        assert_eq!(networks.chain_id, "1".to_string());
        assert_eq!(networks.label, None);
        assert_eq!(networks.network_id, None);
        assert_eq!(networks.currency, None);

        assert_eq!(config.subgraphs.len(), 1);
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&"https://api.thegraph.com/subgraphs/name/xyz".to_string())
        );

        assert_eq!(config.orderbooks.len(), 1);
        let orderbook = config.orderbooks.get("orderbook1").unwrap();
        assert_eq!(orderbook.address, "0x1234567890abcdef");
        assert_eq!(orderbook.network, None);
        assert_eq!(orderbook.subgraph, None);
        assert_eq!(orderbook.label, None);

        assert_eq!(config.tokens.len(), 1);
        let token = config.tokens.get("token1").unwrap();
        assert_eq!(token.network, "mainnet");
        assert_eq!(token.address, "0x2345678901abcdef");
        assert_eq!(token.decimals, None);
        assert_eq!(token.label, None);
        assert_eq!(token.symbol, None);

        assert_eq!(config.deployers.len(), 1);
        let deployer = config.deployers.get("deployer1").unwrap();
        assert_eq!(deployer.address, "0x3456789012abcdef");
        assert_eq!(deployer.network, None);
        assert_eq!(deployer.label, None);

        assert_eq!(config.accounts, None);
        assert_eq!(config.sentry, None);
    }
}
