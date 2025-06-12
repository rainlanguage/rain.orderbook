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
use rain_orderbook_app_settings::spec_version::SpecVersion;
use rain_orderbook_app_settings::yaml::{
    dotrain::DotrainYaml, orderbook::OrderbookYaml, YamlError, YamlParsable,
};
use rain_orderbook_app_settings::ParseConfigSourceError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

/// DotrainOrder represents a parsed and validated dotrain configuration that combines
/// YAML frontmatter with Rainlang code for orderbook operations.
///
/// A dotrain file contains:
/// - YAML frontmatter defining networks, tokens, orders, scenarios, and deployments
/// - Rainlang code sections for order evaluation logic
///
/// This struct provides methods to compile scenarios and deployments into executable
/// Rainlang code with scenario-specific bindings applied.
///
/// # Examples
///
/// ```javascript
/// // Create from dotrain text
/// const result = await DotrainOrder.create(dotrainText);
/// if (result.error) {
///   console.error('Failed:', result.error.readableMsg);
/// } else {
///   const dotrainOrder = result.value;
///   // Do something with the dotrainOrder
/// }
///
/// // Compile scenario to Rainlang
/// const result = await dotrainOrder.composeScenarioToRainlang("my-scenario");
/// if (result.error) {
///   console.error('Failed:', result.error.readableMsg);
/// } else {
///   const rainlang = result.value;
///   // Do something with the rainlang
/// }
/// ```
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

    #[error("Spec version mismatch: got {1}, should be {0}")]
    SpecVersionMismatch(String, String),

    #[error("Spec version missing: should be {0}")]
    MissingSpecVersion(String),

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
            DotrainOrderError::SpecVersionMismatch(expected, got) => {
                format!("Configuration version mismatch. Expected '{}', but found '{}'. Please update 'version'.", expected, got)
            }
            DotrainOrderError::MissingSpecVersion(expected) => {
                format!(
                    "The required 'version' field is missing. Please add it and set it to '{}'.",
                    expected
                )
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
            dotrain_yaml: DotrainYaml::new(vec![], false).unwrap(),
        }
    }
}

#[wasm_export]
impl DotrainOrder {
    /// Creates a new DotrainOrder instance by parsing dotrain configuration text along with additional configuration.
    ///
    /// Parses the YAML frontmatter and validates the configuration including:
    /// - Spec version compatibility
    /// - Remote network configurations
    /// - Remote token definitions
    ///
    /// # Parameters
    ///
    /// * `dotrain` - Complete dotrain text containing YAML frontmatter and Rainlang code
    /// * `settings` - Optional additional YAML configuration strings to merge with the frontmatter
    ///
    /// # Returns
    ///
    /// * `Ok(DotrainOrder)` - Successfully parsed and validated DotrainOrder instance
    /// * `Err(DotrainOrderError)` - Configuration parsing failed, version mismatch, or network errors
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Basic usage
    /// const result = await DotrainOrder.create(dotrainText);
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const dotrainOrder = result.value;
    ///   // Do something with the dotrainOrder
    /// }
    ///
    /// // With additional settings
    /// const result = await DotrainOrder.create(dotrainText, [additionalConfig]);
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const dotrainOrder = result.value;
    ///   // Do something with the dotrainOrder
    /// }
    /// ```
    #[wasm_export(js_name = "create", preserve_js_class)]
    pub async fn create(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<DotrainOrder, DotrainOrderError> {
        let frontmatter = RainDocument::get_front_matter(&dotrain)
            .unwrap_or("")
            .to_string();

        let mut sources = vec![frontmatter.clone()];
        if let Some(settings) = settings {
            sources.extend(settings);
        }

        let mut orderbook_yaml = OrderbookYaml::new(sources.clone(), false)?;
        let spec_version = orderbook_yaml.get_spec_version()?;
        if !SpecVersion::is_current(&spec_version) {
            return Err(DotrainOrderError::SpecVersionMismatch(
                SpecVersion::current().to_string(),
                spec_version.to_string(),
            ));
        }

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

        Ok(DotrainOrder {
            dotrain,
            dotrain_yaml,
        })
    }

    /// Returns the original dotrain text used to create this DotrainOrder instance.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The complete dotrain text including YAML frontmatter and Rainlang code
    /// * `Err(DotrainOrderError)` - Instance not properly initialized (should not occur in normal usage)
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await dotrainOrder.dotrain();
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const dotrain = result.value;
    ///   // Do something with the dotrain
    /// }
    /// ```
    #[wasm_export(js_name = "dotrain", unchecked_return_type = "string")]
    pub fn dotrain(&self) -> Result<String, DotrainOrderError> {
        Ok(self.dotrain.clone())
    }

    /// Compiles a specific scenario into executable Rainlang code for order evaluation.
    ///
    /// Takes a scenario name from the dotrain configuration and compiles the Rainlang
    /// code with that scenario's bindings applied.
    ///
    /// # Parameters
    /// * `scenario` - Name of the scenario defined in the dotrain YAML frontmatter
    ///
    /// # Returns
    /// * `Ok(String)` - Compiled Rainlang code with scenario bindings applied
    /// * `Err(DotrainOrderError)` - Scenario not found or Rainlang compilation failed
    ///
    /// # Examples

    /// ```javascript
    /// // Compile a trading scenario
    /// const result = await dotrainOrder.composeScenarioToRainlang("market-making");
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const rainlang = result.value;
    ///   // Do something with the rainlang
    /// }
    /// ```
    #[wasm_export(
        js_name = "composeScenarioToRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_scenario_to_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        let scenario = self.dotrain_yaml.get_scenario(&scenario)?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )?)
    }

