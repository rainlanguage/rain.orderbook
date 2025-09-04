use crate::gui::{DotrainOrderGui, GuiError};
use rain_orderbook_app_settings::gui::NameAndDescriptionCfg;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use url::Url;
use wasm_bindgen_utils::{prelude::*, wasm_export};

/// A registry system for managing dotrain order configurations with layered content merging.
///
/// The `DotrainRegistry` provides a centralized way to fetch, parse, and manage dotrain order
/// strategies from remote sources. It supports a layered architecture where shared settings
/// are merged with individual order configurations.
///
/// ## Registry File Format
///
/// The registry file follows a specific format:
/// - **First line**: URL to shared settings YAML file (without a key)
/// - **Subsequent lines**: Order entries in format "key url"
///
/// ```text
/// https://example.com/shared-settings.yaml
/// fixed-limit https://example.com/fixed-limit.rain
/// auction-dca https://example.com/auction-dca.rain
/// ```
///
/// ## Content Merging
///
/// For each order, the final dotrain content is created by merging:
/// ```text
/// <settings.yaml content>
///
/// <.rain file content>
/// ```
///
/// This allows strategies to share common network configurations, tokens, and other settings
/// while maintaining their individual logic.
///
/// ## Usage Flow
///
/// 1. **Registry Creation** → Fetches and parses registry file
/// 2. **List Orders** → Get available order strategies with metadata
/// 3. **List Deployments** → Get deployment options for selected order  
/// 4. **Create GUI** → Instantiate DotrainOrderGui with merged content
///
/// ## Examples
///
/// ```javascript
/// // Initialize registry
/// const registry = await DotrainRegistry.new("https://example.com/registry.txt");
///
/// // Get available orders
/// const orders = await registry.getAllOrderDetails();
///
/// // Get deployments for specific order
/// const deployments = await registry.getDeploymentDetails("fixed-limit");
///
/// // Create GUI instance
/// const gui = await registry.getGui("fixed-limit", "mainnet", stateCallback);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainRegistry {
    /// The URL of the registry file containing the list of orders and settings.
    ///
    /// This is the original URL passed to the constructor and serves as the entry point
    /// for fetching the registry configuration. The registry file should contain a
    /// settings URL on the first line followed by order entries.
    registry_url: Url,

    /// The raw content of the registry file as fetched from `registry_url`.
    ///
    /// Contains the complete registry file content including the settings URL and
    /// all order entries. This is stored for reference and debugging purposes.
    /// Format: settings URL on first line, then "key url" pairs for each order.
    registry: String,

    /// The URL of the shared settings YAML file extracted from the first line of the registry.
    ///
    /// This URL points to a YAML file containing shared configuration such as:
    /// - Network configurations (RPCs, chain IDs)
    /// - Subgraph endpoints
    /// - Orderbook contract addresses
    /// - Token definitions
    /// - Other common settings used across multiple strategies
    settings_url: Url,

    /// The content of the shared settings YAML file fetched from `settings_url`.
    ///
    /// Contains shared configuration that will be merged with individual order content
    /// to create complete dotrain files. This typically includes network settings,
    /// contract addresses, and other common configuration that strategies can inherit.
    settings: String,

    /// A map of order keys to their corresponding .rain file URLs.
    ///
    /// Each entry represents an available order where:
    /// - **Key**: Human-readable order identifier (e.g., "fixed-limit", "auction-dca")
    /// - **Value**: URL pointing to the .rain file containing the order's dotrain configuration
    ///
    /// This mapping is parsed from the registry file lines (excluding the first settings line).
    order_urls: HashMap<String, Url>,

    /// A map of order keys to their corresponding .rain file contents.
    ///
    /// Each entry contains the raw dotrain content for an order:
    /// - **Key**: Order identifier matching the keys in `order_urls`
    /// - **Value**: Raw dotrain content fetched from the corresponding URL
    ///
    /// This content is fetched in parallel during registry initialization and stored
    /// for quick access. It gets merged with `settings` content when creating GUIs.
    orders: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum DotrainRegistryError {
    #[error("Failed to fetch registry from URL: {0}")]
    RegistryFetchError(String),
    #[error("Failed to parse registry content")]
    RegistryParseError,
    #[error("Failed to fetch settings from URL: {0}")]
    SettingsFetchError(String),
    #[error("Failed to fetch order content from URL: {0}")]
    OrderFetchError(String),
    #[error("Order key not found: {0}")]
    OrderKeyNotFound(String),
    #[error("Invalid registry format: {0}")]
    InvalidRegistryFormat(String),
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    GuiError(#[from] GuiError),
}

impl DotrainRegistryError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            DotrainRegistryError::RegistryFetchError(url) => {
                format!("Unable to fetch the registry file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRegistryError::RegistryParseError => {
                "The registry file format is invalid. Please ensure it follows the expected format with settings URL on the first line and order entries on subsequent lines.".to_string()
            }
            DotrainRegistryError::SettingsFetchError(url) => {
                format!("Unable to fetch the settings file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRegistryError::OrderFetchError(url) => {
                format!("Unable to fetch the order file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRegistryError::OrderKeyNotFound(key) => {
                format!("The order key '{}' was not found in the registry. Please check the available order keys.", key)
            }
            DotrainRegistryError::InvalidRegistryFormat(msg) => {
                format!("Invalid registry format: {}", msg)
            }
            DotrainRegistryError::HttpError(msg) => {
                format!("Network error: {}", msg)
            }
            DotrainRegistryError::UrlParseError(err) => {
                format!("Invalid URL format: {}. Please ensure the URL is properly formatted.", err)
            }
            DotrainRegistryError::GuiError(err) => err.to_readable_msg()
        }
    }
}

impl From<DotrainRegistryError> for WasmEncodedError {
    fn from(value: DotrainRegistryError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[wasm_bindgen]
impl DotrainRegistry {
    #[wasm_bindgen(getter = registryUrl)]
    pub fn registry_url(&self) -> String {
        self.registry_url.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn registry(&self) -> String {
        self.registry.clone()
    }
    #[wasm_bindgen(getter = settingsUrl)]
    pub fn settings_url(&self) -> String {
        self.settings_url.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn settings(&self) -> String {
        self.settings.clone()
    }
    #[wasm_bindgen(getter = orderUrls)]
    pub fn order_urls(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (key, value) in &self.order_urls {
            map.set(&key.into(), &value.to_string().into());
        }
        map
    }
    #[wasm_bindgen(getter = orders)]
    pub fn orders(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (key, value) in &self.orders {
            map.set(&key.into(), &value.into());
        }
        map
    }
}

#[wasm_export]
impl DotrainRegistry {
    /// Creates a new DotrainRegistry instance by fetching and parsing the registry file.
    ///
    /// The registry file should contain a settings YAML URL on the first line (without a key),
    /// followed by order entries in the format "key url".
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await DotrainRegistry.new("https://example.com/registry.txt");
    /// if (result.error) {
    ///   console.error("Registry creation failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const registry = result.value;
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "DotrainRegistry instance with settings and orders loaded"
    )]
    pub async fn new(
        #[wasm_export(
            js_name = "registryUrl",
            param_description = "URL to the registry file containing settings and order definitions"
        )]
        registry_url: String,
    ) -> Result<DotrainRegistry, DotrainRegistryError> {
        let registry_url = Url::parse(&registry_url)?;
        let (registry_content, settings_url, order_urls) =
            Self::fetch_and_parse_registry(&registry_url).await?;
        let settings = Self::fetch_settings(&settings_url).await?;
        let orders = Self::fetch_orders(&order_urls).await?;

        Ok(DotrainRegistry {
            registry_url,
            registry: registry_content,
            settings_url,
            settings,
            order_urls,
            orders,
        })
    }

    /// Gets details for all orders in the registry.
    ///
    /// This method extracts name and description information for each order,
    /// useful for building the initial order selection UI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getAllOrderDetails();
    /// if (result.error) {
    ///   console.error("Failed to get order details:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orderDetails = result.value;
    /// // Map of order key -> {name, description, short_description}
    /// for (const [orderKey, details] of orderDetails) {
    ///   console.log(`${orderKey}: ${details.name}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getAllOrderDetails",
        unchecked_return_type = "Map<string, NameAndDescriptionCfg>",
        return_description = "Map of order key to order metadata"
    )]
    pub fn get_all_order_details(
        &self,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, DotrainRegistryError> {
        let mut order_details = BTreeMap::new();

        for (order_key, dotrain) in &self.orders {
            let details = DotrainOrderGui::get_order_details(dotrain.clone())?;
            order_details.insert(order_key.clone(), details);
        }

        Ok(order_details)
    }

    /// Returns a list of all order keys available in the registry.
    ///
    /// Use this method to get the available order identifiers.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getOrderKeys();
    /// if (result.error) {
    ///   console.error("Failed to fetch order keys:", result.error.readableMsg);
    ///   return;
    /// }
    /// const keys = result.value;
    /// console.log("Available orders:", keys);
    /// ```
    #[wasm_export(
        js_name = "getOrderKeys",
        unchecked_return_type = "string[]",
        return_description = "Array of order keys available in the registry"
    )]
    pub fn get_order_keys(&self) -> Result<Vec<String>, DotrainRegistryError> {
        Ok(self.order_urls.keys().cloned().collect())
    }

    /// Gets deployment details for a specific order.
    ///
    /// This method extracts deployment information for a given order,
    /// useful for building the deployment selection UI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getDeploymentDetails("fixed-limit");
    /// if (result.error) {
    ///   console.error("Failed to get deployment details:", result.error.readableMsg);
    ///   return;
    /// }
    /// const deploymentDetails = result.value;
    /// // Map of deployment key -> {name, description, short_description}
    /// for (const [deploymentKey, details] of deploymentDetails) {
    ///   console.log(`${deploymentKey}: ${details.name}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getDeploymentDetails",
        unchecked_return_type = "Map<string, NameAndDescriptionCfg>",
        return_description = "Map of deployment key to deployment metadata"
    )]
    pub fn get_deployment_details(
        &self,
        #[wasm_export(
            js_name = "orderKey",
            param_description = "Order key to get deployment details for"
        )]
        order_key: String,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, DotrainRegistryError> {
        let dotrain = self
            .orders
            .get(&order_key)
            .ok_or(DotrainRegistryError::OrderKeyNotFound(order_key.clone()))?;
        let deployment_details = DotrainOrderGui::get_deployment_details(dotrain.clone())?;
        Ok(deployment_details)
    }

    /// Creates a DotrainOrderGui instance for a specific order and deployment.
    ///
    /// This is a convenience method that combines getting a DotrainOrder and creating a GUI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Simple usage without state callback
    /// const result = await registry.getGui("fixed-limit", "mainnet-deployment");
    /// if (result.error) {
    ///   console.error("Failed to create GUI:", result.error.readableMsg);
    ///   return;
    /// }
    /// const gui = result.value;
    ///
    /// // Usage with state update callback for auto-saving
    /// const stateCallback = (newState) => {
    ///   localStorage.setItem('gui-state', JSON.stringify(newState));
    /// };
    /// const resultWithCallback = await registry.getGui("fixed-limit", "mainnet-deployment", stateCallback);
    /// ```
    #[wasm_export(
        js_name = "getGui",
        preserve_js_class,
        unchecked_return_type = "DotrainOrderGui",
        return_description = "DotrainOrderGui instance for the specified order and deployment"
    )]
    pub async fn get_gui(
        &self,
        #[wasm_export(
            js_name = "orderKey",
            param_description = "Order key to fetch the GUI for"
        )]
        order_key: String,
        #[wasm_export(
            js_name = "deploymentKey",
            param_description = "Deployment key to create the GUI for"
        )]
        deployment_key: String,
        #[wasm_export(
            js_name = "stateUpdateCallback",
            param_description = "Optional function called on state changes. \
            After a state change (deposit, field value, vault id, select token, etc.), the callback is called with the new state. \
            This is useful for auto-saving the state of the GUI across sessions."
        )]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<DotrainOrderGui, DotrainRegistryError> {
        let merged_content = self.merge_content_for_order(&order_key)?;
        let gui = DotrainOrderGui::new_with_deployment(
            merged_content,
            deployment_key,
            state_update_callback,
        )
        .await?;
        Ok(gui)
    }
}

