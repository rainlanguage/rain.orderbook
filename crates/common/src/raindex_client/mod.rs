use crate::{
    add_order::AddOrderArgsError, deposit::DepositError, dotrain_order::DotrainOrderError,
    erc20::Error as Erc20Error, meta::TryDecodeRainlangSourceError,
    transaction::WritableTransactionExecuteError, utils::amount_formatter::AmountFormatterError,
};
use alloy::{
    hex::FromHexError,
    primitives::{
        ruint::{FromUintError, ParseError},
        Address, ParseSignedError,
    },
};
use rain_orderbook_app_settings::{
    new_config::ParseConfigError,
    yaml::{orderbook::OrderbookYaml, YamlError, YamlParsable},
};
use rain_orderbook_subgraph_client::{
    types::order_detail_traits::OrderDetailError, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use thiserror::Error;
use tsify::Tsify;
use url::Url;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

pub mod add_orders;
pub mod order_quotes;
pub mod orders;
pub mod remove_orders;
pub mod trades;
pub mod transactions;
pub mod vaults;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct ChainIds(#[tsify(type = "number[]")] pub Vec<u32>);
impl_wasm_traits!(ChainIds);

/// RaindexClient provides a simplified interface for querying orderbook data across
/// multiple networks with automatic configuration management.
///
/// This client abstracts away complex network-specific configurations by parsing YAML
/// configuration files that define networks, tokens, orderbooks, and subgraph endpoints.
/// It enables querying orderbook data either from specific chains or across all
/// configured networks with automatic fallback mechanisms.
///
/// The client handles:
/// - YAML configuration parsing and validation
/// - Network-to-subgraph URL mapping
/// - Multi-network query coordination
/// - Chain ID resolution to network configurations
///
/// ## Examples
///
/// ```javascript
/// const result = await RaindexClient.new([yamlConfig]);
/// if (result.error) {
///   console.error('Failed to create client:', result.error.readableMsg);
/// } else {
///   const client = result.value;
///   // Query orders across all networks or specific chains
/// }
///
/// // Create client with multiple YAML files for modular configuration
/// const result = await RaindexClient.new([
///   yamlConfig1,
///   yamlConfig2,
///   yamlConfig3,
/// ]);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexClient {
    orderbook_yaml: OrderbookYaml,
}

#[wasm_export]
impl RaindexClient {
    /// Constructor that creates and returns RaindexClient instance directly
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Single YAML file
    /// const result = await RaindexClient.new([yamlConfig]);
    /// if (result.error) {
    ///   console.error("Init failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const client = result.value;
    ///
    /// // Multiple YAML files (for modular configuration)
    /// const result = await RaindexClient.new([networksYaml, orderbooksYaml, tokensYaml]);
    /// ```
    #[wasm_export(
        js_name = "new",
        return_description = "Initialized client instance for further operations",
        preserve_js_class
    )]
    pub fn new(
        #[wasm_export(
            js_name = "obYamls",
            param_description = "List of YAML configuration strings. \
            The YAML files must match the [orderbook yaml spec](https://github.com/rainlanguage/specs/blob/main/ob-yaml.md)
            "
        )]
        ob_yamls: Vec<String>,
        validate: Option<bool>,
    ) -> Result<RaindexClient, RaindexError> {
        let orderbook_yaml = OrderbookYaml::new(ob_yamls, validate.unwrap_or(false))?;
        Ok(RaindexClient { orderbook_yaml })
    }

    fn get_multi_subgraph_args(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<BTreeMap<u32, Vec<MultiSubgraphArgs>>, RaindexError> {
        let result = match chain_ids {
            Some(ids) if !ids.is_empty() => {
                let mut multi_subgraph_args = BTreeMap::new();
                for id in ids {
                    let network = self.orderbook_yaml.get_network_by_chain_id(id)?;
                    let orderbooks = self
                        .orderbook_yaml
                        .get_orderbooks_by_network_key(&network.key)?;
                    for orderbook in orderbooks {
                        multi_subgraph_args.entry(id).or_insert(Vec::new()).push(
                            MultiSubgraphArgs {
                                url: orderbook.subgraph.url.clone(),
                                name: network.label.clone().unwrap_or(network.key.clone()),
                            },
                        );
                    }
                }
                multi_subgraph_args
            }
            Some(_) | None => {
                let mut multi_subgraph_args = BTreeMap::new();
                let networks = self.orderbook_yaml.get_networks()?;

                for network in networks.values() {
                    let orderbooks = self
                        .orderbook_yaml
                        .get_orderbooks_by_network_key(&network.key)?;
                    for orderbook in orderbooks {
                        multi_subgraph_args
                            .entry(network.chain_id)
                            .or_insert(Vec::new())
                            .push(MultiSubgraphArgs {
                                url: orderbook.subgraph.url.clone(),
                                name: network.label.clone().unwrap_or(network.key.clone()),
                            });
                    }
                }
                multi_subgraph_args
            }
        };

        if result.is_empty() {
            return Err(RaindexError::NoNetworksConfigured);
        }
        Ok(result)
    }

    #[wasm_export(skip)]
    pub fn get_orderbook_client(
        &self,
        orderbook_address: Address,
    ) -> Result<OrderbookSubgraphClient, RaindexError> {
        let orderbook = self
            .orderbook_yaml
            .get_orderbook_by_address(orderbook_address)?;
        Ok(OrderbookSubgraphClient::new(orderbook.subgraph.url.clone()))
    }

    fn get_rpc_urls_for_chain(&self, chain_id: u32) -> Result<Vec<Url>, RaindexError> {
        let network = self.orderbook_yaml.get_network_by_chain_id(chain_id)?;
        Ok(network.rpcs.clone())
    }
}

