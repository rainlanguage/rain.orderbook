use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use strict_yaml_rust::{scanner::ScanError, StrictYaml, StrictYamlLoader};
use thiserror::Error;
use typeshare::typeshare;
use url::Url;

#[derive(Error, Debug)]
pub enum OrderbookAppSettingsParseError {
    #[error("Invalid Yaml")]
    YamlInvalid(#[from] ScanError),
    #[error("chains element missing field: {0}")]
    ChainSettingsFieldMissing(String),
    #[error("chains element invalid field: {0}")]
    ChainSettingsFieldInvalid(String),
    #[error("orderbooks element missing field: {0}")]
    OrderbookSettingsFieldMissing(String),
    #[error("orderbooks element invalid field: {0}")]
    OrderbookSettingsFieldInvalid(String),
    #[error(transparent)]
    OrderbookAppSettingsBuilder(#[from] OrderbookAppSettingsBuilderError),
    #[error(transparent)]
    OrderbookSettingsBuilder(#[from] OrderbookSettingsBuilderError),
    #[error(transparent)]
    ChainSettingsBuilder(#[from] ChainSettingsBuilderError),
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct ChainSettings {
    label: Option<String>,
    #[typeshare(typescript(type = "string"))]
    rpc_url: Url,
}

impl TryFrom<StrictYaml> for ChainSettings {
    type Error = OrderbookAppSettingsParseError;

    fn try_from(val: StrictYaml) -> Result<ChainSettings, Self::Error> {
        let label = val["label"].as_str().map(|s| s.into());
        let rpc_url_str = val["rpc-url"].as_str().ok_or(
            OrderbookAppSettingsParseError::ChainSettingsFieldMissing("rpc-url".into()),
        )?;
        let rpc_url = Url::parse(rpc_url_str).map_err(|_| {
            OrderbookAppSettingsParseError::ChainSettingsFieldInvalid("rpc-url".into())
        })?;
        let settings = ChainSettingsBuilder::default()
            .label(label)
            .rpc_url(rpc_url)
            .build()?;

        Ok(settings)
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct OrderbookSettings {
    label: Option<String>,
    // This should be a u64, but currently typeshare does not support generating ts types from u64
    // See https://github.com/1Password/typeshare/issues/24
    chain_id: u32,
    #[typeshare(typescript(type = "string"))]
    address: Address,
    #[typeshare(typescript(type = "string"))]
    subgraph_url: Url,
}

impl TryFrom<StrictYaml> for OrderbookSettings {
    type Error = OrderbookAppSettingsParseError;

    fn try_from(val: StrictYaml) -> Result<OrderbookSettings, Self::Error> {
        let label = val["label"].as_str().map(|s| s.into());
        let chain_id = val["chain-id"]
            .as_str()
            .ok_or(
                OrderbookAppSettingsParseError::OrderbookSettingsFieldMissing("chain-id".into()),
            )?
            .parse::<u32>()
            .map_err(|_| {
                OrderbookAppSettingsParseError::OrderbookSettingsFieldInvalid("chain-id".into())
            })?;
        let address = val["address"]
            .as_str()
            .ok_or(OrderbookAppSettingsParseError::OrderbookSettingsFieldMissing("address".into()))?
            .parse::<Address>()
            .map_err(|_| {
                OrderbookAppSettingsParseError::OrderbookSettingsFieldInvalid("address".into())
            })?;
        let subgraph_url_str = val["subgraph-url"].as_str().ok_or(
            OrderbookAppSettingsParseError::OrderbookSettingsFieldMissing("subgraph-url".into()),
        )?;
        let subgraph_url = Url::parse(subgraph_url_str).map_err(|_| {
            OrderbookAppSettingsParseError::OrderbookSettingsFieldInvalid("subgraph-url".into())
        })?;
        let settings = OrderbookSettingsBuilder::default()
            .label(label)
            .chain_id(chain_id)
            .address(address)
            .subgraph_url(subgraph_url)
            .build()?;

        Ok(settings)
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Default)]
pub struct OrderbookAppSettings {
    pub chains: Vec<ChainSettings>,
    pub orderbooks: Vec<OrderbookSettings>,
}

/// Parse string of settings yaml into OrderbookAppSettings
/// Text MUST be strict yaml of the following structure
///
/// ```yaml
/// chains:
///   - label: Polygon Public RPC
///     rpc-url: https://polygon-rpc.com/
///   - rpc-url: https://polygon-mainnet.infura.io/v3/942704a709db49d4832df3c02740f404
///
/// orderbooks:
///   - label: My cool orderbook
///     chain-id: 137
///     address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5a6c
///     subgraph-url: https://api.thegraph.com/subgraphs/name/siddharth2207/obv3subparser
///     fork-block-number: 53678763
///
///   - chain-id: 137
///     address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5a6c
///     subgraph-url: https://api.thegraph.com/subgraphs/name/siddharth2207/obv3subparser
///     label: Another orderbook
/// ```
impl TryFrom<String> for OrderbookAppSettings {
    type Error = OrderbookAppSettingsParseError;

    fn try_from(val: String) -> Result<OrderbookAppSettings, Self::Error> {
        // Parse strict yaml
        let yaml_vec = StrictYamlLoader::load_from_str(val.as_str())
            .map_err(OrderbookAppSettingsParseError::YamlInvalid)?;
        let maybe_yaml = yaml_vec.first();

        match maybe_yaml {
            // Yaml has no contents, return empty settings
            None => Ok(OrderbookAppSettings::default()),

            // Yaml has contents, parse them
            Some(yaml) => {
                let chains: Vec<ChainSettings> = yaml["chains"]
                    .as_vec()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.clone().try_into())
                    .collect::<Result<Vec<ChainSettings>, Self::Error>>()?;
                let orderbooks: Vec<OrderbookSettings> = yaml["orderbooks"]
                    .as_vec()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.clone().try_into())
                    .collect::<Result<Vec<OrderbookSettings>, Self::Error>>()?;
                let settings = OrderbookAppSettingsBuilder::default()
                    .chains(chains)
                    .orderbooks(orderbooks)
                    .build()?;

                Ok(settings)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_settings_from_yaml() {
        let settings_text: String = "
chains:
  - label: Polygon Public RPC
    rpc-url: https://polygon-rpc.com/
  - rpc-url: https://polygon-mainnet.infura.io/v3/abcdefg

orderbooks:
  - label: My cool orderbook
    chain-id: 137
    address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5111
    subgraph-url: https://api.thegraph.com/subgraphs/name/myname/parser1

  - address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5222
    chain-id: 1
    subgraph-url: https://api.thegraph.com/subgraphs/name/myname/parser2

  - address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5333
    chain-id: 1337
    subgraph-url: https://api.thegraph.com/subgraphs/name/myname/parser3
    label: A 3rd orderbook
"
        .into();

        let settings: OrderbookAppSettings = settings_text.try_into().unwrap();

        assert_eq!(settings.chains.len(), 2);
        assert_eq!(settings.orderbooks.len(), 3);

        assert_eq!(settings.chains[0].label, Some("Polygon Public RPC".into()));
        assert_eq!(
            settings.chains[0].rpc_url,
            Url::parse("https://polygon-rpc.com/").unwrap()
        );
        assert_eq!(settings.chains[1].label, None);
        assert_eq!(
            settings.chains[1].rpc_url,
            Url::parse("https://polygon-mainnet.infura.io/v3/abcdefg").unwrap()
        );

        assert_eq!(
            settings.orderbooks[0].label,
            Some("My cool orderbook".into())
        );
        assert_eq!(settings.orderbooks[0].chain_id, 137);
        assert_eq!(
            settings.orderbooks[0].address,
            "0xde5abe2837bc042397d80e37fb7b2c850a8d5111"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.orderbooks[0].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/parser1").unwrap()
        );

        assert_eq!(settings.orderbooks[1].label, None);
        assert_eq!(settings.orderbooks[1].chain_id, 1);
        assert_eq!(
            settings.orderbooks[1].address,
            "0xde5abe2837bc042397d80e37fb7b2c850a8d5222"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.orderbooks[1].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/parser2").unwrap()
        );

        assert_eq!(settings.orderbooks[2].label, Some("A 3rd orderbook".into()));
        assert_eq!(settings.orderbooks[2].chain_id, 1337);
        assert_eq!(
            settings.orderbooks[2].address,
            "0xde5abe2837bc042397d80e37fb7b2c850a8d5333"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.orderbooks[2].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/parser3").unwrap()
        );
    }

    #[test]
    fn empty_settings() {
        let settings_text: String = "
chains:
orderbooks:
"
        .into();

        let settings: OrderbookAppSettings = settings_text.try_into().unwrap();
        assert_eq!(settings.chains.len(), 0);
        assert_eq!(settings.orderbooks.len(), 0);

        let settings_text: String = "".into();
        let settings: OrderbookAppSettings = settings_text.try_into().unwrap();
        assert_eq!(settings.chains.len(), 0);
        assert_eq!(settings.orderbooks.len(), 0);
    }

    #[test]
    fn missing_chain_field_rpc_url() {
        let settings_text: String = "
chains:
    - label: abcd
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn inavalid_chain_field_rpc_url() {
        let settings_text: String = "
chains:
    - label: abcd
      rpc-url: abcdef
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn missing_orderbook_field_subgraph_url() {
        let settings_text: String = "
orderbooks:
    - chain-id: 137
      address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5111
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn missing_orderbook_field_address() {
        let settings_text: String = "
orderbooks:
    - chain-id: 137
      subgraph-url: https://mysubgraph.com
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn missing_orderbook_field_chain_id() {
        let settings_text: String = "
orderbooks:
  - subgraph-url: https://mysubgraph.com
    address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5111
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn invalid_orderbook_field_subgraph_url() {
        let settings_text: String = "
orderbooks:
    - subgraph-url: abcdef
      chain-id: 137
      address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5111
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn invalid_orderbook_field_address() {
        let settings_text: String = "
orderbooks:
  - subgraph-url: https://mysubgraph.com
    chain-id: 137
    address: xyz
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn invalid_orderbook_field_chain_id() {
        let settings_text: String = "
orderbooks:
  - subgraph-url: https://mysubgraph.com
    chain-id: abcdef
    address: 0xde5abe2837bc042397d80e37fb7b2c850a8d5111
"
        .into();
        let settings: Result<OrderbookAppSettings, OrderbookAppSettingsParseError> =
            settings_text.try_into();
        assert!(settings.is_err());
    }
}
