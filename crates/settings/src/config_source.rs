use crate::blocks::BlocksCfg;
use crate::remote::chains::{chainid::ChainIdError, RemoteNetworkError, RemoteNetworks};
use crate::{GuiConfigSourceCfg, MetricCfg, PlotCfg};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{
    impl_wasm_traits, prelude::*, serialize_hashmap_as_object, serialize_opt_hashmap_as_object,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ConfigSource {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, RemoteNetworksConfigSource>")
    )]
    pub using_networks_from: HashMap<String, RemoteNetworksConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, NetworkConfigSource>")
    )]
    pub networks: HashMap<String, NetworkConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, string>")
    )]
    pub subgraphs: HashMap<String, Url>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, OrderbookConfigSource>")
    )]
    pub orderbooks: HashMap<String, OrderbookConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, TokenConfigSource>")
    )]
    pub tokens: HashMap<String, TokenConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, DeployerConfigSource>")
    )]
    pub deployers: HashMap<String, DeployerConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, OrderConfigSource>")
    )]
    pub orders: HashMap<String, OrderConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, ScenarioConfigSource>")
    )]
    pub scenarios: HashMap<String, ScenarioConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, ChartConfigSource>")
    )]
    pub charts: HashMap<String, ChartConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, DeploymentConfigSource>")
    )]
    pub deployments: HashMap<String, DeploymentConfigSource>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, string>")
    )]
    pub metaboards: HashMap<String, Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentry: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raindex_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(optional, type = "Record<string, string>")
    )]
    pub accounts: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gui: Option<GuiConfigSourceCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ConfigSource);

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgSubgraphRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgScenarioRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgNetworkRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgDeployerRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgOrderRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgOrderbookRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgTokenRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgMetaboardRef = String;

