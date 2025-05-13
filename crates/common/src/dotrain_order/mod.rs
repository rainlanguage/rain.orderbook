use crate::GH_COMMIT_SHA;
use crate::{
    add_order::{ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS, ORDERBOOK_ORDER_ENTRYPOINTS},
    rainlang::compose_to_rainlang,
};
use alloy::primitives::Address;
use alloy_ethers_typecast::transaction::{ReadableClient, ReadableClientError};
use dotrain::{error::ComposeError, RainDocument};
use futures::future::join_all;
use rain_interpreter_parser::{ParserError, ParserV2};
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::remote_networks::{ParseRemoteNetworksError, RemoteNetworksCfg};
use rain_orderbook_app_settings::remote_tokens::{ParseRemoteTokensError, RemoteTokensCfg};
use rain_orderbook_app_settings::yaml::cache::Cache;
use rain_orderbook_app_settings::yaml::{
    default_document, dotrain::DotrainYaml, orderbook::OrderbookYaml, YamlError, YamlParsable,
};
use rain_orderbook_app_settings::ParseConfigSourceError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[wasm_bindgen]
pub struct DotrainOrder {
    dotrain: String,
    dotrain_yaml: DotrainYaml,
}

impl PartialEq for DotrainOrder {
    fn eq(&self, other: &Self) -> bool {
        self.dotrain == other.dotrain
    }
}

#[derive(Error, Debug)]
pub enum DotrainOrderError {
    #[error("DotrainOrder is not initialized")]
    DotrainOrderNotInitialized,

