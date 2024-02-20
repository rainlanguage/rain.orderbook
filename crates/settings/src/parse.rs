use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use strict_yaml_rust::{scanner::ScanError, StrictYaml, StrictYamlLoader};
use thiserror::Error;
use typeshare::typeshare;
use url::Url;

#[derive(Error, Debug)]
pub enum AppSettingsParseError {
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
    AppSettingsBuilder(#[from] AppSettingsBuilderError),
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
    orderbooks: Vec<OrderbookSettings>,
}

impl TryFrom<StrictYaml> for ChainSettings {
    type Error = AppSettingsParseError;

    fn try_from(val: StrictYaml) -> Result<ChainSettings, Self::Error> {
        let label = val["label"].as_str().map(|s| s.into());
        let rpc_url_str =
            val["rpc-url"]
                .as_str()
                .ok_or(AppSettingsParseError::ChainSettingsFieldMissing(
                    "rpc-url".into(),
                ))?;
        let rpc_url = Url::parse(rpc_url_str)
            .map_err(|_| AppSettingsParseError::ChainSettingsFieldInvalid("rpc-url".into()))?;
        let orderbooks: Vec<OrderbookSettings> = val["orderbooks"]
            .as_vec()
            .unwrap_or(&vec![])
            .iter()
            .map(|o| OrderbookSettings::try_from(o.clone()))
            .collect::<Result<Vec<OrderbookSettings>, AppSettingsParseError>>()?;

        Ok(ChainSettingsBuilder::default()
            .label(label)
            .rpc_url(rpc_url)
            .orderbooks(orderbooks)
            .build()?)
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
pub struct OrderbookSettings {
    label: Option<String>,
    #[typeshare(typescript(type = "string"))]
    address: Address,
    #[typeshare(typescript(type = "string"))]
    subgraph_url: Url,
}

impl TryFrom<StrictYaml> for OrderbookSettings {
    type Error = AppSettingsParseError;

    fn try_from(val: StrictYaml) -> Result<OrderbookSettings, Self::Error> {
        let label = val["label"].as_str().map(|s| s.into());
        let address = val["address"]
            .as_str()
            .ok_or(AppSettingsParseError::OrderbookSettingsFieldMissing(
                "address".into(),
            ))?
            .parse::<Address>()
            .map_err(|_| AppSettingsParseError::OrderbookSettingsFieldInvalid("address".into()))?;
        let subgraph_url_str = val["subgraph-url"].as_str().ok_or(
            AppSettingsParseError::OrderbookSettingsFieldMissing("subgraph-url".into()),
        )?;
        let subgraph_url = Url::parse(subgraph_url_str).map_err(|_| {
            AppSettingsParseError::OrderbookSettingsFieldInvalid("subgraph-url".into())
        })?;

        Ok(OrderbookSettingsBuilder::default()
            .label(label)
            .address(address)
            .subgraph_url(subgraph_url)
            .build()?)
    }
}

/// Parse string of settings yaml into AppSettings
/// Text MUST be strict yaml of the following structure
///
/// ```yaml
/// chains:
///   - rpc-url: https://eth.llamarpc.com
///     orderbooks:
///       - address: 0x0000000000000000000000000000000000000001
///         subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph1
///
///   - label: Polygon Infura
///     rpc-url: https://polygon-mainnet.infura.io/v3/abcd
///     orderbooks:
///       - label: My special orderbook
///         address: 0x0000000000000000000000000000000000000002
///         subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph2
///
///       - address: 0x0000000000000000000000000000000000000001
///         subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph3
/// ```
#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Builder, Default)]
pub struct AppSettings {
    chains: Vec<ChainSettings>,
}

impl TryFrom<String> for AppSettings {
    type Error = AppSettingsParseError;

    fn try_from(val: String) -> Result<AppSettings, Self::Error> {
        // Parse strict yaml
        let yaml_vec = StrictYamlLoader::load_from_str(val.as_str())
            .map_err(AppSettingsParseError::YamlInvalid)?;
        let maybe_yaml = yaml_vec.first();

        match maybe_yaml {
            // Yaml has no contents, return empty settings
            None => Ok(AppSettings::default()),

            // Yaml has contents, parse them
            Some(yaml) => {
                let chains: Vec<ChainSettings> = yaml["chains"]
                    .as_vec()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.clone().try_into())
                    .collect::<Result<Vec<ChainSettings>, Self::Error>>()?;

                Ok(AppSettingsBuilder::default().chains(chains).build()?)
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
  - label: Polygon Public Rpc
    rpc-url: https://polygon-rpc.com/
    orderbooks:
      - label: My special orderbook
        address: 0x0000000000000000000000000000000000000001
        subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph1

      - address: 0x0000000000000000000000000000000000000002
        subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph2

  - rpc-url: https://eth-rpc.com
    orderbooks:
      - address: 0x0000000000000000000000000000000000000003
        subgraph-url: https://api.thegraph.com/subgraphs/name/myname/mysubgraph3
"
        .into();

        let settings: AppSettings = settings_text.try_into().unwrap();

        assert_eq!(settings.chains.len(), 2);
        assert_eq!(settings.chains[0].label, Some("Polygon Public Rpc".into()));
        assert_eq!(
            settings.chains[0].rpc_url,
            Url::parse("https://polygon-rpc.com/").unwrap()
        );
        assert_eq!(settings.chains[0].orderbooks.len(), 2);
        assert_eq!(
            settings.chains[0].orderbooks[0].label,
            Some("My special orderbook".into())
        );
        assert_eq!(
            settings.chains[0].orderbooks[0].address,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.chains[0].orderbooks[0].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/mysubgraph1").unwrap()
        );
        assert_eq!(settings.chains[0].orderbooks[1].label, None);
        assert_eq!(
            settings.chains[0].orderbooks[1].address,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.chains[0].orderbooks[1].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/mysubgraph2").unwrap()
        );
        assert_eq!(settings.chains[1].label, None);
        assert_eq!(
            settings.chains[1].rpc_url,
            Url::parse("https://eth-rpc.com").unwrap()
        );
        assert_eq!(
            settings.chains[1].orderbooks[0].address,
            "0x0000000000000000000000000000000000000003"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            settings.chains[1].orderbooks[0].subgraph_url,
            Url::parse("https://api.thegraph.com/subgraphs/name/myname/mysubgraph3").unwrap()
        );
    }

    #[test]
    fn empty_settings() {
        let settings_text: String = "
chains:
"
        .into();

        let settings: AppSettings = settings_text.try_into().unwrap();
        assert_eq!(settings.chains.len(), 0);

        let settings_text: String = "".into();
        let settings: AppSettings = settings_text.try_into().unwrap();
        assert_eq!(settings.chains.len(), 0);
    }

    #[test]
    fn missing_chain_field_rpc_url() {
        let settings_text: String = "
chains:
    - label: abcd
"
        .into();
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
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
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn missing_orderbook_field_subgraph_url() {
        let settings_text: String = "
chains:
  - label: abcd
    rpc-url: https://rpc.com
    orderbooks:
      - address: 0x0000000000000000000000000000000000000003
"
        .into();
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn missing_orderbook_field_address() {
        let settings_text: String = "
chains:
  - label: abcd
    rpc-url: https://rpc.com
    orderbooks:
      - subgraph-url: https://mysubgraph.com
"
        .into();
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn invalid_orderbook_field_subgraph_url() {
        let settings_text: String = "
chains:
  - label: abcd
    rpc-url: https://rpc.com
    orderbooks:
      - address: 0x0000000000000000000000000000000000000003
        subgraph-url: abc
"
        .into();
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
        assert!(settings.is_err());
    }

    #[test]
    fn invalid_orderbook_field_address() {
        let settings_text: String = "
chains:
  - label: abcd
    rpc-url: https://rpc.com
    orderbooks:
      - address: abc
        subgraph-url: https://mysubgraph.com
"
        .into();
        let settings: Result<AppSettings, AppSettingsParseError> = settings_text.try_into();
        assert!(settings.is_err());
    }
}