#[cfg_attr(target_family = "wasm", tsify::declare)]
pub type CfgDeploymentRef = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct NetworkConfigSource {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub rpc: Url,
    pub chain_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(NetworkConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RemoteNetworksConfigSource {
    pub url: String,
    pub format: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RemoteNetworksConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderbookConfigSource {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<CfgNetworkRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgraph: Option<CfgSubgraphRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenConfigSource {
    pub network: CfgNetworkRef,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TokenConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeployerConfigSource {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<CfgNetworkRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeployerConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentConfigSource {
    pub scenario: CfgScenarioRef,
    pub order: CfgOrderRef,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct IOStringConfigSource {
    pub token: CfgTokenRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "string"))]
    pub vault_id: Option<U256>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(IOStringConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderConfigSource {
    pub inputs: Vec<IOStringConfigSource>,
    pub outputs: Vec<IOStringConfigSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<CfgDeployerRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbook: Option<CfgOrderbookRef>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ScenarioConfigSource {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, string>")
    )]
    pub bindings: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<BlocksCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployer: Option<CfgDeployerRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(optional, type = "Record<string, ScenarioConfigSource>")
    )]
    pub scenarios: Option<HashMap<String, ScenarioConfigSource>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ScenarioConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ChartConfigSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<CfgScenarioRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(optional, type = "Record<string, PlotCfg>")
    )]
    pub plots: Option<HashMap<String, PlotCfg>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<MetricCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartConfigSource);

#[derive(Error, Debug)]
pub enum ConfigSourceError {
    #[error(transparent)]
    YamlDeserializerError(#[from] serde_yaml::Error),
    #[error(transparent)]
    RemoteNetworkError(#[from] RemoteNetworkError),
    #[error("Conflicting networks, a network with key '{}' already exists", 0)]
    ConflictingNetworks(String),
    #[error(transparent)]
    ChainIdError(#[from] ChainIdError),
}

impl ConfigSource {
    pub async fn try_from_string(
        val: String,
        top_config: Option<String>,
    ) -> Result<(ConfigSource, ConfigSource), ConfigSourceError> {
        if let Some(top_config) = top_config {
            let merged = MergedConfigSource::new(val, top_config).await?;
            Ok((merged.main, merged.top_config))
        } else {
            let mut conf: ConfigSource = serde_yaml::from_str(&val)?;
            if !conf.using_networks_from.is_empty() {
                for (_key, item) in conf.using_networks_from.iter() {
                    let remote_networks =
                        RemoteNetworks::try_from_remote_network_config_source(item.clone()).await?;
                    match remote_networks {
                        RemoteNetworks::ChainId(chains) => {
                            for chain in &chains {
                                if conf.networks.iter().all(|(k, _v)| *k != chain.short_name) {
                                    if let Ok(v) = chain.clone().try_into() {
                                        conf.networks.insert(chain.short_name.clone(), v);
                                    }
                                } else {
                                    return Err(ConfigSourceError::ConflictingNetworks(
                                        chain.name.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Ok((conf, ConfigSource::default()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct MergedConfigSource {
    main: ConfigSource,
    top_config: ConfigSource,
}

impl MergedConfigSource {
    async fn new(
        main_config: String,
        top_config: String,
    ) -> Result<MergedConfigSource, ConfigSourceError> {
        let mut main_indented = String::new();
        let mut top_config_indented = String::new();

        // indent each line of the given ymls 1 level
        // so they can go under a sep top key on a merged yml
        // this ensures that keys dont collide and also is the
        // safest since the original chars of each yml stay intact
        main_config.lines().for_each(|line| {
            main_indented.push_str("  ");
            main_indented.push_str(line);
            main_indented.push('\n');
        });
        top_config.lines().for_each(|line| {
            top_config_indented.push_str("  ");
            top_config_indented.push_str(line);
            top_config_indented.push('\n');
        });

        // top config can have anchors and main config that sits lower can
        // ref them ie cant use a ref that its anchor comes after the fact
        let merged = format!(
            "top-config:
{}

main:
{}
",
            top_config_indented, main_indented
        );
        let mut merged_conf: MergedConfigSource = serde_yaml::from_str(&merged)?;

        // handle remote networks for both ymls
        if !merged_conf.main.using_networks_from.is_empty() {
            for (_key, item) in merged_conf.main.using_networks_from.iter() {
                let remote_networks =
                    RemoteNetworks::try_from_remote_network_config_source(item.clone()).await?;
                match remote_networks {
                    RemoteNetworks::ChainId(chains) => {
                        for chain in &chains {
                            if merged_conf
                                .main
                                .networks
                                .iter()
                                .all(|(k, _v)| *k != chain.short_name)
                            {
                                if let Ok(v) = chain.clone().try_into() {
                                    merged_conf
                                        .main
                                        .networks
                                        .insert(chain.short_name.clone(), v);
                                }
                            } else {
                                return Err(ConfigSourceError::ConflictingNetworks(
                                    chain.name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        if !merged_conf.top_config.using_networks_from.is_empty() {
            for (_key, item) in merged_conf.top_config.using_networks_from.iter() {
                let remote_networks =
                    RemoteNetworks::try_from_remote_network_config_source(item.clone()).await?;
                match remote_networks {
                    RemoteNetworks::ChainId(chains) => {
                        for chain in &chains {
                            if merged_conf
                                .top_config
                                .networks
                                .iter()
                                .all(|(k, _v)| *k != chain.short_name)
                            {
                                if let Ok(v) = chain.clone().try_into() {
                                    merged_conf
                                        .top_config
                                        .networks
                                        .insert(chain.short_name.clone(), v);
                                }
                            } else {
                                return Err(ConfigSourceError::ConflictingNetworks(
                                    chain.name.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(merged_conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};
    use serde_json::json;

    #[tokio::test]
    async fn parse_yaml_into_configstrings() {
        let mocked_chain_id_server = MockServer::start_async().await;
        let yaml_data = format!(
            r#"
raindex-version: 123

using-networks-from:
    chainid:
        url: {}
        format: chainid

networks:
    mainnet:
        rpc: https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet:
        rpc: https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH

subgraphs:
    mainnet: https://mainnet.subgraph
    testnet: https://testnet.subgraph

orderbooks:
    mainnetOrderbook:
        address: 0xabc0000000000000000000000000000000000001
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
    testnetOrderbook:
        address: 0xabc0000000000000000000000000000000000002
        network: testnet
        subgraph: testnet
        label: Testnet Orderbook

tokens:
    eth:
        network: mainnet
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: mainnet
        address: 0xabc0000000000000000000000000000000000004
        decimals: 18
        label: Dai
        symbol: DAI

deployers:
    mainDeployer:
        address: 0xabc0000000000000000000000000000000000005
        network: mainnet
        label: Main Deployer
    testDeployer:
        address: 0xabc0000000000000000000000000000000000006
        network: testnet
        label: Test Deployer

orders:
    buyETH:
        inputs:
            - token: eth
            - token: dai
        outputs:
            - token: dai
              vault-id: 3
        deployer: mainDeployer
        orderbook: mainnetOrderbook

scenarios:
    mainScenario:
        bindings:
            key1: value1
            key2: value2
        runs: 100
        network: mainnet
        deployer: mainDeployer
        scenarios:
            subScenario1:
                bindings:
                    key3: value3
            subScenario2:
                bindings:
                    key4: value4
charts:
    mainChart:
        scenario: mainScenario
        metrics:
        -   label: A metric
            description: A description
            unit-prefix: $
            unit-suffix: USD
            value: 0.1
        -   label: Another metric
            unit-suffix: ETH
            value: 0.2
        -   label: Yet another metric
            unit-prefix: £
            value: 0.3
        plots:
            plot1:
                title: "My plot"
                subtitle: "My subtitle"
                marks:
                -   type: dot
                    options:
                        x: "0.1"
                        y: "0.2"
                        stroke: "black"
            plot2:
                title: "Hexbin"
                marks:
                    - type: dot
                      options:
                        transform:
                            type: hexbin
                            content:
                                outputs:
                                    fill: count
                                options:
                                    x: 0.1
                                    y: 0.2
                                    bin-width: 10
deployments:
    first-deployment:
        scenario: mainScenario
        order: sellETH
    second-deployment:
        scenario: mainScenario
        order: buyETH

sentry: true

accounts:
    name-one: address-one
    name-two: address-two

gui:
  name: Fixed limit
  description: Fixed limit order strategy
  deployments:
    some-deployment:
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
            - "10"
            - "100"
            - "1000"
            - "10000"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value: "false"
            - name: Preset 3
              value: "some-string"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
"#,
            mocked_chain_id_server.url("/json")
        );

        let mocked_chain_id_response = json!([
            {
                "name": "Ethereum Mainnet",
                "chain": "ETH",
                "rpc": ["https://abcd.com/v3/${API_KEY}","https://api.mycryptoapi.com/eth","https://cloudflare-eth.com"],
                "nativeCurrency": {"name": "Ether","symbol": "ETH","decimals": 18},
                "infoURL": "https://ethereum.org",
                "shortName": "eth",
                "chainId": 1,
                "networkId": 1
            },
            {
                "name": "Polygon Mainnet",
                "chain": "Polygon",
                "rpc": ["https://polygon-rpc.com/","wss://polygon.drpc.org"],
                "nativeCurrency": {"name": "MATIC","symbol": "MATIC","decimals": 18},
                "infoURL": "https://polygon.technology/",
                "shortName": "matic",
                "chainId": 137,
                "networkId": 137
            }
        ]);
        mocked_chain_id_server.mock(|when, then| {
            when.method(GET).path("/json");
            then.json_body_obj(&mocked_chain_id_response);
        });

        let config = ConfigSource::try_from_string(yaml_data, None)
            .await
            .unwrap()
            .0;

        // Asserting a few values to verify successful parsing
        assert_eq!(
            config.clone().networks.get("mainnet").unwrap().rpc,
            Url::parse("https://mainnet.node").unwrap()
        );
        assert_eq!(
            config.networks.get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&Url::parse("https://mainnet.subgraph").unwrap())
        );
        assert_eq!(
            config.orderbooks.get("mainnetOrderbook").unwrap().address,
            "0xabc0000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(config.tokens.get("eth").unwrap().decimals, Some(18));
        assert!(config.sentry.unwrap());

        // remote networks fetched from remote source and converted and added to networks
        assert_eq!(
            config.clone().networks.get("eth").unwrap().rpc,
            Url::parse("https://api.mycryptoapi.com/eth").unwrap()
        );
        assert_eq!(
            config.networks.get("eth").unwrap().label,
            Some("Ethereum Mainnet".into())
        );
        assert_eq!(
            config.clone().networks.get("matic").unwrap().rpc,
            Url::parse("https://polygon-rpc.com/").unwrap()
        );
        assert_eq!(
            config.networks.get("matic").unwrap().label,
            Some("Polygon Mainnet".into())
        );

        let expected_order = OrderConfigSource {
            inputs: vec![
                IOStringConfigSource {
                    token: "eth".to_string(),
                    vault_id: None,
                },
                IOStringConfigSource {
                    token: "dai".to_string(),
                    vault_id: None,
                },
            ],
            outputs: vec![IOStringConfigSource {
                token: "dai".to_string(),
                vault_id: Some(U256::from(3)),
            }],
            deployer: Some("mainDeployer".to_string()),
            orderbook: Some("mainnetOrderbook".to_string()),
        };
        let order = config.orders.get("buyETH").unwrap();
        assert_eq!(order.inputs[0].token, expected_order.inputs[0].token);
        assert_eq!(order.inputs[0].vault_id, expected_order.inputs[0].vault_id);
        assert_eq!(order.inputs[1].token, expected_order.inputs[1].token);
        assert_eq!(order.inputs[1].vault_id, expected_order.inputs[1].vault_id);
        assert_eq!(order.outputs[0].token, expected_order.outputs[0].token);
        assert_eq!(
            order.outputs[0].vault_id,
            expected_order.outputs[0].vault_id
        );
        assert_eq!(order.deployer, expected_order.deployer);
        assert_eq!(order.orderbook, expected_order.orderbook);

        assert_eq!(config.raindex_version, Some("123".to_string()));

        let accounts = config.accounts.unwrap();
        assert_eq!(accounts.get("name-one").unwrap(), "address-one");
        assert_eq!(accounts.get("name-two").unwrap(), "address-two");

        let gui = config.gui.unwrap();
        assert_eq!(gui.name, "Fixed limit");
        assert_eq!(gui.description, "Fixed limit order strategy");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = gui.deployments.get("some-deployment").unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = &deployment.deposits[0];
        assert_eq!(deposit.token, "token1".to_string());
        let presets = deposit.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 5);
        assert_eq!(presets[0], "0".to_string());
        assert_eq!(presets[1], "10".to_string());
        assert_eq!(presets[2], "100".to_string());
        assert_eq!(presets[3], "1000".to_string());
        assert_eq!(presets[4], "10000".to_string());
        assert_eq!(deployment.fields.len(), 2);
        let field = &deployment.fields[0];
        assert_eq!(field.binding, "binding-1");
        assert_eq!(field.name, "Field 1 name");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        let presets = field.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].name, Some("Preset 1".to_string()));
        assert_eq!(
            presets[0].value,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
        assert_eq!(presets[1].name, Some("Preset 2".to_string()));
        assert_eq!(presets[1].value, "false".to_string());
        assert_eq!(presets[2].name, Some("Preset 3".to_string()));
        assert_eq!(presets[2].value, "some-string".to_string());
        let field = &deployment.fields[1];
        assert_eq!(field.binding, "binding-2");
        assert_eq!(field.name, "Field 2 name");
        assert_eq!(field.description, Some("Field 2 description".to_string()));
        let presets = field.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].value, "99.2".to_string());
        assert_eq!(presets[1].value, "582.1".to_string());
        assert_eq!(presets[2].value, "648.239".to_string());
    }

    #[tokio::test]
    async fn test_remote_chain_configstrings_unhappy() {
        let mocked_chain_id_server = MockServer::start_async().await;
        let yaml_data = format!(
            r#"
using-networks-from:
    chainid:
        url: {}
        format: chainid"#,
            mocked_chain_id_server.url("/json")
        );

        let mocked_chain_id_response = json!([
            {
                "name": "Ethereum Mainnet",
                "chain": "ETH",
                "rpc": ["https://abcd.com, wss://abcd.com/ws"],
                "nativeCurrency": {"name": "Ether","symbol": "ETH","decimals": 18},
                "infoURL": "https://ethereum.org",
                "shortName": "eth",
                "chainId": 1,
                "networkId": 1
            }
        ]);
        mocked_chain_id_server.mock(|when, then| {
            when.method(GET).path("/json");
            then.json_body_obj(&mocked_chain_id_response);
        });

        let config = ConfigSource::try_from_string(yaml_data, None)
            .await
            .expect_err("expected to fail");
        matches!(config, ConfigSourceError::ChainIdError(_));
    }

    #[tokio::test]
    async fn parse_yaml_into_configstrings_with_anchors() {
        let top_yml_data = r#"
raindex-version: &raindex 123
networks:
    mainnet: &mainnet
        rpc: https://mainnet.node
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
    testnet: &testnet
        rpc: https://testnet.node
        chain-id: 2
        label: Testnet
        network-id: 2
        currency: ETH
subgraphs: &subgraphs
    mainnet: https://mainnet.subgraph
    testnet: https://testnet.subgraph
orderbooks: &orderbooks
    mainnetOrderbook:
        address: 0xabc0000000000000000000000000000000000001
        network: mainnet
        subgraph: mainnet
        label: Mainnet Orderbook
"#;

        let yaml_data = r#"
raindex-version: *raindex
networks:
    mainnet: *mainnet
    testnet: *testnet
subgraphs: *subgraphs
orderbooks: *orderbooks
"#;

        let (config, top_config) =
            ConfigSource::try_from_string(yaml_data.to_string(), Some(top_yml_data.to_string()))
                .await
                .unwrap();

        // Asserting a few values to verify successful parsing for config
        assert_eq!(config.clone().raindex_version.unwrap(), "123".to_string());
        assert_eq!(
            config.clone().networks.get("mainnet").unwrap().rpc,
            Url::parse("https://mainnet.node").unwrap()
        );
        assert_eq!(
            config.networks.get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            config.subgraphs.get("mainnet"),
            Some(&Url::parse("https://mainnet.subgraph").unwrap())
        );
        assert_eq!(
            config.orderbooks.get("mainnetOrderbook").unwrap().address,
            "0xabc0000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );

        // Asserting a few values to verify successful parsing for other config
        assert_eq!(
            top_config.clone().raindex_version.unwrap(),
            "123".to_string()
        );
        assert_eq!(
            top_config.clone().networks.get("mainnet").unwrap().rpc,
            Url::parse("https://mainnet.node").unwrap()
        );
        assert_eq!(
            top_config.networks.get("mainnet").unwrap().label,
            Some("Mainnet".into())
        );
        assert_eq!(
            top_config.subgraphs.get("mainnet"),
            Some(&Url::parse("https://mainnet.subgraph").unwrap())
        );
        assert_eq!(
            top_config
                .orderbooks
                .get("mainnetOrderbook")
                .unwrap()
                .address,
            "0xabc0000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );

        // in this case both configs should be equal
        assert_eq!(config, top_config);
    }
}