    #[error(transparent)]
    ParseConfigSourceError(#[from] ParseConfigSourceError),

    #[error("Scenario {0} not found")]
    ScenarioNotFound(String),

    #[error("Metaboard {0} not found")]
    MetaboardNotFound(String),

    #[error(transparent)]
    ComposeError(#[from] ComposeError),

    #[error(transparent)]
    AuthoringMetaV2Error(#[from] AuthoringMetaV2Error),

    #[error(transparent)]
    FetchAuthoringMetaV2WordError(#[from] FetchAuthoringMetaV2WordError),

    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),

    #[error(transparent)]
    ParserError(#[from] ParserError),

    #[error("{0}")]
    CleanUnusedFrontmatterError(String),

    #[error("Raindex version mismatch: got {1}, should be {0}")]
    RaindexVersionMismatch(String, String),

    #[error("Raindex version missing: should be {0}")]
    MissingRaindexVersion(String),

    #[error("Deployment {0} not found")]
    DeploymentNotFound(String),

    #[error("Order {0} not found")]
    OrderNotFound(String),

    #[error("Token {0} not found")]
    TokenNotFound(String),

    #[error("Invalid index for vault ID")]
    InvalidVaultIdIndex,

    #[error(transparent)]
    YamlError(#[from] YamlError),

    #[error(transparent)]
    ParseRemoteNetworksError(#[from] ParseRemoteNetworksError),
    #[error(transparent)]
    ParseRemoteTokensError(#[from] ParseRemoteTokensError),
}

impl DotrainOrderError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            DotrainOrderError::DotrainOrderNotInitialized => {
                "DotrainOrder is not initialized. Please call initialize() first.".to_string()
            }
            DotrainOrderError::ParseConfigSourceError(e) => {
                format!("Error parsing the configuration source: {}", e)
            }
            DotrainOrderError::ScenarioNotFound(name) => {
                format!("Scenario '{}' is not defined in the configuration.", name)
            }
            DotrainOrderError::MetaboardNotFound(name) => {
                format!("Metaboard configuration for network '{}' is missing.", name)
            }
            DotrainOrderError::ComposeError(e) => {
                format!(
                    "Error composing the Rainlang script from the .rain file: {}",
                    e
                )
            }
            DotrainOrderError::AuthoringMetaV2Error(e) => {
                format!("Error processing contract authoring metadata: {}", e)
            }
            DotrainOrderError::FetchAuthoringMetaV2WordError(e) => {
                format!(
                    "Error fetching words from contract authoring metadata: {}",
                    e
                )
            }
            DotrainOrderError::ReadableClientError(e) => {
                format!("Problem communicating with the rpc: {}", e)
            }
            DotrainOrderError::ParserError(e) => {
                format!("Error parsing the Rainlang script: {}", e)
            }
            DotrainOrderError::CleanUnusedFrontmatterError(e) => {
                format!("Internal configuration processing error: {}", e)
            }
            DotrainOrderError::RaindexVersionMismatch(expected, got) => {
                format!("Configuration Raindex version mismatch. Expected '{}', but found '{}'. Please update 'raindex-version'.", expected, got)
            }
            DotrainOrderError::MissingRaindexVersion(expected) => {
                format!("The required 'raindex-version' field is missing. Please add it and set it to '{}'.", expected)
            }
            DotrainOrderError::DeploymentNotFound(name) => {
                format!("Deployment '{}' is not defined in the configuration.", name)
            }
            DotrainOrderError::OrderNotFound(name) => {
                format!("Order '{}' is not defined in the configuration.", name)
            }
            DotrainOrderError::TokenNotFound(name) => {
                format!("Token '{}' is not defined in the configuration.", name)
            }
            DotrainOrderError::InvalidVaultIdIndex => {
                "Internal error: Invalid index used for vault ID.".to_string()
            }
            DotrainOrderError::YamlError(e) => {
                format!("Error parsing the YAML configuration: {}", e)
            }
            DotrainOrderError::ParseRemoteNetworksError(e) => {
                format!("Error parsing the remote networks configuration: {}", e)
            }
            DotrainOrderError::ParseRemoteTokensError(e) => {
                format!("Error parsing the remote tokens configuration: {}", e)
            }
        }
    }
}

impl From<DotrainOrderError> for JsValue {
    fn from(value: DotrainOrderError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<DotrainOrderError> for WasmEncodedError {
    fn from(value: DotrainOrderError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub enum WordsResult {
    Success(AuthoringMetaV2),
    Error(String),
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ContractWords {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub words: WordsResult,
}

impl From<Result<AuthoringMetaV2, DotrainOrderError>> for WordsResult {
    fn from(result: Result<AuthoringMetaV2, DotrainOrderError>) -> Self {
        match result {
            Ok(meta) => WordsResult::Success(meta),
            Err(err) => WordsResult::Error(err.to_string()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ScenarioWords {
    pub scenario: String,
    pub pragma_words: Vec<ContractWords>,
    pub deployer_words: ContractWords,
}

impl DotrainOrder {
    pub fn dummy() -> Self {
        Self {
            dotrain: "".to_string(),
            dotrain_yaml: DotrainYaml {
                documents: vec![default_document()],
                cache: Cache::new(),
            },
        }
    }
    pub fn is_initialized(&self) -> bool {
        !self.dotrain.is_empty()
    }

    async fn _initialize(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<(String, DotrainYaml), DotrainOrderError> {
        let frontmatter = RainDocument::get_front_matter(&dotrain)
            .unwrap_or("")
            .to_string();

        let mut sources = vec![frontmatter.clone()];
        if let Some(settings) = settings {
            sources.extend(settings);
        }

        let mut orderbook_yaml = OrderbookYaml::new(sources.clone(), false)?;
        let mut dotrain_yaml = DotrainYaml::new(sources.clone(), false)?;

        let remote_networks =
            RemoteNetworksCfg::fetch_networks(orderbook_yaml.get_remote_networks()?).await?;
        if !remote_networks.is_empty() {
            orderbook_yaml
                .cache
                .update_remote_networks(remote_networks.clone());
            dotrain_yaml
                .cache
                .update_remote_networks(remote_networks.clone());
        }

        if let Some(remote_tokens_cfg) = orderbook_yaml.get_remote_tokens()? {
            let networks = orderbook_yaml.get_networks()?;
            let remote_tokens = RemoteTokensCfg::fetch_tokens(&networks, remote_tokens_cfg).await?;
            dotrain_yaml.cache.update_remote_tokens(remote_tokens);
        }

        Ok((dotrain, dotrain_yaml))
    }
}

/*

NOTE FOR DEVELOPERS:

Due to the way `wasm_bindgen` works, and how the `impl_wasm_traits` macro, we must separate the construction and initialization steps for `DotrainOrder`.

- When using the macro, the `DotrainOrder` object created on the JavaScript side is **not** a class instance; it's just a plain object with fields (which will be `undefined`).
- To get a proper class instance in JavaScript, you must use the constructor exposed by `wasm_bindgen` (`new DotrainOrder()`), which gives you a real class object.
- After construction, you must call the `.initialize()` method to populate the instance with your configuration and make it ready for use.

This two-step process is required for correct interop between Rust and JavaScript via WASM.

*/
#[wasm_bindgen]
impl DotrainOrder {
    /// Creates a new, uninitialized `DotrainOrder` instance.
    ///
    /// # JavaScript Usage
    ///
    /// To use `DotrainOrder` from JavaScript, you must first create an instance
    /// using this constructor, and then initialize it with your configuration:
    ///
    /// ```javascript
    /// // Step 1: Create a new DotrainOrder instance (not yet initialized)
    /// const dotrainOrder = new DotrainOrder();
    ///
    /// // Step 2: Initialize the instance with your dotrain script and optional settings
    /// await dotrainOrder.initialize(dotrain, [settings]);
    ///
    /// // Now you can use other methods on dotrainOrder
    /// const rainlang = await dotrainOrder.composeScenarioToRainlang("my-scenario");
    /// ```
    ///
    /// **Note:** The constructor does NOT initialize the instance.  
    /// You must always call `.initialize()` before using any other methods.  
    /// If you try to use methods before initialization, they will throw an error.
    #[wasm_bindgen(constructor)]
    pub fn new() -> DotrainOrder {
        Self::dummy()
    }

    /// Creates a new `DotrainOrder` instance asynchronously.
    ///
    /// **Deprecated:** This method is deprecated and will be removed in a future version.
    /// The preferred way to create and initialize a `DotrainOrder` instance is to
    /// first instantiate it using the constructor `new DotrainOrder()` and then
    /// call the asynchronous `initialize` method.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// // Deprecated usage:
    /// // const dotrainOrder = await DotrainOrder.create(dotrain, [settings]);
    ///
    /// // Preferred usage:
    /// const dotrainOrder = new DotrainOrder();
    /// await dotrainOrder.initialize(dotrain, [settings]);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `dotrain` - A string containing the dotrain script.
    /// * `settings` - An optional vector of strings representing additional configuration settings.
    ///
    /// # See Also
    ///
    /// * [`initialize`](#method.initialize)
    #[wasm_bindgen(js_name = "create")]
    pub async fn create(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<DotrainOrder, DotrainOrderError> {
        let (dotrain, dotrain_yaml) = DotrainOrder::_initialize(dotrain, settings).await?;
        Ok(DotrainOrder {
            dotrain,
            dotrain_yaml,
        })
    }
}

#[wasm_export]
impl DotrainOrder {
    /// Initializes the `DotrainOrder` instance asynchronously with the provided dotrain script and settings.
    ///
    /// This method should be called after creating an instance with `new DotrainOrder()`.
    /// It processes the dotrain script and settings, fetching remote configurations if necessary.
    ///
    /// # Example (JavaScript)
    ///
    /// ```javascript
    /// const dotrainOrder = new DotrainOrder();
    /// await dotrainOrder.initialize(dotrain, [settings]);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `dotrain` - A string containing the dotrain script.
    /// * `settings` - An optional vector of strings representing additional configuration settings.
    #[wasm_export(js_name = "initialize", unchecked_return_type = "void")]
    pub async fn initialize(
        &mut self,
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<(), DotrainOrderError> {
        let (dotrain, dotrain_yaml) = DotrainOrder::_initialize(dotrain, settings).await?;
        self.dotrain = dotrain;
        self.dotrain_yaml = dotrain_yaml;
        Ok(())
    }

    // get this instance's dotrain string
    #[wasm_export(js_name = "dotrain", unchecked_return_type = "string")]
    pub fn dotrain(&self) -> Result<String, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }
        Ok(self.dotrain.clone())
    }

    #[wasm_export(
        js_name = "composeScenarioToRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_scenario_to_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let scenario = self.dotrain_yaml.get_scenario(&scenario)?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )?)
    }

    #[wasm_export(
        js_name = "composeScenarioToPostTaskRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_scenario_to_post_task_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let scenario = self.dotrain_yaml.get_scenario(&scenario)?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS,
        )?)
    }

    #[wasm_export(
        js_name = "composeDeploymentToRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_deployment_to_rainlang(
        &self,
        deployment: String,
    ) -> Result<String, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let scenario = self.dotrain_yaml.get_deployment(&deployment)?.scenario;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )?)
    }
}

impl DotrainOrder {
    pub fn dotrain_yaml(&self) -> DotrainYaml {
        self.dotrain_yaml.clone()
    }

    pub fn orderbook_yaml(&self) -> OrderbookYaml {
        OrderbookYaml::from_dotrain_yaml(self.dotrain_yaml.clone())
    }

    pub async fn get_pragmas_for_scenario(
        &self,
        scenario: &str,
    ) -> Result<Vec<Address>, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let deployer = self.dotrain_yaml.get_scenario(scenario)?.deployer;
        let parser: ParserV2 = deployer.address.into();
        let rainlang = self
            .compose_scenario_to_rainlang(scenario.to_string())
            .await?;

        let client = ReadableClient::new_from_url(deployer.network.rpc.clone().to_string())?;
        let pragmas = parser.parse_pragma_text(&rainlang, client).await?;
        Ok(pragmas)
    }

    pub async fn get_contract_authoring_meta_v2_for_scenario(
        &self,
        scenario: &str,
        address: Address,
    ) -> Result<AuthoringMetaV2, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let network = &self.dotrain_yaml.get_scenario(scenario)?.deployer.network;

        let rpc = &network.rpc;
        let metaboard = self.orderbook_yaml().get_metaboard(&network.key)?.url;
        Ok(
            AuthoringMetaV2::fetch_for_contract(address, rpc.to_string(), metaboard.to_string())
                .await?,
        )
    }

    pub async fn get_deployer_words_for_scenario(
        &self,
        scenario: &str,
    ) -> Result<ContractWords, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let deployer = &self.dotrain_yaml.get_scenario(scenario)?.deployer.address;

        Ok(ContractWords {
            address: *deployer,
            words: self
                .get_contract_authoring_meta_v2_for_scenario(scenario, *deployer)
                .await
                .into(),
        })
    }

    pub async fn get_pragma_words_for_scenario(
        &self,
        scenario: &str,
    ) -> Result<Vec<ContractWords>, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let pragma_addresses = self.get_pragmas_for_scenario(scenario).await?;
        let mut futures = vec![];

        for pragma in &pragma_addresses {
            futures.push(self.get_contract_authoring_meta_v2_for_scenario(scenario, *pragma));
        }

        Ok(pragma_addresses
            .into_iter()
            .zip(join_all(futures).await)
            .map(|(address, words)| ContractWords {
                address,
                words: words.into(),
            })
            .collect())
    }

    pub async fn get_all_words_for_scenario(
        &self,
        scenario: &str,
    ) -> Result<ScenarioWords, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let deployer = &self.dotrain_yaml.get_scenario(scenario)?.deployer.address;
        let mut addresses = vec![*deployer];
        addresses.extend(self.get_pragmas_for_scenario(scenario).await?);

        let mut futures = vec![];
        for address in addresses.clone() {
            futures.push(self.get_contract_authoring_meta_v2_for_scenario(scenario, address));
        }
        let mut results = join_all(futures).await;

        let deployer_words = ContractWords {
            address: *deployer,
            words: results.drain(0..1).nth(0).unwrap().into(),
        };
        let pragma_words = results
            .into_iter()
            .enumerate()
            .map(|(i, v)| ContractWords {
                address: addresses[i + 1],
                words: v.into(),
            })
            .collect();

        Ok(ScenarioWords {
            scenario: scenario.to_string(),
            pragma_words,
            deployer_words,
        })
    }

    pub async fn get_all_scenarios_all_words(
        &self,
    ) -> Result<Vec<ScenarioWords>, DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let mut scenarios = vec![];
        for scenario in self.dotrain_yaml.get_scenario_keys()? {
            scenarios.push(self.get_all_words_for_scenario(&scenario).await?);
        }
        Ok(scenarios)
    }

