use crate::subgraph::SubgraphError;
use rain_orderbook_app_settings::yaml::{orderbook::OrderbookYaml, YamlError, YamlParsable};
use rain_orderbook_common::dotrain_order::DotrainOrderError;
use rain_orderbook_subgraph_client::MultiSubgraphArgs;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

pub mod orders;
pub mod vaults;

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
    /// - `sources` - Vector of YAML configuration strings
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
    pub async fn new(sources: Vec<String>) -> Result<RaindexClient, RaindexError> {
        let orderbook_yaml = OrderbookYaml::new(sources, true)?;
        Ok(RaindexClient { orderbook_yaml })
    }

    #[wasm_export(skip)]
    pub fn get_multi_subgraph_args(
        &self,
        chain_id: Option<u64>,
    ) -> Result<Vec<MultiSubgraphArgs>, RaindexError> {
        match chain_id {
            Some(id) => {
                let network = self.orderbook_yaml.get_network_by_chain_id(id)?;
                let orderbook = self
                    .orderbook_yaml
                    .get_orderbook_by_network_key(&network.key)?;
                Ok(vec![MultiSubgraphArgs {
                    url: orderbook.subgraph.url.clone(),
                    name: network.label.clone().unwrap_or(network.key.clone()),
                }])
            }
            None => {
                let mut multi_subgraph_args = Vec::new();
                let networks = self.orderbook_yaml.get_networks()?;

                for network in networks.values() {
                    let orderbook = self
                        .orderbook_yaml
                        .get_orderbook_by_network_key(&network.key)?;
                    multi_subgraph_args.push(MultiSubgraphArgs {
                        url: orderbook.subgraph.url.clone(),
                        name: network.label.clone().unwrap_or(network.key.clone()),
                    });
                }

                if multi_subgraph_args.is_empty() {
                    return Err(RaindexError::SubgraphNotConfigured(
                        "no networks".to_string(),
                    ));
                }

                Ok(multi_subgraph_args)
            }
        }
    }

    fn get_subgraph_url_for_chain(&self, chain_id: u64) -> Result<String, RaindexError> {
        let network = self.orderbook_yaml.get_network_by_chain_id(chain_id)?;
        let orderbook = self
            .orderbook_yaml
            .get_orderbook_by_network_key(&network.key)?;

        Ok(orderbook.subgraph.url.to_string())
    }
}

#[derive(Error, Debug)]
pub enum RaindexError {
    #[error("Invalid yaml configuration")]
    InvalidYamlConfig,
    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u64),
    #[error("Subgraph not configured for chain ID: {0}")]
    SubgraphNotConfigured(String),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    SerdeError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    SubgraphError(#[from] SubgraphError),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
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
            RaindexError::SubgraphNotConfigured(chain_id) => {
                format!("No subgraph is configured for chain ID '{}'.", chain_id)
            }
            RaindexError::YamlError(err) => format!("YAML configuration error: {}", err),
            RaindexError::SerdeError(err) => format!("Data serialization error: {}", err),
            RaindexError::SubgraphError(err) => format!("Subgraph query error: {}", err),
            RaindexError::DotrainOrderError(err) => format!("Order configuration error: {}", err),
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
    use rain_orderbook_app_settings::{spec_version::SpecVersion, yaml::YamlError};
    use url::Url;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn get_test_yaml() -> String {
        format!(
            r#"
version: {spec_version}
networks:
    mainnet:
        rpc: https://mainnet.infura.io
        chain-id: 1
        label: Ethereum Mainnet
        network-id: 1
        currency: ETH
    polygon:
        rpc: https://polygon-rpc.com
        chain-id: 137
        label: Polygon Mainnet
        network-id: 137
        currency: MATIC
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/xyz
    polygon: https://api.thegraph.com/subgraphs/name/polygon
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
    async fn test_raindex_client_new_success() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();
        assert!(!client.orderbook_yaml.documents.is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_raindex_client_new_invalid_yaml() {
        let err = RaindexClient::new(vec![get_invalid_yaml()])
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            RaindexError::YamlError(YamlError::Field { .. })
        ));
        assert!(err.to_readable_msg().contains("YAML configuration error"));
    }

    #[wasm_bindgen_test]
    async fn test_raindex_client_new_empty_yaml() {
        let err = RaindexClient::new(vec!["".to_string()]).await.unwrap_err();
        assert!(matches!(err, RaindexError::YamlError(YamlError::EmptyFile)));
    }

    #[wasm_bindgen_test]
    async fn test_get_subgraph_url_for_chain_success() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();

        let url = client.get_subgraph_url_for_chain(1).unwrap();
        assert_eq!(url, "https://api.thegraph.com/subgraphs/name/xyz");

        let url = client.get_subgraph_url_for_chain(137).unwrap();
        assert_eq!(url, "https://api.thegraph.com/subgraphs/name/polygon");
    }

    #[wasm_bindgen_test]
    async fn test_get_subgraph_url_for_chain_not_found() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();

        let err = client.get_subgraph_url_for_chain(999).unwrap_err();
        assert!(
            matches!(err, RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg == "network with chain-id: 999")
        );
        assert!(err.to_readable_msg().contains("network with chain-id: 999"));
    }

    #[wasm_bindgen_test]
    async fn test_get_multi_subgraph_args_single_chain() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();

        let args = client.get_multi_subgraph_args(Some(1)).unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(
            args[0].url,
            Url::parse("https://api.thegraph.com/subgraphs/name/xyz").unwrap()
        );
        assert_eq!(args[0].name, "Ethereum Mainnet");
    }

    #[wasm_bindgen_test]
    async fn test_get_multi_subgraph_args_all_chains() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();

        let args = client.get_multi_subgraph_args(None).unwrap();
        assert_eq!(args.len(), 2);

        let urls: Vec<&str> = args.iter().map(|arg| arg.url.as_str()).collect();
        assert!(urls.contains(&"https://api.thegraph.com/subgraphs/name/xyz"));
        assert!(urls.contains(&"https://api.thegraph.com/subgraphs/name/polygon"));

        let names: Vec<&str> = args.iter().map(|arg| arg.name.as_str()).collect();
        assert!(names.contains(&"Ethereum Mainnet"));
        assert!(names.contains(&"Polygon Mainnet"));
    }

    #[wasm_bindgen_test]
    async fn test_get_multi_subgraph_args_invalid_chain() {
        let client = RaindexClient::new(vec![get_test_yaml()]).await.unwrap();

        let err = client.get_multi_subgraph_args(Some(999)).unwrap_err();
        assert!(
            matches!(err, RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg.contains("network with chain-id: 999"))
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_multi_subgraph_args_no_networks() {
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

        let client = RaindexClient::new(vec![yaml]).await.unwrap();

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
