use crate::{
    deposit::DepositError, dotrain_order::DotrainOrderError, meta::TryDecodeRainlangSourceError,
    transaction::WritableTransactionExecuteError,
};
use alloy::{
    hex::FromHexError,
    primitives::{ruint::ParseError, ParseSignedError},
};
use rain_orderbook_app_settings::yaml::{orderbook::OrderbookYaml, YamlError, YamlParsable};
use rain_orderbook_subgraph_client::{MultiSubgraphArgs, OrderbookSubgraphClientError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use tsify::Tsify;
use url::Url;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

pub mod add_orders;
pub mod orders;
pub mod remove_orders;
pub mod trades;
pub mod transactions;
pub mod vaults;

/// RaindexClient provides a simplified interface for querying orderbook data across
/// multiple blockchain networks with automatic configuration management.
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
    /// # Parameters
    ///
    /// - `ob_yamls` - Vector of YAML configuration strings
    /// The YAML files must match the orderbook yaml [spec]()
    ///
    /// # Returns
    ///
    /// - `Ok(RaindexClient)` - Initialized client instance for further operations
    /// - `Err(RaindexError)` - For YAML parsing or initialization errors
    ///
    /// # Examples
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
    #[wasm_export(js_name = "new", preserve_js_class)]
    pub fn new(
        ob_yamls: Vec<String>,
        validate: Option<bool>,
    ) -> Result<RaindexClient, RaindexError> {
        let orderbook_yaml = OrderbookYaml::new(ob_yamls, validate.unwrap_or(false))?;
        Ok(RaindexClient { orderbook_yaml })
    }

    fn get_multi_subgraph_args(
        &self,
        chain_id: Option<u64>,
    ) -> Result<BTreeMap<u64, MultiSubgraphArgs>, RaindexError> {
        let result = match chain_id {
            Some(id) => {
                let network = self.orderbook_yaml.get_network_by_chain_id(id)?;
                let orderbook = self
                    .orderbook_yaml
                    .get_orderbook_by_network_key(&network.key)?;
                HashMap::from([(
                    id,
                    MultiSubgraphArgs {
                        url: orderbook.subgraph.url.clone(),
                        name: network.label.clone().unwrap_or(network.key.clone()),
                    },
                )])
            }
            None => {
                let mut multi_subgraph_args = HashMap::new();
                let networks = self.orderbook_yaml.get_networks()?;

                for network in networks.values() {
                    let orderbook = self
                        .orderbook_yaml
                        .get_orderbook_by_network_key(&network.key)?;
                    multi_subgraph_args.insert(
                        network.chain_id,
                        MultiSubgraphArgs {
                            url: orderbook.subgraph.url.clone(),
                            name: network.label.clone().unwrap_or(network.key.clone()),
                        },
                    );
                }

                if multi_subgraph_args.is_empty() {
                    return Err(RaindexError::NoNetworksConfigured);
                }

                multi_subgraph_args
            }
        };

        Ok(result.into_iter().collect::<BTreeMap<_, _>>())
    }

    fn get_subgraph_url_for_chain(&self, chain_id: u64) -> Result<Url, RaindexError> {
        let network = self.orderbook_yaml.get_network_by_chain_id(chain_id)?;
        let orderbook = self
            .orderbook_yaml
            .get_orderbook_by_network_key(&network.key)?;

        Ok(orderbook.subgraph.url.clone())
    }

    fn get_rpc_url_for_chain(&self, chain_id: u64) -> Result<Url, RaindexError> {
        let network = self.orderbook_yaml.get_network_by_chain_id(chain_id)?;
        Ok(network.rpc.clone())
    }
}

#[derive(Error, Debug)]
pub enum RaindexError {
    #[error("Invalid yaml configuration")]
    InvalidYamlConfig,
    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u64),
    #[error("No networks configured")]
    NoNetworksConfigured,
    #[error("Subgraph not configured for chain ID: {0}")]
    SubgraphNotConfigured(String),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    SerdeError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
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

    pub fn get_test_yaml(subgraph1: &str, subgraph2: &str, rpc1: &str, rpc2: &str) -> String {
        format!(
            r#"
version: {spec_version}
networks:
    mainnet:
        rpc: {rpc1}
        chain-id: 1
        label: Ethereum Mainnet
        network-id: 1
        currency: ETH
    polygon:
        rpc: {rpc2}
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
        use rain_orderbook_app_settings::yaml::YamlError;
        use url::Url;
        use wasm_bindgen_test::wasm_bindgen_test;

        fn get_invalid_yaml() -> String {
            format!(
                r#"
    version: {spec_version}
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
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
        fn test_get_subgraph_url_for_chain_success() {
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

            let url = client.get_subgraph_url_for_chain(1).unwrap();
            assert_eq!(url, Url::parse("http://localhost:3000/sg1").unwrap());

            let url = client.get_subgraph_url_for_chain(137).unwrap();
            assert_eq!(url, Url::parse("http://localhost:3000/sg2").unwrap());
        }

        #[wasm_bindgen_test]
        fn test_get_subgraph_url_for_chain_not_found() {
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

            let err = client.get_subgraph_url_for_chain(999).unwrap_err();
            assert!(
                matches!(err, RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg == "network with chain-id: 999")
            );
            assert!(err.to_readable_msg().contains("network with chain-id: 999"));
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

            let args = client.get_multi_subgraph_args(Some(1)).unwrap();
            assert_eq!(args.len(), 1);
            assert_eq!(
                args.get(&1).unwrap().url,
                Url::parse("http://localhost:3000/sg1").unwrap()
            );
            assert_eq!(args.get(&1).unwrap().name, "Ethereum Mainnet");
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

            let urls: Vec<&str> = args.iter().map(|(_, arg)| arg.url.as_str()).collect();
            assert!(urls.contains(&"http://localhost:3000/sg1"));
            assert!(urls.contains(&"http://localhost:3000/sg2"));

            let names: Vec<&str> = args.iter().map(|(_, arg)| arg.name.as_str()).collect();
            assert!(names.contains(&"Ethereum Mainnet"));
            assert!(names.contains(&"Polygon Mainnet"));
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

            let err = client.get_multi_subgraph_args(Some(999)).unwrap_err();
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
            rpc: https://isolated.rpc
            chain-id: 999
        some-network:
            rpc: https://some-network.rpc
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