    pub async fn validate_raindex_version(&self) -> Result<(), DotrainOrderError> {
        if !self.is_initialized() {
            return Err(DotrainOrderError::DotrainOrderNotInitialized);
        }

        let app_sha = GH_COMMIT_SHA.to_string();

        if let Some(raindex_version) = &self.orderbook_yaml().get_raindex_version()? {
            if app_sha != *raindex_version {
                return Err(DotrainOrderError::RaindexVersionMismatch(
                    app_sha,
                    raindex_version.to_string(),
                ));
            }
        } else {
            return Err(DotrainOrderError::MissingRaindexVersion(app_sha));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{hex::encode_prefixed, primitives::B256, sol, sol_types::SolValue};
    use alloy_ethers_typecast::rpc::Response;
    use httpmock::MockServer;
    use rain_metadata::{KnownMagic, RainMetaDocumentV1Item};
    use serde_bytes::ByteBuf;

    sol!(
        struct AuthoringMetaV2Sol {
            bytes32 word;
            string description;
        }
    );
    sol!(
        struct PragmaV1 { address[] usingWordsFrom; }
    );

    #[tokio::test]
    async fn test_config_parse() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        assert_eq!(
            dotrain_order
                .orderbook_yaml()
                .get_network("polygon")
                .unwrap()
                .rpc
                .to_string(),
            server.url("/rpc"),
        );
    }

    #[tokio::test]
    async fn test_rainlang_from_scenario() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        bindings:
            key1: 10
        deployer: polygon
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let rainlang = dotrain_order
            .compose_scenario_to_rainlang("polygon".to_string())
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;"#
        );
    }

    #[tokio::test]
    async fn test_rainlang_post_from_scenario() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
