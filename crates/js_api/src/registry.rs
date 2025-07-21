use crate::gui::{DotrainOrderGui, GuiError};
use rain_orderbook_app_settings::gui::NameAndDescriptionCfg;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
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
    registry_url: String,

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
    settings_url: String,

    /// The content of the shared settings YAML file fetched from `settings_url`.
    ///
    /// Contains shared configuration that will be merged with individual order content
    /// to create complete dotrain files. This typically includes network settings,
    /// contract addresses, and other common configuration that strategies can inherit.
    settings: String,

    /// A map of order keys to their corresponding .rain file URLs.
    ///
    /// Each entry represents an available order strategy where:
    /// - **Key**: Human-readable order identifier (e.g., "fixed-limit", "auction-dca")
    /// - **Value**: URL pointing to the .rain file containing the order's dotrain configuration
    ///
    /// This mapping is parsed from the registry file lines (excluding the first settings line).
    order_urls: HashMap<String, String>,

    /// A map of order keys to their corresponding .rain file contents.
    ///
    /// Each entry contains the raw dotrain content for an order strategy:
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
            DotrainRegistryError::GuiError(err) => {
                format!("GUI error: {}", err)
            }
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
        self.registry_url.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn registry(&self) -> String {
        self.registry.clone()
    }
    #[wasm_bindgen(getter = settingsUrl)]
    pub fn settings_url(&self) -> String {
        self.settings_url.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn settings(&self) -> String {
        self.settings.clone()
    }
    #[wasm_bindgen(getter = orderUrls)]
    pub fn order_urls(&self) -> js_sys::Map {
        let map = js_sys::Map::new();
        for (key, value) in &self.order_urls {
            map.set(&key.into(), &value.into());
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
        let mut instance = DotrainRegistry {
            registry_url: registry_url.clone(),
            registry: "".to_string(),
            settings_url: "".to_string(),
            settings: "".to_string(),
            order_urls: HashMap::new(),
            orders: HashMap::new(),
        };
        instance.fetch_and_parse_registry().await?;
        instance.fetch_and_store_settings().await?;
        instance.fetch_and_store_orders().await?;
        Ok(instance)
    }

    /// Gets strategy details for all orders in the registry.
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
    pub async fn get_all_order_details(
        &self,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, DotrainRegistryError> {
        let mut order_details = BTreeMap::new();

        for (order_key, dotrain) in &self.orders {
            let strategy_details = DotrainOrderGui::get_strategy_details(dotrain.clone()).await?;
            order_details.insert(order_key.clone(), strategy_details);
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
    pub async fn get_order_keys(&self) -> Result<Vec<String>, DotrainRegistryError> {
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
    pub async fn get_deployment_details(
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
        let deployment_details = DotrainOrderGui::get_deployment_details(dotrain.clone()).await?;
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
    async fn fetch_and_parse_registry(&mut self) -> Result<(), DotrainRegistryError> {
        let registry_content = self.fetch_url_content(&self.registry_url).await?;
        self.registry = registry_content.clone();
        self.parse_registry_content(registry_content)?;
        Ok(())
    }

    fn parse_registry_content(&mut self, content: String) -> Result<(), DotrainRegistryError> {
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

        // First line should be the settings URL (no key)
        let first_line = lines[0];
        if first_line.contains(' ') {
            return Err(DotrainRegistryError::InvalidRegistryFormat(
                "First line should be a settings URL without a key".to_string(),
            ));
        }

        self.settings_url = first_line.to_string();

        // Parse remaining lines as order entries
        for line in &lines[1..] {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(DotrainRegistryError::InvalidRegistryFormat(format!(
                    "Invalid order entry format: '{}'. Expected: 'key url'",
                    line
                )));
            }

            let key = parts[0].to_string();
            let url = parts[1].to_string();

            self.order_urls.insert(key, url);
        }

        Ok(())
    }

    async fn fetch_url_content(&self, url: &str) -> Result<String, DotrainRegistryError> {
        let response = reqwest::get(url)
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

    async fn fetch_and_store_settings(&mut self) -> Result<(), DotrainRegistryError> {
        let settings_content = self.fetch_url_content(&self.settings_url).await?;
        self.settings = settings_content;
        Ok(())
    }

    async fn fetch_and_store_orders(&mut self) -> Result<(), DotrainRegistryError> {
        use futures::future::join_all;

        let mut futures = Vec::new();

        for (key, url) in &self.order_urls {
            let key_clone = key.clone();
            let url_clone = url.clone();
            futures.push(async move {
                let content = reqwest::get(&url_clone)
                    .await
                    .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))?
                    .text()
                    .await
                    .map_err(|e| DotrainRegistryError::HttpError(e.to_string()))?;
                Ok::<(String, String), DotrainRegistryError>((key_clone, content))
            });
        }

        let results = join_all(futures).await;

        for result in results {
            let (key, content) = result?;
            self.orders.insert(key, content);
        }

        Ok(())
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