impl DotrainRegistry {
    async fn fetch_and_parse_registry(
        registry_url: &Url,
    ) -> Result<(String, Url, HashMap<String, Url>), DotrainRegistryError> {
        let registry_content = Self::fetch_url_content(registry_url).await?;
        let (settings_url, order_urls) = Self::parse_registry_content(&registry_content)?;
        Ok((registry_content, settings_url, order_urls))
    }

    fn parse_registry_content(
        content: &str,
    ) -> Result<(Url, HashMap<String, Url>), DotrainRegistryError> {
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(DotrainRegistryError::InvalidRegistryFormat(
                "Registry file is empty".to_string(),
            ));
        }

        let first_line = lines[0];
        if first_line.contains(' ') {
            return Err(DotrainRegistryError::InvalidRegistryFormat(
                "First line should be a settings URL without a key".to_string(),
            ));
        }

        let settings_url = Url::parse(first_line)?;
        let mut order_urls = HashMap::new();

        for line in &lines[1..] {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(DotrainRegistryError::InvalidRegistryFormat(format!(
                    "Invalid order entry format: '{}'. Expected: 'key url'",
                    line
                )));
            }

            let key = parts[0].to_string();
            let url = Url::parse(parts[1])?;

            order_urls.insert(key, url);
        }

        Ok((settings_url, order_urls))
    }

    async fn fetch_url_content(url: &Url) -> Result<String, DotrainRegistryError> {
        let response = reqwest::get(url.as_str())
            .await
            .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DotrainRegistryError::HttpError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))
    }

    async fn fetch_settings(settings_url: &Url) -> Result<String, DotrainRegistryError> {
        Self::fetch_url_content(settings_url).await
    }

    async fn fetch_orders(
        order_urls: &HashMap<String, Url>,
    ) -> Result<HashMap<String, String>, DotrainRegistryError> {
        use futures::future::join_all;

        let mut futures = Vec::new();

        for (key, url) in order_urls {
            let key_clone = key.clone();
            let url_clone = url.clone();
            futures.push(async move {
                let content = reqwest::get(url_clone.as_str())
                    .await
                    .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))?
                    .text()
                    .await
                    .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))?;
                Ok::<(String, String), DotrainRegistryError>((key_clone, content))
            });
        }

        let results = join_all(futures).await;
        let mut orders = HashMap::new();

        for result in results {
            let (key, content) = result?;
            orders.insert(key, content);
        }

        Ok(orders)
    }

    fn merge_content_for_order(&self, order_key: &str) -> Result<String, DotrainRegistryError> {
        let dotrain = self
            .orders
            .get(order_key)
            .ok_or(DotrainRegistryError::OrderKeyNotFound(
                order_key.to_string(),
            ))?;

        if self.settings.is_empty() {
            Ok(dotrain.clone())
        } else {
            Ok(format!("{}\n\n{}", self.settings, dotrain))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const MOCK_REGISTRY_CONTENT: &str = r#"https://example.com/settings.yaml
fixed-limit https://example.com/fixed-limit.rain
auction-dca https://example.com/auction-dca.rain"#;

    const MOCK_SETTINGS_CONTENT: &str = r#"version: 3
networks:
  flare:
    rpcs: 
      - https://rpc.ankr.com/flare
    chain-id: 14
    currency: FLR
  base:
    rpcs: 
      - https://mainnet.base.org
    chain-id: 8453
    currency: ETH
subgraphs:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn
metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
orderbooks:
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
    network: flare
    subgraph: flare
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
    network: base
    subgraph: base
deployers:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
    network: flare
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
    network: base
tokens:
  token1:
    address: 0x4200000000000000000000000000000000000042
    network: flare
  token2:
    address: 0x4200000000000000000000000000000000000042
    network: base
"#;

    const MOCK_DOTRAIN_PREFIX: &str = r#"gui:
  name: Test gui
  description: Test description
  short-description: Test short description
  deployments:
    flare:
      name: Flare order name
      description: Flare order description
      deposits:
        - token: token1
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
    base:
      name: Base order name
      description: Base order description
      deposits:
        - token: token2
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
scenarios:
  flare:
    deployer: flare
    runs: 1
  base:
    deployer: base
    runs: 1
orders:
  flare:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  base:
    orderbook: base
    inputs:
      - token: token2
    outputs:
      - token: token2
deployments:
  flare:
    scenario: flare
    order: flare
  base:
    scenario: base
    order: base
"#;

    fn get_first_dotrain_content() -> String {
        format!(
            r#"{prefix}
----
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            prefix = MOCK_DOTRAIN_PREFIX
        )
    }

    fn get_second_dotrain_content() -> String {
        format!(
            r#"{prefix}
----
#calculate-io
_ _: 1 1;
#handle-io
:;
#handle-add-order
:;"#,
            prefix = MOCK_DOTRAIN_PREFIX
        )
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        fn test_parse_registry_content() {
            let (settings_url, order_urls) =
                DotrainRegistry::parse_registry_content(MOCK_REGISTRY_CONTENT).unwrap();

            assert_eq!(
                settings_url.to_string(),
                "https://example.com/settings.yaml"
            );
            assert_eq!(order_urls.len(), 2);
            assert_eq!(
                order_urls.get("fixed-limit").map(|u| u.to_string()),
                Some("https://example.com/fixed-limit.rain".to_string())
            );
            assert_eq!(
                order_urls.get("auction-dca").map(|u| u.to_string()),
                Some("https://example.com/auction-dca.rain".to_string())
            );
        }

        #[wasm_bindgen_test]
        fn test_parse_invalid_registry_content() {
            let result = DotrainRegistry::parse_registry_content("");
            assert!(result.is_err());

            let result = DotrainRegistry::parse_registry_content("invalid first line");
            assert!(result.is_err());

            let result = DotrainRegistry::parse_registry_content(
                "https://example.com/settings.yaml\ninvalid-entry",
            );
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_get_order_keys() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: "".to_string(),
                order_urls: vec![
                    (
                        "fixed-limit".to_string(),
                        Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                    ),
                    (
                        "auction-dca".to_string(),
                        Url::parse("https://example.com/auction-dca.rain").unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
                orders: HashMap::new(),
            };

            let keys = registry.get_order_keys().unwrap();
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"fixed-limit".to_string()));
            assert!(keys.contains(&"auction-dca".to_string()));
        }

        #[wasm_bindgen_test]
        fn test_get_all_order_details() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: MOCK_SETTINGS_CONTENT.to_string(),
                order_urls: vec![
                    (
                        "fixed-limit".to_string(),
                        Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                    ),
                    (
                        "auction-dca".to_string(),
                        Url::parse("https://example.com/auction-dca.rain").unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
                orders: vec![
                    ("fixed-limit".to_string(), get_first_dotrain_content()),
                    ("auction-dca".to_string(), get_second_dotrain_content()),
                ]
                .into_iter()
                .collect(),
            };

            let result = registry.get_all_order_details();
            wasm_bindgen_test::console_log!("Result: {:?}", result);
            assert!(result.is_ok());

            let order_details = result.unwrap();
            assert_eq!(order_details.len(), 2);
            assert!(order_details.contains_key("fixed-limit"));
            assert!(order_details.contains_key("auction-dca"));

            let fixed_limit_details = order_details.get("fixed-limit").unwrap();
            assert_eq!(fixed_limit_details.name, "Test gui");
            assert_eq!(fixed_limit_details.description, "Test description");
        }

        #[wasm_bindgen_test]
        fn test_get_deployment_details() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: MOCK_SETTINGS_CONTENT.to_string(),
                order_urls: vec![(
                    "fixed-limit".to_string(),
                    Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                )]
                .into_iter()
                .collect(),
                orders: vec![("fixed-limit".to_string(), get_first_dotrain_content())]
                    .into_iter()
                    .collect(),
            };

            let result = registry.get_deployment_details("fixed-limit".to_string());
            assert!(result.is_ok());

            let deployment_details = result.unwrap();
            assert_eq!(deployment_details.len(), 2);
            assert!(deployment_details.contains_key("flare"));
            assert!(deployment_details.contains_key("base"));

            let flare_details = deployment_details.get("flare").unwrap();
            assert_eq!(flare_details.name, "Flare order name");
            assert_eq!(flare_details.description, "Flare order description");

            let base_details = deployment_details.get("base").unwrap();
            assert_eq!(base_details.name, "Base order name");
            assert_eq!(base_details.description, "Base order description");
        }

        #[wasm_bindgen_test]
        fn test_get_deployment_details_order_not_found() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: MOCK_SETTINGS_CONTENT.to_string(),
                order_urls: HashMap::new(),
                orders: HashMap::new(),
            };

            let result = registry.get_deployment_details("non-existent".to_string());
            assert!(result.is_err());

            match result.err().unwrap() {
                DotrainRegistryError::OrderKeyNotFound(key) => {
                    assert_eq!(key, "non-existent");
                }
                _ => panic!("Expected OrderKeyNotFound error"),
            }
        }

        #[wasm_bindgen_test]
        fn test_getter_methods() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/registry.txt").unwrap(),
                registry: MOCK_REGISTRY_CONTENT.to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: MOCK_SETTINGS_CONTENT.to_string(),
                order_urls: vec![(
                    "fixed-limit".to_string(),
                    Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                )]
                .into_iter()
                .collect(),
                orders: vec![("fixed-limit".to_string(), get_first_dotrain_content())]
                    .into_iter()
                    .collect(),
            };

            assert_eq!(registry.registry_url(), "https://example.com/registry.txt");
            assert_eq!(registry.settings_url(), "https://example.com/settings.yaml");
            assert_eq!(registry.registry(), MOCK_REGISTRY_CONTENT);
            assert_eq!(registry.settings(), MOCK_SETTINGS_CONTENT);

            let order_urls_map = registry.order_urls();
            assert_eq!(order_urls_map.size(), 1);

            let orders_map = registry.orders();
            assert_eq!(orders_map.size(), 1);
        }

        #[wasm_bindgen_test]
        fn test_merge_content_order_not_found() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: "".to_string(),
                order_urls: HashMap::new(),
                orders: HashMap::new(),
            };

            let result = registry.merge_content_for_order("non-existent");
            assert!(result.is_err());
            match result.err().unwrap() {
                DotrainRegistryError::OrderKeyNotFound(key) => {
                    assert_eq!(key, "non-existent");
                }
                _ => panic!("Expected OrderKeyNotFound error"),
            }
        }

        #[wasm_bindgen_test]
        fn test_error_readable_messages() {
            let registry_error =
                DotrainRegistryError::RegistryFetchError("https://example.com".to_string());
            let readable = registry_error.to_readable_msg();
            assert!(readable.contains("Unable to fetch the registry file"));
            assert!(readable.contains("https://example.com"));

            let parse_error = DotrainRegistryError::RegistryParseError;
            let readable = parse_error.to_readable_msg();
            assert!(readable.contains("registry file format is invalid"));

            let not_found_error = DotrainRegistryError::OrderKeyNotFound("test-order".to_string());
            let readable = not_found_error.to_readable_msg();
            assert!(readable.contains("order key 'test-order' was not found"));
        }

        #[wasm_bindgen_test]
        fn test_merge_content_for_order() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: MOCK_SETTINGS_CONTENT.to_string(),
                order_urls: HashMap::new(),
                orders: vec![
                    ("first-order".to_string(), get_first_dotrain_content()),
                    ("second-order".to_string(), get_second_dotrain_content()),
                ]
                .into_iter()
                .collect(),
            };

            let merged1 = registry.merge_content_for_order("first-order").unwrap();
            let expected1 = format!(
                "{}\n\n{}",
                MOCK_SETTINGS_CONTENT,
                get_first_dotrain_content()
            );
            assert_eq!(merged1, expected1);

            assert!(merged1.starts_with("version: 3\nnetworks:"));
            assert!(merged1.contains("\n\ngui:"));
            assert!(merged1.contains("_ _: 0 0;"));

            let merged2 = registry.merge_content_for_order("second-order").unwrap();
            let expected2 = format!(
                "{}\n\n{}",
                MOCK_SETTINGS_CONTENT,
                get_second_dotrain_content()
            );
            assert_eq!(merged2, expected2);

            assert!(merged2.starts_with("version: 3\nnetworks:"));
            assert!(merged2.contains("\n\ngui:"));
            assert!(merged2.contains("_ _: 1 1;"));

            assert_ne!(merged1, merged2);
            assert!(merged1.contains("_ _: 0 0;") && !merged1.contains("_ _: 1 1;"));
            assert!(merged2.contains("_ _: 1 1;") && !merged2.contains("_ _: 0 0;"));
        }

        #[wasm_bindgen_test]
        fn test_merge_content_empty_settings() {
            let registry = DotrainRegistry {
                registry_url: Url::parse("https://example.com/test").unwrap(),
                registry: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: "".to_string(),
                order_urls: HashMap::new(),
                orders: vec![("test-order".to_string(), get_first_dotrain_content())]
                    .into_iter()
                    .collect(),
            };

            let merged = registry.merge_content_for_order("test-order").unwrap();
            assert_eq!(merged, get_first_dotrain_content());
            assert!(merged.starts_with("gui:"));
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use httpmock::MockServer;

        #[tokio::test]
        async fn test_new_constructor() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\nfirst-order {}/first-order.rain\nsecond-order {}/second-order.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(MOCK_SETTINGS_CONTENT);
            });

            server.mock(|when, then| {
                when.method("GET").path("/first-order.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/second-order.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let registry = DotrainRegistry::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(
                registry.registry_url(),
                format!("{}/registry.txt", server.url(""))
            );
            assert_eq!(registry.registry(), test_registry_content);
            assert_eq!(
                registry.settings_url(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(registry.settings(), MOCK_SETTINGS_CONTENT);
            assert_eq!(registry.order_urls.len(), 2);
            assert_eq!(registry.orders.len(), 2);
            assert!(registry.order_urls.contains_key("first-order"));
            assert!(registry.order_urls.contains_key("second-order"));
            assert!(registry.orders.contains_key("first-order"));
            assert!(registry.orders.contains_key("second-order"));

            let first_order_content = registry.orders.get("first-order").unwrap();
            let second_order_content = registry.orders.get("second-order").unwrap();
            assert_ne!(first_order_content, second_order_content);
            assert!(first_order_content.contains("_ _: 0 0;"));
            assert!(second_order_content.contains("_ _: 1 1;"));
        }

        #[tokio::test]
        async fn test_fetch_and_parse_registry() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\norder1 {}/order1.rain\norder2 {}/order2.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            let registry_url = Url::parse(&format!("{}/registry.txt", server.url(""))).unwrap();
            let (registry_content, settings_url, order_urls) =
                DotrainRegistry::fetch_and_parse_registry(&registry_url)
                    .await
                    .unwrap();

            assert_eq!(registry_content, test_registry_content);
            assert_eq!(
                settings_url.to_string(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(order_urls.len(), 2);
            assert!(order_urls.contains_key("order1"));
            assert!(order_urls.contains_key("order2"));
        }

        #[tokio::test]
        async fn test_fetch_url_content_success() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/test.txt");
                then.status(200).body("test content");
            });

            let url = Url::parse(&format!("{}/test.txt", server.url(""))).unwrap();
            let content = DotrainRegistry::fetch_url_content(&url).await.unwrap();

            assert_eq!(content, "test content");
        }

        #[tokio::test]
        async fn test_fetch_url_content_http_error() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/error.txt");
                then.status(500);
            });

            let url = Url::parse(&format!("{}/error.txt", server.url(""))).unwrap();
            let result = DotrainRegistry::fetch_url_content(&url).await;

            assert!(result.is_err());
            match result.err().unwrap() {
                DotrainRegistryError::HttpError(msg) => {
                    assert!(msg.contains("HTTP 500"));
                }
                _ => panic!("Expected HttpError"),
            }
        }

        #[tokio::test]
        async fn test_fetch_settings() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body("test settings content");
            });

            let url = Url::parse(&format!("{}/settings.yaml", server.url(""))).unwrap();
            let settings = DotrainRegistry::fetch_settings(&url).await.unwrap();

            assert_eq!(settings, "test settings content");
        }

        #[tokio::test]
        async fn test_fetch_orders() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/order1.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/order2.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let order_urls: HashMap<String, Url> = vec![
                (
                    "order1".to_string(),
                    Url::parse(&format!("{}/order1.rain", server.url(""))).unwrap(),
                ),
                (
                    "order2".to_string(),
                    Url::parse(&format!("{}/order2.rain", server.url(""))).unwrap(),
                ),
            ]
            .into_iter()
            .collect();

            let orders = DotrainRegistry::fetch_orders(&order_urls).await.unwrap();

            assert_eq!(orders.len(), 2);
            assert_eq!(orders.get("order1").unwrap(), &get_first_dotrain_content());
            assert_eq!(orders.get("order2").unwrap(), &get_second_dotrain_content());

            let order1_content = orders.get("order1").unwrap();
            let order2_content = orders.get("order2").unwrap();
            assert_ne!(order1_content, order2_content);
            assert!(order1_content.contains("_ _: 0 0;"));
            assert!(order2_content.contains("_ _: 1 1;"));
        }

        #[tokio::test]
        async fn test_invalid_registry_format() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/invalid-registry.txt");
                then.status(200)
                    .body("invalid format without proper structure");
            });

            let result =
                DotrainRegistry::new(format!("{}/invalid-registry.txt", server.url(""))).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_empty_registry_file() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/empty-registry.txt");
                then.status(200).body("");
            });

            let result =
                DotrainRegistry::new(format!("{}/empty-registry.txt", server.url(""))).await;
            assert!(result.is_err());

            match result.err().unwrap() {
                DotrainRegistryError::InvalidRegistryFormat(msg) => {
                    assert!(msg.contains("Registry file is empty"));
                }
                _ => panic!("Expected InvalidRegistryFormat error for empty registry"),
            }
        }

        #[tokio::test]
        async fn test_registry_with_only_settings() {
            let server = MockServer::start_async().await;

            let settings_only_content = format!("{}/settings.yaml", server.url(""));

            server.mock(|when, then| {
                when.method("GET").path("/settings-only-registry.txt");
                then.status(200).body(settings_only_content);
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(MOCK_SETTINGS_CONTENT);
            });

            let registry =
                DotrainRegistry::new(format!("{}/settings-only-registry.txt", server.url("")))
                    .await
                    .unwrap();

            assert_eq!(
                registry.settings_url(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(registry.settings(), MOCK_SETTINGS_CONTENT);
            assert_eq!(registry.order_urls.len(), 0);
            assert_eq!(registry.orders.len(), 0);

            let keys = registry.get_order_keys().unwrap();
            assert!(keys.is_empty());
        }

        #[tokio::test]
        async fn test_get_gui() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\nfirst-order {}/first-order.rain\nsecond-order {}/second-order.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content);
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(MOCK_SETTINGS_CONTENT);
            });

            server.mock(|when, then| {
                when.method("GET").path("/first-order.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/second-order.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let registry = DotrainRegistry::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(registry.order_urls.len(), 2);
            assert!(registry.order_urls.contains_key("first-order"));
            assert!(registry.order_urls.contains_key("second-order"));
            assert_eq!(registry.orders.len(), 2);

            let gui1 = registry
                .get_gui("first-order".to_string(), "flare".to_string(), None)
                .await
                .unwrap();

            let deployment_details1 = gui1.get_current_deployment().unwrap();
            assert_eq!(deployment_details1.name, "Flare order name");
            assert_eq!(deployment_details1.description, "Flare order description");

            let merged_content1 = registry.merge_content_for_order("first-order").unwrap();
            assert!(merged_content1.contains(MOCK_SETTINGS_CONTENT));
            assert!(merged_content1.contains("_ _: 0 0;"));

            let gui2 = registry
                .get_gui("second-order".to_string(), "base".to_string(), None)
                .await
                .unwrap();

            let deployment_details2 = gui2.get_current_deployment().unwrap();
            assert_eq!(deployment_details2.name, "Base order name");
            assert_eq!(deployment_details2.description, "Base order description");

            let merged_content2 = registry.merge_content_for_order("second-order").unwrap();
            assert!(merged_content2.contains(MOCK_SETTINGS_CONTENT));
            assert!(merged_content2.contains("_ _: 1 1;"));

            assert_ne!(merged_content1, merged_content2);
            assert!(merged_content1.contains("_ _: 0 0;"));
            assert!(!merged_content1.contains("_ _: 1 1;"));
            assert!(merged_content2.contains("_ _: 1 1;"));
            assert!(!merged_content2.contains("_ _: 0 0;"));

            let result = registry
                .get_gui("non-existent-order".to_string(), "flare".to_string(), None)
                .await;
            assert!(result.is_err());
            match result.err().unwrap() {
                DotrainRegistryError::OrderKeyNotFound(key) => {
                    assert_eq!(key, "non-existent-order");
                }
                _ => panic!("Expected OrderKeyNotFound error"),
            }
        }
    }
}