"#,
            rpc_url = server.url("/rpc"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let rainlang = dotrain_order
            .compose_scenario_to_post_task_rainlang("polygon".to_string())
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. handle-add-order */ 
_ _: 1 2;"#
        );
    }

    #[tokio::test]
    async fn test_config_merge() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
networks:
  polygon:
    rpc: {rpc_url}
    chain-id: 137
    network-id: 137
    currency: MATIC
---
#calculate-io
_ _: 00;

#handle-io
:;"#,
            rpc_url = server.url("/rpc-polygon"),
        );

        let settings = format!(
            r#"
networks:
    mainnet:
        rpc: {rpc_url}
        chain-id: 1
        network-id: 1
        currency: ETH"#,
            rpc_url = server.url("/rpc-mainnet"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), Some(vec![settings.to_string()]))
            .await
            .unwrap();

        assert_eq!(
            dotrain_order
                .orderbook_yaml()
                .get_network("mainnet")
                .unwrap()
                .rpc
                .to_string(),
            server.url("/rpc-mainnet")
        );
    }

    #[tokio::test]
    async fn test_get_pragmas_for_scenario() {
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
networks:
    sepolia:
        rpc: {rpc_url}
        chain-id: 0
deployers:
    sepolia:
        address: 0x017F5651eB8fa4048BBc17433149c6c035d391A6
scenarios:
    sepolia:
        deployer: sepolia
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
using-words-from 0xb06202aA3Fe7d85171fB7aA5f17011d17E63f382
_: order-hash(),
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let pragmas = dotrain_order
            .get_pragmas_for_scenario("sepolia")
            .await
            .unwrap();

        assert_eq!(pragmas, pragma_addresses);
    }

    #[tokio::test]
    async fn test_get_contract_authoring_meta_v2_for_scenario() {
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key1: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key1 !Test binding
    #calculate-io
    using-words-from 0xbc609623F5020f6Fc7481024862cD5EE3FFf52D7
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let result = dotrain_order
            .get_contract_authoring_meta_v2_for_scenario("sepolia", pragma_addresses[0])
            .await
            .unwrap();

        assert_eq!(&result.words[0].word, "some-word");
        assert_eq!(&result.words[0].description, "some-desc");

        assert_eq!(&result.words[1].word, "some-other-word");
        assert_eq!(&result.words[1].description, "some-other-desc");
    }

    #[tokio::test]
    async fn test_get_pragma_words_for_scenario() {
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key1: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key1 !Test binding
    #calculate-io
    using-words-from 0xbc609623F5020f6Fc7481024862cD5EE3FFf52D7
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let result = dotrain_order
            .get_pragma_words_for_scenario("sepolia")
            .await
            .unwrap();

        assert!(result.len() == 1);
        assert_eq!(result[0].address, pragma_addresses[0]);
        assert!(matches!(result[0].words, WordsResult::Success(_)));
        if let WordsResult::Success(authoring_meta) = &result[0].words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
    }

    #[tokio::test]
    async fn test_get_deployer_words_for_scenario() {
        let server = mock_server(vec![]);
        let deployer = Address::random();
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer_address}
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key1: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #calculate-io
    using-words-from 0xbc609623F5020f6Fc7481024862cD5EE3FFf52D7
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            deployer_address = encode_prefixed(deployer),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let result = dotrain_order
            .get_deployer_words_for_scenario("sepolia")
            .await
            .unwrap();

        assert_eq!(result.address, deployer);
        assert!(matches!(result.words, WordsResult::Success(_)));
        if let WordsResult::Success(authoring_meta) = &result.words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
    }

    #[tokio::test]
    async fn test_get_all_words_for_scenario() {
        let deployer = Address::random();
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer_address}
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key1: 10
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key1 !Test binding
    #calculate-io
    using-words-from 0xbc609623F5020f6Fc7481024862cD5EE3FFf52D7
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            deployer_address = encode_prefixed(deployer),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let result = dotrain_order
            .get_all_words_for_scenario("sepolia")
            .await
            .unwrap();

        assert_eq!(&result.scenario, "sepolia");

        assert_eq!(result.deployer_words.address, deployer);
        assert!(matches!(
            result.deployer_words.words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &result.deployer_words.words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }

        assert!(result.pragma_words.len() == 1);
        assert_eq!(result.pragma_words[0].address, pragma_addresses[0]);
        assert!(matches!(
            result.pragma_words[0].words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &result.pragma_words[0].words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
    }

    #[tokio::test]
    async fn test_get_all_scenarios_all_words() {
        let deployer = Address::random();
        let pragma_addresses = vec![Address::random()];
        let server = mock_server(pragma_addresses.clone());
        let dotrain = format!(
            r#"
    networks:
        sepolia:
            rpc: {rpc_url}
            chain-id: 0
    deployers:
        sepolia:
            address: {deployer_address}
    scenarios:
        sepolia:
            deployer: sepolia
            bindings:
                key1: 10
        other-scenario:
            deployer: sepolia
            bindings:
                key1: 40
    metaboards:
        sepolia: {metaboard_url}
    ---
    #key1 !Test binding
    #calculate-io
    using-words-from 0xbc609623F5020f6Fc7481024862cD5EE3FFf52D7
    _: order-hash(),
    _ _: 0 0;
    #handle-io
    :;"#,
            rpc_url = server.url("/rpc"),
            metaboard_url = server.url("/sg"),
            deployer_address = encode_prefixed(deployer),
        );
        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let results = dotrain_order.get_all_scenarios_all_words().await.unwrap();

        assert_eq!(results.len(), 2);

        let other_scenario_result = results
            .iter()
            .find(|r| r.scenario == "other-scenario")
            .expect("Did not find results for 'other-scenario'");

        assert_eq!(other_scenario_result.deployer_words.address, deployer);
        assert!(matches!(
            other_scenario_result.deployer_words.words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &other_scenario_result.deployer_words.words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
        assert!(other_scenario_result.pragma_words.len() == 1);
        assert_eq!(
            other_scenario_result.pragma_words[0].address,
            pragma_addresses[0]
        );
        assert!(matches!(
            other_scenario_result.pragma_words[0].words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &other_scenario_result.pragma_words[0].words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }

        let sepolia_result = results
            .iter()
            .find(|r| r.scenario == "sepolia")
            .expect("Did not find results for 'sepolia'");

        assert_eq!(sepolia_result.deployer_words.address, deployer);
        assert!(matches!(
            sepolia_result.deployer_words.words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &sepolia_result.deployer_words.words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
        assert!(sepolia_result.pragma_words.len() == 1);
        assert_eq!(sepolia_result.pragma_words[0].address, pragma_addresses[0]);
        assert!(matches!(
            sepolia_result.pragma_words[0].words,
            WordsResult::Success(_)
        ));
        if let WordsResult::Success(authoring_meta) = &sepolia_result.pragma_words[0].words {
            assert_eq!(&authoring_meta.words[0].word, "some-word");
            assert_eq!(&authoring_meta.words[0].description, "some-desc");

            assert_eq!(&authoring_meta.words[1].word, "some-other-word");
            assert_eq!(&authoring_meta.words[1].description, "some-other-desc");
        }
    }

    // helper function to mock rpc and sg response
    fn mock_server(with_pragma_addresses: Vec<Address>) -> MockServer {
        let server = MockServer::start();
        // mock contract calls
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7ffffffff");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[0]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7");
            then.body(
                Response::new_success(1, &B256::left_padding_from(&[1]).to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x6f5aa28d");
            then.body(
                Response::new_success(1, &B256::random().to_string())
                    .to_json_string()
                    .unwrap(),
            );
        });
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x5514ca20");
            then.body(
                Response::new_success(
                    1,
                    &encode_prefixed(
                        PragmaV1 {
                            usingWordsFrom: with_pragma_addresses,
                        }
                        .abi_encode(),
                    ),
                )
                .to_json_string()
                .unwrap(),
            );
        });

        // mock sg query
        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&serde_json::json!({
                "data": {
                    "metaV1S": [{
                        "meta": encode_prefixed(
                            RainMetaDocumentV1Item {
                                payload: ByteBuf::from(
                                    vec![
                                        AuthoringMetaV2Sol {
                                            word: B256::right_padding_from("some-word".as_bytes()),
                                            description: "some-desc".to_string(),
                                        },
                                        AuthoringMetaV2Sol {
                                            word: B256::right_padding_from("some-other-word".as_bytes()),
                                            description: "some-other-desc".to_string(),
                                        }
                                    ]
                                    .abi_encode(),
                                ),
                                magic: KnownMagic::AuthoringMetaV2,
                                content_type: rain_metadata::ContentType::OctetStream,
                                content_encoding: rain_metadata::ContentEncoding::None,
                                content_language: rain_metadata::ContentLanguage::None,
                            }
                            .cbor_encode()
                            .unwrap()
                        ),
                        "metaHash": "0x00",
                        "sender": "0x00",
                        "id": "0x00",
                        "metaBoard": {
                            "id": "0x00",
                            "metas": [],
                            "address": "0x00",
                        },
                        "subject": "0x00",
                    }]
                }
            }));
        });
        server
    }

    #[tokio::test]
    async fn test_validate_raindex_version_happy() {
        let dotrain = format!(
            r#"
                raindex-version: {GH_COMMIT_SHA}
                networks:
                    sepolia:
                        rpc: http://example.com
                        chain-id: 0
                deployers:
                    sepolia:
                        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
                ---
                #calculate-io
                _ _: 0 0;
                #handle-io
                :;"#,
            GH_COMMIT_SHA = GH_COMMIT_SHA,
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        dotrain_order.validate_raindex_version().await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_raindex_version_unhappy() {
        let dotrain = format!(
            r#"
                raindex-version: {GH_COMMIT_SHA}
                networks:
                    sepolia:
                        rpc: http://example.com
                        chain-id: 0
                deployers:
                    sepolia:
                        address: 0x3131baC3E2Ec97b0ee93C74B16180b1e93FABd59
                ---
                #calculate-io
                _ _: 0 0;
                #handle-io
                :;"#,
            GH_COMMIT_SHA = "1234567890",
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        assert!(dotrain_order.validate_raindex_version().await.is_err());
    }

    #[tokio::test]
    async fn test_rainlang_from_deployment() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
networks:
    polygon:
        rpc: {rpc_url}
        chain-id: 137
        network-id: 137
        currency: MATIC
deployers:
    polygon:
        address: 0x1234567890123456789012345678901234567890
scenarios:
    polygon:
        deployer: polygon
        bindings:
            key1: 10
tokens:
    t1:
        network: polygon
        address: 0x1111111111111111111111111111111111111111
        decimals: 18
        label: Token1
        symbol: Token1
    t2:
        network: polygon
        address: 0x2222222222222222222222222222222222222222
        decimals: 18
        label: Token2
        symbol: token2
orders:
    polygon:
        inputs:
            - token: t1
        outputs:
            - token: t2
deployments:
    polygon:
        scenario: polygon
        order: polygon
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;"#,
            rpc_url = server.url("/rpc"),
        );

        let mut dotrain_order = DotrainOrder::new();
        dotrain_order
            .initialize(dotrain.to_string(), None)
            .await
            .unwrap();

        let rainlang = dotrain_order
            .compose_deployment_to_rainlang("polygon".to_string())
            .await
            .unwrap();

        assert_eq!(
            rainlang,
            r#"/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;"#
        );
    }

    #[tokio::test]
    async fn test_is_initialized() {
        let dotrain_order = DotrainOrder::new();
        assert!(!dotrain_order.is_initialized());

        assert!(matches!(
            dotrain_order.dotrain().unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .compose_deployment_to_rainlang("".to_string())
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .compose_scenario_to_rainlang("".to_string())
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .compose_scenario_to_post_task_rainlang("".to_string())
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_pragmas_for_scenario("")
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_contract_authoring_meta_v2_for_scenario("", Address::ZERO)
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_deployer_words_for_scenario("")
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_pragma_words_for_scenario("")
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_all_words_for_scenario("")
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order
                .get_all_scenarios_all_words()
                .await
                .unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
        assert!(matches!(
            dotrain_order.validate_raindex_version().await.unwrap_err(),
            DotrainOrderError::DotrainOrderNotInitialized
        ));
    }
}