    /// Compiles a specific scenario into Rainlang code for post-task execution.
    ///
    /// Post-task Rainlang code runs after an order is successfully added to the orderbook.
    ///
    /// # Parameters
    ///
    /// * `scenario` - Name of the scenario defined in the dotrain YAML frontmatter
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Compiled post-task Rainlang code with scenario bindings applied
    /// * `Err(DotrainOrderError)` - Scenario not found or Rainlang compilation failed
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Compile post-task logic for notifications
    /// const result = await dotrainOrder.composeScenarioToPostTaskRainlang("notification-scenario");
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const postTaskCode = result.value;
    ///   // Do something with the postTaskCode
    /// }
    /// ```
    #[wasm_export(
        js_name = "composeScenarioToPostTaskRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_scenario_to_post_task_rainlang(
        &self,
        scenario: String,
    ) -> Result<String, DotrainOrderError> {
        let scenario = self.dotrain_yaml.get_scenario(&scenario)?;

        Ok(compose_to_rainlang(
            self.dotrain.clone(),
            scenario.bindings.clone(),
            &ORDERBOOK_ADDORDER_POST_TASK_ENTRYPOINTS,
        )?)
    }

    /// Compiles a specific deployment configuration into executable Rainlang code.
    ///
    /// A deployment combines an order definition with a scenario to create a complete
    /// configuration ready for execution. This method resolves the deployment's scenario
    /// and compiles the Rainlang code with the appropriate bindings.
    ///
    /// # Parameters
    ///
    /// * `deployment` - Name of the deployment defined in the dotrain YAML frontmatter
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Compiled Rainlang code for the deployment's scenario
    /// * `Err(DotrainOrderError)` - Deployment not found or Rainlang compilation failed
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // Compile a production deployment
    /// const result = await dotrainOrder.composeDeploymentToRainlang("production");
    /// if (result.error) {
    ///   console.error('Failed:', result.error.readableMsg);
    /// } else {
    ///   const rainlang = result.value;
    ///   // Do something with the rainlang
    /// }
    /// ```
    #[wasm_export(
        js_name = "composeDeploymentToRainlang",
        unchecked_return_type = "string"
    )]
    pub async fn compose_deployment_to_rainlang(
        &self,
        deployment: String,
    ) -> Result<String, DotrainOrderError> {
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
        let mut scenarios = vec![];
        for scenario in self.dotrain_yaml.get_scenario_keys()? {
            scenarios.push(self.get_all_words_for_scenario(&scenario).await?);
        }
        Ok(scenarios)
    }

    pub async fn validate_spec_version(&self) -> Result<(), DotrainOrderError> {
        let spec_version = self.orderbook_yaml().get_spec_version()?;
        if !SpecVersion::is_current(&spec_version) {
            return Err(DotrainOrderError::SpecVersionMismatch(
                SpecVersion::current(),
                spec_version,
            ));
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
    use rain_orderbook_app_settings::yaml::FieldErrorKind;
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
version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
version: {spec_version}
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
            spec_version = SpecVersion::current()
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

        let dotrain_order =
            DotrainOrder::create(dotrain.to_string(), Some(vec![settings.to_string()]))
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
version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    version: {spec_version}
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
            spec_version = SpecVersion::current()
        );
        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
    async fn test_validate_spec_version_happy() {
        let dotrain = format!(
            "
                version: {spec_version}
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
                :;",
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
            .await
            .unwrap();

        dotrain_order.validate_spec_version().await.unwrap();
    }

    #[tokio::test]
    async fn test_validate_missing_spec_version() {
        let dotrain = "
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
                :;";

        let err = DotrainOrder::create(dotrain.to_string(), None)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DotrainOrderError::YamlError(YamlError::Field {
                kind: FieldErrorKind::Missing(ref key),
                location
            }) if key == "version" && location == "root"
        ));
    }

    #[tokio::test]
    async fn test_validate_spec_version_unhappy() {
        let dotrain = "
                version: 2
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
                :;";

        let err = DotrainOrder::create(dotrain.to_string(), None)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            DotrainOrderError::SpecVersionMismatch(
                ref expected,
                ref got
            ) if expected == &SpecVersion::current() && got == "2"
        ));
    }

    #[tokio::test]
    async fn test_rainlang_from_deployment() {
        let server = mock_server(vec![]);
        let dotrain = format!(
            r#"
version: {spec_version}
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
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.to_string(), None)
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
}