#[derive(Error, Debug)]
pub enum RaindexError {
    #[error("Invalid yaml configuration")]
    InvalidYamlConfig,
    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u32),
    #[error("No networks configured")]
    NoNetworksConfigured,
    #[error("Subgraph not configured for chain ID: {0}")]
    SubgraphNotConfigured(String),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    SerdeError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    DotrainOrderError(Box<DotrainOrderError>),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    OrderbookSubgraphClientError(#[from] OrderbookSubgraphClientError),
    #[error(transparent)]
    TryDecodeRainlangSourceError(#[from] TryDecodeRainlangSourceError),
    #[error(transparent)]
    U256ParseError(#[from] ParseError),
    #[error(transparent)]
    I256ParseError(#[from] ParseSignedError),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,
    #[error("Zero amount")]
    ZeroAmount,
    #[error("Existing allowance")]
    ExistingAllowance,
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),
    #[error(transparent)]
    DepositArgsError(#[from] DepositError),
    #[error("Orderbook not found for address: {0} on chain ID: {1}")]
    OrderbookNotFound(String, u32),
    #[error(transparent)]
    OrderDetailError(#[from] OrderDetailError),
    #[error(transparent)]
    ParseConfigError(#[from] ParseConfigError),
    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
    #[error(transparent)]
    OrderbookQuoteError(#[from] rain_orderbook_quote::error::Error),
    #[error("Missing subgraph {0} for order {1}")]
    SubgraphNotFound(String, String),
    #[error("Invalid vault balance change type: {0}")]
    InvalidVaultBalanceChangeType(String),
    #[error(transparent)]
    AmountFormatterError(#[from] AmountFormatterError),
    #[error(transparent)]
    Erc20Error(Box<Erc20Error>),
    #[error(transparent)]
    FromUint8Error(#[from] FromUintError<u8>),
}

impl From<DotrainOrderError> for RaindexError {
    fn from(err: DotrainOrderError) -> Self {
        Self::DotrainOrderError(Box::new(err))
    }
}

impl From<Erc20Error> for RaindexError {
    fn from(err: Erc20Error) -> Self {
        Self::Erc20Error(Box::new(err))
    }
}

impl RaindexError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            RaindexError::InvalidYamlConfig => {
                "The YAML configuration is invalid. Please check your configuration.".to_string()
            }
            RaindexError::ChainIdNotFound(chain_id) => format!(
                "The chain ID '{}' was not found in the configuration.",
                chain_id
            ),
            RaindexError::NoNetworksConfigured => {
                "No networks configured. Please check your configuration.".to_string()
            }
            RaindexError::SubgraphNotConfigured(chain_id) => {
                format!("No subgraph is configured for chain ID '{}'.", chain_id)
            }
            RaindexError::YamlError(err) => format!(
                "YAML configuration parsing failed: {}. Check file syntax and structure.",
                err
            ),
            RaindexError::SerdeError(err) => format!(
                "Data conversion failed: {}. The data format may be incompatible.",
                err
            ),
            RaindexError::DotrainOrderError(err) => format!(
                "Order configuration is invalid: {}. Please check the order parameters and format.",
                err
            ),
            RaindexError::FromHexError(err) => {
                format!(
                    "Invalid address format: {}. Please provide a valid hexadecimal address.",
                    err
                )
            }
            RaindexError::OrderbookSubgraphClientError(err) => {
                format!("Failed to query subgraph: {}. Check network connection and subgraph availability.", err)
            }
            RaindexError::TryDecodeRainlangSourceError(err) => {
                format!("Failed to decode Rainlang source: {}. The source code may be corrupted or incompatible.", err)
            }
            RaindexError::U256ParseError(err) => {
                format!(
                    "Invalid number format: {}. Please provide a valid numeric value.",
                    err
                )
            }
            RaindexError::I256ParseError(err) => {
                format!(
                    "Invalid number format: {}. Please provide a valid numeric value.",
                    err
                )
            }
            RaindexError::JsError(err) => format!("JavaScript error: {}", err),
            RaindexError::ReadLockError => {
                "Failed to read the YAML configuration due to a lock error".to_string()
            }
            RaindexError::WriteLockError => {
                "Failed to modify the YAML configuration due to a lock error".to_string()
            }
            RaindexError::ZeroAmount => "Amount cannot be zero".to_string(),
            RaindexError::WritableTransactionExecuteError(err) => {
                format!("Failed to execute transaction: {}", err)
            }
            RaindexError::ExistingAllowance => {
                "There is already an allowance for this vault".to_string()
            }
            RaindexError::DepositArgsError(err) => {
                format!("Failed to create deposit arguments: {}", err)
            }
            RaindexError::OrderbookNotFound(address, chain_id) => {
                format!(
                    "Orderbook not found for address: {} on chain ID: {}",
                    address, chain_id
                )
            }
            RaindexError::OrderDetailError(err) => {
                format!("Failed to decode order detail: {}", err)
            }
            RaindexError::ParseConfigError(err) => {
                format!("Failed to parse yaml sources for configuration: {}", err)
            }
            RaindexError::AddOrderArgsError(e) => {
                format!("Failed to prepare the add order calldata: {}", e)
            }
            RaindexError::OrderbookQuoteError(err) => {
                format!("Failed to get order quote: {}", err)
            }
            RaindexError::SubgraphNotFound(subgraph, order) => {
                format!(
                    "Subgraph with name '{}' not found for the order with hash '{}'",
                    subgraph, order
                )
            }
            RaindexError::InvalidVaultBalanceChangeType(typ) => {
                format!("Invalid vault balance change type: {}", typ)
            }
            RaindexError::AmountFormatterError(err) => {
                format!("There was a problem formatting the amount: {}", err)
            }
            RaindexError::Erc20Error(err) => {
                format!("There was an error with the ERC20 token: {}", err)
            }
            RaindexError::FromUint8Error(err) => {
                format!("There was an error converting from u8 number: {}", err)
            }
        }
    }
}

impl From<RaindexError> for JsValue {
    fn from(value: RaindexError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<RaindexError> for WasmEncodedError {
    fn from(value: RaindexError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;

    pub const CHAIN_ID_1_ORDERBOOK_ADDRESS: &str = "0x1234567890123456789012345678901234567890";
    pub fn get_test_yaml(subgraph1: &str, subgraph2: &str, rpc1: &str, rpc2: &str) -> String {
        format!(
            r#"
version: {spec_version}
networks:
    mainnet:
        rpcs:
            - {rpc1}
        chain-id: 1
        label: Ethereum Mainnet
        network-id: 1
        currency: ETH
    polygon:
        rpcs:
            - {rpc2}
        chain-id: 137
        label: Polygon Mainnet
        network-id: 137
        currency: MATIC
subgraphs:
    mainnet: {subgraph1}
    polygon: {subgraph2}
metaboards:
    mainnet: https://api.thegraph.com/subgraphs/name/xyz
    polygon: https://api.thegraph.com/subgraphs/name/polygon
orderbooks:
    mainnet-orderbook:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        subgraph: mainnet
        label: Primary Orderbook
    polygon-orderbook:
        address: 0x0987654321098765432109876543210987654321
        network: polygon
        subgraph: polygon
        label: Polygon Orderbook
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
        label: Wrapped Ether
        symbol: WETH
    usdc:
        network: polygon
        address: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
        decimals: 6
        label: USD Coin
        symbol: USDC
deployers:
    mainnet-deployer:
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
        network: mainnet
"#,
            spec_version = SpecVersion::current()
        )
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use alloy::primitives::Address;
        use rain_orderbook_app_settings::yaml::YamlError;
        use std::str::FromStr;
        use url::Url;
        use wasm_bindgen_test::wasm_bindgen_test;

        fn get_invalid_yaml() -> String {
            format!(
                r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    orderbooks:
        invalid-orderbook:
            address: 0x1234567890123456789012345678901234567890
            network: nonexistent-network
            subgraph: nonexistent-subgraph
    "#,
                spec_version = SpecVersion::current()
            )
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_success() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();
            assert!(!client.orderbook_yaml.documents.is_empty());
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_invalid_yaml() {
            let err = RaindexClient::new(vec![get_invalid_yaml()], Some(true)).unwrap_err();
            assert!(matches!(
                err,
                RaindexError::YamlError(YamlError::Field { .. })
            ));
            assert!(err
                .to_readable_msg()
                .contains("YAML configuration parsing failed"));
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_empty_yaml() {
            let err = RaindexClient::new(vec!["".to_string()], None).unwrap_err();
            assert!(matches!(err, RaindexError::YamlError(YamlError::EmptyFile)));
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_single_chain() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(Some(vec![1])).unwrap();
            assert_eq!(args.len(), 1);
            assert_eq!(
                args.get(&1).unwrap()[0].url,
                Url::parse("http://localhost:3000/sg1").unwrap()
            );
            assert_eq!(args.get(&1).unwrap()[0].name, "Ethereum Mainnet");
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_all_chains() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(None).unwrap();
            assert_eq!(args.len(), 2);

            let urls: Vec<&str> = args.iter().map(|(_, arg)| arg[0].url.as_str()).collect();
            assert!(urls.contains(&"http://localhost:3000/sg1"));
            assert!(urls.contains(&"http://localhost:3000/sg2"));

            let names: Vec<&str> = args.iter().map(|(_, arg)| arg[0].name.as_str()).collect();
            assert!(names.contains(&"Ethereum Mainnet"));
            assert!(names.contains(&"Polygon Mainnet"));
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_multiple_chains() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(Some(vec![1, 137])).unwrap();
            assert_eq!(args.len(), 2);

            let args1 = args.get(&1).unwrap();
            assert_eq!(args1.len(), 1);
            assert_eq!(
                args1[0].url,
                Url::parse("http://localhost:3000/sg1").unwrap()
            );
            assert_eq!(args1[0].name, "Ethereum Mainnet");

            let args2 = args.get(&137).unwrap();
            assert_eq!(args2.len(), 1);
            assert_eq!(
                args2[0].url,
                Url::parse("http://localhost:3000/sg2").unwrap()
            );
            assert_eq!(args2[0].name, "Polygon Mainnet");
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_invalid_chain() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let err = client.get_multi_subgraph_args(Some(vec![999])).unwrap_err();
            assert!(
                matches!(err, RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg.contains("network with chain-id: 999"))
            );
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_no_networks() {
            let yaml = format!(
                r#"
    version: {spec_version}
    networks:
        isolated:
            rpcs:
                - https://isolated.rpc
            chain-id: 999
        some-network:
            rpcs:
                - https://some-network.rpc
            chain-id: 1000
    subgraphs:
        test: https://test.subgraph
    metaboards:
        test: https://test.metaboard
    tokens:
        test-token:
            network: isolated
            address: 0x1111111111111111111111111111111111111111
            decimals: 18
    deployers:
        test-deployer:
            address: 0x2222222222222222222222222222222222222222
            network: isolated
    orderbooks:
        test-orderbook:
            address: 0x1111111111111111111111111111111111111111
            network: some-network
            subgraph: test
            label: Test Orderbook
    "#,
                spec_version = SpecVersion::current()
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let err = client.get_multi_subgraph_args(None).unwrap_err();
            assert!(matches!(
                err,
                RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg.contains("orderbook with network key: isolated")
            ));
            assert!(err
                .to_readable_msg()
                .contains("orderbook with network key: isolated not found"));
        }
    }
}
